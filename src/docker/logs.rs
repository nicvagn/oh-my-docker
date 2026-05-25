use bollard::Docker;
use bollard::container::LogsOptions;
use bollard::container::LogOutput;
use anyhow::Result;
use futures_util::StreamExt;
use tokio::sync::mpsc::UnboundedSender;

use crate::app::event::{AppEvent, LogEntry};

pub async fn stream_logs(docker: Docker, id: String, tx: UnboundedSender<AppEvent>) -> Result<()> {
    let mut since: i64 = 0;
    let mut first = true;

    loop {
        let options = LogsOptions::<String> {
            follow: false,
            stdout: true,
            stderr: true,
            timestamps: true,
            tail: if first { "100".to_string() } else { "all".to_string() },
            since,
            ..Default::default()
        };

        let mut stream = docker.logs(&id, Some(options));
        let mut batch = Vec::new();
        let mut latest: i64 = since;

        while let Some(msg_result) = stream.next().await {
            match msg_result {
                Ok(msg) => {
                    let raw = match msg {
                        LogOutput::StdOut { message: ref bytes }
                        | LogOutput::Console { message: ref bytes } => {
                            String::from_utf8_lossy(bytes).to_string()
                        }
                        LogOutput::StdErr { message: ref bytes } => {
                            String::from_utf8_lossy(bytes).to_string()
                        }
                        _ => continue,
                    };

                    let (ts_str, message) = match raw.split_once(' ') {
                        Some((ts, rest)) => (ts.to_string(), rest.to_string()),
                        None => (String::new(), raw.clone()),
                    };

                    if let Ok(epoch) = parse_docker_timestamp(&ts_str) {
                        latest = latest.max(epoch);
                    }

                    batch.push(LogEntry { message });
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::Info(format!("Log poll error: {}", e)));
                }
            }
        }

        if !batch.is_empty() {
            if tx.send(AppEvent::LogLines(id.clone(), batch)).is_err() {
                return Ok(());
            }
            since = latest + 1;
        }

        first = false;
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
}

fn parse_docker_timestamp(s: &str) -> Result<i64, ()> {
    if s.len() < 20 {
        return Err(());
    }
    let datetime_str = &s[..19];
    let secs = match chrono::NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S") {
        Ok(dt) => dt.and_utc().timestamp(),
        Err(_) => return Err(()),
    };
    Ok(secs)
}
