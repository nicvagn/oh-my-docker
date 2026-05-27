use bollard::Docker;
use bollard::container::{ListContainersOptions, StopContainerOptions, RestartContainerOptions};
use bollard::container::RemoveContainerOptions;
use anyhow::Result;
use serde_json::Value;

use crate::app::event::ContainerSummary;

fn extract_label(labels: &Option<std::collections::HashMap<String, String>>, key: &str) -> String {
    labels
        .as_ref()
        .and_then(|l| l.get(key).map(|s| s.to_string()))
        .unwrap_or_default()
}

fn extract_project(labels: &Option<std::collections::HashMap<String, String>>) -> String {
    extract_label(labels, "com.docker.compose.project")
}

fn extract_service(labels: &Option<std::collections::HashMap<String, String>>) -> String {
    extract_label(labels, "com.docker.compose.service")
}

fn extract_config_file(labels: &Option<std::collections::HashMap<String, String>>) -> String {
    extract_label(labels, "com.docker.compose.config-files")
}

pub async fn list_containers(docker: &Docker) -> Result<Vec<ContainerSummary>> {
    let options = ListContainersOptions::<String> {
        all: true,
        size: true,
        ..Default::default()
    };

    let containers = docker.list_containers(Some(options)).await?;

    let summaries = containers
        .into_iter()
        .map(|c| {
            let name = c
                .names
                .as_ref()
                .and_then(|v| v.first())
                .cloned()
                .unwrap_or_default()
                .trim_start_matches('/')
                .to_string();

            let ports_vec = c.ports.as_ref().map(|p| p.as_slice()).unwrap_or(&[]);
    let ports = ports_vec
                .iter()
                .map(|p| {
                    let typ = p.typ.as_ref().map(|t| format!("{}", t)).unwrap_or_else(|| "tcp".to_string());
                    if let (Some(host), Some(hp)) = (&p.ip, p.public_port) {
                        format!("{}:{}->{}/{}", host, hp, p.private_port, typ)
                    } else {
                        format!("{}/{}", p.private_port, typ)
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");

            let health = c.status.as_ref()
                .and_then(|s| {
                    if s.contains("(healthy)") { Some("healthy".into()) }
                    else if s.contains("(unhealthy)") { Some("unhealthy".into()) }
                    else if s.contains("(health:") { Some("starting".into()) }
                    else { None }
                })
                .unwrap_or_default();

            ContainerSummary {
                id: c.id.as_ref().map(|s| s.clone()).unwrap_or_default(),
                name,
                image: c.image.as_ref().map(|s| s.clone()).unwrap_or_default(),
                state: c.state.as_ref().map(|s| s.clone()).unwrap_or_default(),
                status: c.status.as_ref().map(|s| s.clone()).unwrap_or_default(),
                ports,
                project: extract_project(&c.labels),
                service: extract_service(&c.labels),
                compose_file: extract_config_file(&c.labels),
                health,
            }
        })
        .collect();

    Ok(summaries)
}

pub async fn inspect_container(docker: &Docker, id: &str) -> Result<(Value, String)> {
    let inspect = docker.inspect_container(id, None).await?;
    let json = serde_json::to_value(&inspect)?;
    let name = inspect.name.unwrap_or_default().trim_start_matches('/').to_string();
    Ok((json, name))
}

pub async fn start_container(docker: &Docker, id: &str) -> Result<()> {
    docker.start_container::<String>(id, None).await?;
    Ok(())
}

pub async fn stop_container(docker: &Docker, id: &str) -> Result<()> {
    docker.stop_container(id, None::<StopContainerOptions>).await?;
    Ok(())
}

pub async fn restart_container(docker: &Docker, id: &str) -> Result<()> {
    docker.restart_container(id, None::<RestartContainerOptions>).await?;
    Ok(())
}

pub async fn delete_container(docker: &Docker, id: &str) -> Result<()> {
    docker.remove_container(id, None::<RemoveContainerOptions>).await?;
    Ok(())
}
