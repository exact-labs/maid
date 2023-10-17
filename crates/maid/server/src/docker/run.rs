use crate::table;
use crate::structs::ConnectionData;

use bytes::Bytes;
use bollard::{Docker, errors::Error};
use macros_rs::{string, fmtstr, str};
use warp::ws::{Message, WebSocket};
use bollard::image::CreateImageOptions;
use futures_util::{SinkExt, StreamExt};
use futures_core::Stream;
use futures_util::stream::{SplitSink, SplitStream, TryStreamExt};
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::container::{Config, RemoveContainerOptions, UploadToContainerOptions, DownloadFromContainerOptions};
use std::default::Default;
use warp::hyper::Body;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::PathBuf;
use text_placeholder::Template;
use flate2::{Compression, write::GzEncoder};
use std::io::Write;

pub async fn concat_byte_stream<S>(s: S) -> Result<Vec<u8>, Error>
where
    S: Stream<Item = Result<Bytes, Error>>,
{
    s.try_fold(Vec::new(), |mut acc, chunk| async move {
        acc.extend_from_slice(&chunk[..]);
        Ok(acc)
    })
    .await
}

// add error handling to all the unwraps
pub async fn exec(tx: SplitSink<WebSocket, Message>, mut rx: SplitStream<WebSocket>, docker: Docker) -> Result<(), Box<dyn std::error::Error + 'static>> { 
    let mut parsed: Option<ConnectionData> = None;
    let tx_ref = Arc::new(Mutex::new(tx));
    
    while parsed.is_none() {
        if let Some(result) = rx.next().await {
            let msg = result.unwrap();              
            match serde_json::from_str::<ConnectionData>(msg.to_str().unwrap()) {
                Ok(value) => {
                    parsed = Some(value);
                }
                Err(err) => {
                    eprintln!("Failed to deserialize JSON: {:?}", err);
                }
            }
        }
    }
        
    let parsed = parsed.unwrap();
    let name = &parsed.info.name;
    
    println!("creating container for task [{name}]");
    docker
        .create_image(
            Some(CreateImageOptions {
                from_image: str!(parsed.info.remote.image.clone()),
                ..Default::default()
            }),
            None,
            None,
        )
        .for_each(|msg| {
            let tx_ref = Arc::clone(&tx_ref);
            
            async move {
                let msg = msg.as_ref().expect("Failed to get CreateImageInfo");
                let formatted = format!(
                    "{} {}",
                    msg.status.clone().unwrap_or_else(|| string!("Waiting")),
                    msg.progress.clone().unwrap_or_else(|| string!(""))
                );
                
                let mut tx_lock = tx_ref.lock().await;
                tx_lock.send(Message::text(
                    serde_json::to_string(&serde_json::json!({
                        "level": "docker",
                        "time": chrono::Utc::now().timestamp_millis(),
                        "data": { "message": formatted },
                    }))
                    .unwrap(),
                ))
                .await;   
            }
        })
        .await;

    let config = Config {
        image: Some(parsed.info.remote.image),
        tty: Some(true),
        ..Default::default()
    };

    let id = docker.create_container::<&str, String>(None, config).await?.id;
    println!("created container");
    
    docker.start_container::<String>(&id, None).await?;
    println!("started container");
    
    let tx_ref = Arc::clone(&tx_ref);
    let mut tx_lock = tx_ref.lock().await;
    
    tx_lock.send(Message::text(
        serde_json::to_string(&serde_json::json!({
            "level": "success",
            "time": chrono::Utc::now().timestamp_millis(),
            "data": { "binary": true },
        }))
        .unwrap(),
    ))
    .await
    .unwrap();
    
    if let Some(result) = rx.next().await {
        println!("received message: binary");
    
        let msg = result.unwrap();
        fn bytes_to_body(bytes: &[u8]) -> Body {
            Body::from(bytes.to_vec())
        }
        
        // note: this `Result` may be an `Err` variant, which should be handled
        // help: use `let _ = ...` to ignore the resulting value
        docker.upload_to_container(&id, Some(UploadToContainerOptions{ path: "/opt", ..Default::default() }), bytes_to_body(&msg.as_bytes())).await;
        println!("wrote tarfile to container");
    }
    
    let dependencies = match &parsed.maidfile.tasks[&parsed.info.name].depends {
        Some(deps) => {     
            let mut dep_script: Vec<String> = vec![];
            for item in deps.iter() {
                dep_script.push(
                    parsed.maidfile.tasks[item]
                        .script
                        .as_array()
                        .map(|arr| {
                            arr.iter()
                                .map(|val| val.as_str().unwrap_or_default())
                                .collect::<Vec<_>>()
                                .join("\n")
                        })
                        .unwrap_or_default()
                );
            };
            dep_script.join("\n")
        }
        None => { string!("") }
    };
    
    // move common things such as structs and helpers to seperate crate
    let table = table::create(parsed.maidfile.clone(), &parsed.info.args, PathBuf::new().join("/opt"));
    let script = Template::new_with_placeholder(str!(parsed.info.script.join("\n")), "%{", "}").fill_with_hashmap(&table);
    let dependencies = Template::new_with_placeholder(str!(dependencies), "%{", "}").fill_with_hashmap(&table);

    let exec = docker
        .create_exec(
            &id,
            CreateExecOptions {
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                cmd: Some(vec![str!(parsed.info.remote.shell), "-c", fmtstr!("cd /opt && touch script.sh && echo '{dependencies}\n{script}' > script.sh && chmod +x script.sh && ./script.sh")]),
                ..Default::default()
            },
        )
        .await?
        .id;

    if let StartExecResults::Attached { mut output, .. } = docker.start_exec(&exec, None).await? {
        tx_lock.send(Message::text(
            serde_json::to_string(&serde_json::json!({
                "level": "build",
                "time": chrono::Utc::now().timestamp_millis(),
                "data": { "message": "waiting for build to finish..." },
            }))
            .unwrap(),
        ))
        .await
        .unwrap();
        
        while let Some(Ok(msg)) = output.next().await {
            if !parsed.info.remote.silent {
                let parsed = format!("{msg}");   
                if parsed != "" {
                    tx_lock.send(Message::text(
                        serde_json::to_string(&serde_json::json!({
                            "level": "build",
                            "time": chrono::Utc::now().timestamp_millis(),
                            "data": { "message": parsed.trim() },
                        }))
                        .unwrap(),
                    ))
                    .await
                    .unwrap();
                }
            }
        }
    }
    
    let res = docker.download_from_container(&id, Some(DownloadFromContainerOptions{ path: fmtstr!("/opt/{}", parsed.info.remote.pull.clone()) }));
    let bytes = concat_byte_stream(res).await?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    
    encoder.write_all(&bytes)?;
    let compressed_data = encoder.finish()?;
    
    tx_lock.send(Message::binary(compressed_data)).await.unwrap();
    println!("sent message: binary, from [{}]", parsed.info.remote.pull);
    
    tx_lock.send(Message::text(
        serde_json::to_string(&serde_json::json!({
            "level": "success",
            "time": chrono::Utc::now().timestamp_millis(),
            "data": { "done": true, "message": "" },
        }))
        .unwrap(),
    ))
    .await
    .unwrap();
    println!("sent message: [done]");
    
    
    println!("deleted old container");
    // delete container if socket closed
    docker.remove_container(&id, Some(RemoveContainerOptions { force: true, ..Default::default() })).await?;
    Ok(())
}
