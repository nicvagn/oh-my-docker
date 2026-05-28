use anyhow::Result;
use bollard::Docker;
use bollard::exec::{CreateExecOptions, StartExecOptions, StartExecResults};
use bollard::container::{LogOutput, UploadToContainerOptions, DownloadFromContainerOptions};
use futures_util::StreamExt;
use tokio::sync::mpsc::UnboundedSender;
use std::path::Path;

use crate::app::event::AppEvent;
use crate::app::state::ExplorerEntry;

pub async fn list_container_dir(
    docker: &Docker,
    container_id: &str,
    path: &str,
) -> Result<Vec<ExplorerEntry>> {
    let exec = docker
        .create_exec(
            container_id,
            CreateExecOptions {
                cmd: Some(vec![
                    "ls".to_string(),
                    "-1ap".to_string(),
                    "--".to_string(),
                    path.to_string(),
                ]),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                ..Default::default()
            },
        )
        .await?;

    let output = docker.start_exec(&exec.id, None::<StartExecOptions>).await?;

    let mut stdout_bytes = Vec::new();

    if let StartExecResults::Attached { mut output, .. } = output {
        while let Some(msg_result) = output.next().await {
            match msg_result {
                Ok(LogOutput::StdOut { message }) => {
                    stdout_bytes.extend_from_slice(&message);
                }
                Ok(LogOutput::StdErr { message }) => {
                    let stderr = String::from_utf8_lossy(&message);
                    if !stderr.trim().is_empty() {
                        // ignore stderr, stdout may still have partial output
                    }
                }
                _ => {}
            }
        }
    }

    let stdout = String::from_utf8_lossy(&stdout_bytes);
    let mut entries = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let is_dir = line.ends_with('/');
        let name = if is_dir {
            line.strip_suffix('/').unwrap_or(line)
        } else {
            line
        };
        if name == "." || name == ".." {
            continue;
        }
        entries.push(ExplorerEntry {
            name: name.to_string(),
            is_dir,
            size: 0,
            modified: String::new(),
            permissions: String::new(),
        });
    }

    entries.sort_by(|a, b| {
        a.is_dir.cmp(&b.is_dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(entries)
}

pub async fn list_host_dir(path: &str) -> Result<Vec<ExplorerEntry>> {
    let mut entries = Vec::new();
    let dir = std::fs::read_dir(path)?;

    for entry in dir {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let name = entry.file_name()
            .to_string_lossy()
            .to_string();

        if name == "." || name == ".." {
            continue;
        }

        entries.push(ExplorerEntry {
            name,
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified: String::new(),
            permissions: String::new(),
        });
    }

    entries.sort_by(|a, b| {
        a.is_dir.cmp(&b.is_dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(entries)
}

pub async fn copy_to_container(
    docker: &Docker,
    container_id: &str,
    host_path: String,
    container_path: String,
) -> Result<()> {
    let path = Path::new(&host_path);
    if !path.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", host_path));
    }

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;

    let mut tar_bytes = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_bytes);

        if path.is_dir() {
            builder.append_dir_all(file_name, path)?;
        } else {
            let mut file = std::fs::File::open(path)?;
            builder.append_file(file_name, &mut file)?;
        }

        builder.finish()?;
    }

    docker
        .upload_to_container(
            container_id,
            Some(UploadToContainerOptions {
                path: container_path,
                no_overwrite_dir_non_dir: "false".to_string(),
            }),
            tar_bytes.into(),
        )
        .await?;

    Ok(())
}

pub async fn copy_from_container(
    docker: &Docker,
    container_id: &str,
    container_path: String,
    host_dest_dir: &str,
) -> Result<std::path::PathBuf> {
    let stream = docker.download_from_container(
        container_id,
        Some(DownloadFromContainerOptions {
            path: container_path.clone(),
        }),
    );

    let mut all_bytes = Vec::new();
    let mut stream = stream;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        all_bytes.extend_from_slice(&chunk);
    }

    let dest_dir = Path::new(host_dest_dir);
    let mut archive = tar::Archive::new(all_bytes.as_slice());
    archive.unpack(dest_dir)?;

    let dest = dest_dir.join(
        Path::new(&container_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "file".to_string()),
    );

    Ok(dest)
}

pub fn spawn_list_container_dir(
    docker: Docker,
    tx: UnboundedSender<AppEvent>,
    container_id: String,
    path: String,
) {
    tokio::spawn(async move {
        match list_container_dir(&docker, &container_id, &path).await {
            Ok(entries) => {
                let _ = tx.send(AppEvent::ExplorerContainerDirUpdated(container_id, path, entries));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::ExplorerTransferError(format!("Failed to list directory: {}", e)));
            }
        }
    });
}

pub async fn delete_in_container(
    docker: &Docker,
    container_id: &str,
    path: &str,
) -> Result<()> {
    let exec = docker
        .create_exec(
            container_id,
            CreateExecOptions {
                cmd: Some(vec![
                    "rm".to_string(),
                    "-rf".to_string(),
                    "--".to_string(),
                    path.to_string(),
                ]),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                ..Default::default()
            },
        )
        .await?;

    let output = docker.start_exec(&exec.id, None::<StartExecOptions>).await?;

    if let StartExecResults::Attached { mut output, .. } = output {
        while let Some(msg_result) = output.next().await {
            match msg_result {
                Ok(LogOutput::StdErr { message }) => {
                    let stderr = String::from_utf8_lossy(&message);
                    let trimmed = stderr.trim();
                    if !trimmed.is_empty() {
                        return Err(anyhow::anyhow!("{}", trimmed));
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

pub async fn rename_in_container(
    docker: &Docker,
    container_id: &str,
    old_path: &str,
    new_path: &str,
) -> Result<()> {
    let exec = docker
        .create_exec(
            container_id,
            CreateExecOptions {
                cmd: Some(vec![
                    "mv".to_string(),
                    "--".to_string(),
                    old_path.to_string(),
                    new_path.to_string(),
                ]),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                ..Default::default()
            },
        )
        .await?;

    let output = docker.start_exec(&exec.id, None::<StartExecOptions>).await?;

    if let StartExecResults::Attached { mut output, .. } = output {
        while let Some(msg_result) = output.next().await {
            match msg_result {
                Ok(LogOutput::StdErr { message }) => {
                    let stderr = String::from_utf8_lossy(&message);
                    let trimmed = stderr.trim();
                    if !trimmed.is_empty() {
                        return Err(anyhow::anyhow!("{}", trimmed));
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}
