use bollard::container::{ListContainersOptions, StatsOptions};
use bollard::Docker;
use futures_util::stream::StreamExt;
use std::collections::HashMap;

pub async fn list(docker: &Docker) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut filter: HashMap<String, Vec<String>> = HashMap::new();
    let mut container_list: Vec<String> = vec![];

    filter.insert(String::from("status"), vec![String::from("running")]);
    let containers =
        &docker
            .list_containers(Some(ListContainersOptions {
                all: true,
                filters: filter,
                ..Default::default()
            }))
            .await?;

    if containers.is_empty() {
        Ok(container_list)
    } else {
        for container in containers {
            let container_id = container.id.as_ref().unwrap();
            let stream = &mut docker.stats(container_id, Some(StatsOptions { stream: false, ..Default::default() })).take(1);

            while let Some(Ok(_)) = stream.next().await {
                container_list.push(format!("{}:{}", container.names.as_ref().unwrap()[0], container.image.as_ref().unwrap_or(&"".to_string())));
            }
        }

        Ok(container_list)
    }
}
