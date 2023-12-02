macro_rules! Handle {
    ($id:ident, $socket:ident, $expr:expr $(, || $callback:expr)?) => {
        $( $callback; )?
        if let Err(err) = $expr {
            log::error!("{err}");
            $socket.remove_container(&$id, Some(RemoveContainerOptions { force: true, ..Default::default() })).await?;
            log::warn!("removed old container");
        }
    };
}

use crate::{structs::ConnectionData, table, Kind, Level, Response};
use bytes::Bytes;
use flate2::{write::GzEncoder, Compression};
use futures_core::Stream;
use futures_util::{stream::TryStreamExt, SinkExt, StreamExt};
use macros_rs::{fmtstr, str, string};
use rocket_ws::{stream::DuplexStream, Message};
use std::{default::Default, io::Write, path::PathBuf};
use text_placeholder::Template;

use bollard::{
    container::{Config, DownloadFromContainerOptions, RemoveContainerOptions, UploadToContainerOptions},
    errors::Error,
    exec::{CreateExecOptions, StartExecResults},
    image::CreateImageOptions,
    Docker,
};

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

pub async fn exec(mut stream: DuplexStream, docker: &Result<Docker, anyhow::Error>) -> Result<(), anyhow::Error> {
    let socket = &docker.as_ref().unwrap();
    let mut parsed: Option<ConnectionData> = None;

    while parsed.is_none() {
        if let Some(result) = stream.next().await {
            match serde_json::from_str::<ConnectionData>(&result.unwrap().to_string()) {
                Ok(value) => {
                    parsed = Some(value);
                }
                Err(err) => log::error!("Failed to deserialize JSON: {:?}", err),
            }
        }
    }

    let parsed = parsed.unwrap();
    let name = &parsed.info.name;
    let image = parsed.info.remote.image.clone();

    log::info!("creating container (task={name}, image={})", image);

    let image_config = CreateImageOptions {
        from_image: str!(image.clone()),
        ..Default::default()
    };

    let mut container = socket.create_image(Some(image_config), None, None);
    log::info!("image created");

    while let Some(message) = container.next().await {
        let message = message.as_ref().expect("Failed to get CreateImageInfo");
        let formatted = format!(
            "{} {}",
            message.status.clone().unwrap_or_else(|| string!("Waiting")),
            message.progress.clone().unwrap_or_else(|| string!(""))
        );

        let docker_message =
            Response {
                level: Level::Docker,
                message: Some(formatted),
                kind: Kind::Message,
            };

        stream.send(docker_message.into()).await?;
    }

    let config = Config {
        image: Some(image),
        tty: Some(true),
        ..Default::default()
    };

    let id = socket.create_container::<&str, String>(None, config).await?.id;
    log::info!("created container");

    Handle!(id, socket, socket.start_container::<String>(&id, None).await, || log::info!("started container"));

    let binary_message = Response {
        level: Level::Success,
        kind: Kind::Binary,
        message: None,
    };

    stream.send(binary_message.into()).await?;

    if let Some(result) = stream.next().await {
        log::info!("received message: binary");

        let msg = result?;
        let bytes_to_body = |bytes: &[u8]| -> rocket::http::hyper::Body { rocket::http::hyper::Body::from(bytes.to_vec()) };
        let upload_options = UploadToContainerOptions { path: "/opt", ..Default::default() };

        Handle!(id, socket, socket.upload_to_container(&id, Some(upload_options), bytes_to_body(&msg.into_data())).await);
        log::info!("wrote tarfile to container");
    }

    let dependencies = match &parsed.maidfile.tasks[&parsed.info.name].depends {
        Some(deps) => {
            let mut dep_script: Vec<String> = vec![];
            for item in deps.iter() {
                dep_script.push(
                    parsed.maidfile.tasks[item]
                        .script
                        .as_array()
                        .map(|arr| arr.iter().map(|val| val.as_str().unwrap_or_default()).collect::<Vec<_>>().join("\n"))
                        .unwrap_or_default(),
                );
            }
            dep_script.join("\n")
        }
        None => {
            string!("")
        }
    };

    // move common things such as structs and helpers to seperate crate
    let table = table::create(parsed.maidfile.clone(), &parsed.info.args, PathBuf::new().join("/opt"));
    let script = Template::new_with_placeholder(str!(parsed.info.script.join("\n")), "%{", "}").fill_with_hashmap(&table);
    let dependencies = Template::new_with_placeholder(str!(dependencies), "%{", "}").fill_with_hashmap(&table);

    let exec = socket
        .create_exec(
            &id,
            CreateExecOptions {
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                cmd: Some(vec![
                    str!(parsed.info.remote.shell),
                    "-c",
                    fmtstr!("cd /opt && touch script.sh && echo '{dependencies}\n{script}' > script.sh && chmod +x script.sh && ./script.sh"),
                ]),
                ..Default::default()
            },
        )
        .await?
        .id;

    if let StartExecResults::Attached { mut output, .. } = socket.start_exec(&exec, None).await? {
        let build_start_message = Response {
            level: Level::Build,
            kind: Kind::Message,
            message: Some("waiting for build to finish..".to_string()),
        };

        Handle!(id, socket, stream.send(build_start_message.into()).await);

        while let Some(Ok(msg)) = output.next().await {
            if !parsed.info.remote.silent {
                let output_message = Response {
                    level: Level::None,
                    kind: Kind::Message,
                    message: Some(msg.to_string()),
                };

                Handle!(id, socket, stream.send(output_message.into()).await);
            }
        }
    }

    let res =
        socket.download_from_container(
            &id,
            Some(DownloadFromContainerOptions {
                path: fmtstr!("/opt/{}", parsed.info.remote.pull.clone()),
            }),
        );

    let bytes = concat_byte_stream(res).await?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());

    encoder.write_all(&bytes)?;
    let compressed_data = encoder.finish()?;

    Handle!(id, socket, stream.send(Message::binary(compressed_data)).await);
    log::info!("sent message: binary, from [{}]", parsed.info.remote.pull);

    let done_message = Response {
        level: Level::Success,
        kind: Kind::Done,
        message: None,
    };

    stream.send(done_message.into()).await?;
    log::info!("sent message: [done]");

    socket.remove_container(&id, Some(RemoveContainerOptions { force: true, ..Default::default() })).await?;
    log::info!("removed old container");

    Ok(())
}
