use std::sync::Arc;

use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::interface::module::{Modules, ModulesExt};

const REMOTE_LAUNCH_RETRY_SECONDS: u64 = 5;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum BrokerMessage {
    #[serde(rename = "launch-work")]
    LaunchWork {
        #[serde(rename = "workId")]
        work_id: String,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum BrokerAckMessage<'a> {
    #[serde(rename = "launch-ack")]
    LaunchAck {
        #[serde(rename = "workId")]
        work_id: &'a str,
        status: &'a str,
    },
}

pub fn spawn_remote_launch_client(modules: Arc<Modules>) {
    tauri::async_runtime::spawn(async move {
        loop {
            if let Err(err) = run_once(modules.clone()).await {
                log::warn!("remote launch connection failed: {err}");
            }

            sleep(Duration::from_secs(REMOTE_LAUNCH_RETRY_SECONDS)).await;
        }
    });
}

async fn run_once(modules: Arc<Modules>) -> anyhow::Result<()> {
    let settings = modules.app_settings_use_case().get_remote_share_settings().await?;
    let server_base_url = match settings.remote_share_server_base_url {
        Some(value) if !value.is_empty() => value,
        _ => return Ok(()),
    };
    let device_id = match settings.remote_share_device_id {
        Some(value) if !value.is_empty() => value,
        _ => return Ok(()),
    };
    let device_secret = match settings.remote_share_device_secret {
        Some(value) if !value.is_empty() => value,
        _ => return Ok(()),
    };

    let broker_url = build_remote_launch_ws_url(&server_base_url, &device_id, &device_secret)?;
    let (mut stream, _) = connect_async(broker_url).await?;
    log::info!("remote launch broker connected");

    while let Some(message) = stream.next().await {
        let message = message?;
        match message {
            Message::Text(text) => {
                if let Ok(payload) = serde_json::from_str::<BrokerMessage>(&text) {
                    handle_broker_message(&modules, payload, &mut stream).await;
                }
            }
            Message::Close(_) => break,
            Message::Ping(value) => {
                stream.send(Message::Pong(value)).await?;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn handle_broker_message(
    modules: &Arc<Modules>,
    payload: BrokerMessage,
    stream: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) {
    match payload {
        BrokerMessage::LaunchWork { work_id } => {
            let status = match launch_first_work_link(modules, &work_id).await {
                Ok(true) => "queued",
                Ok(false) => "not-found",
                Err(err) => {
                    log::warn!("failed to launch work {work_id}: {err}");
                    "error"
                }
            };

            let ack = BrokerAckMessage::LaunchAck {
                work_id: &work_id,
                status,
            };

            if let Ok(text) = serde_json::to_string(&ack) {
                let _ = stream.send(Message::Text(text)).await;
            }
        }
    }
}

async fn launch_first_work_link(modules: &Arc<Modules>, work_id: &str) -> anyhow::Result<bool> {
    let links = modules.work_use_case().list_work_lnks(work_id.to_string()).await?;
    let Some((first_lnk_id, _)) = links.first() else {
        return Ok(false);
    };

    let _ = modules
        .work_use_case()
        .launch_work(false, *first_lnk_id)
        .await?;

    Ok(true)
}

fn build_remote_launch_ws_url(
    server_base_url: &str,
    device_id: &str,
    device_secret: &str,
) -> anyhow::Result<String> {
    let trimmed = server_base_url.trim().trim_end_matches('/');
    let mut url = url::Url::parse(trimmed)?;
    match url.scheme() {
        "https" => {
            url.set_scheme("wss")
                .map_err(|_| anyhow::anyhow!("failed to set wss scheme"))?;
        }
        "http" => {
            url.set_scheme("ws")
                .map_err(|_| anyhow::anyhow!("failed to set ws scheme"))?;
        }
        _ => anyhow::bail!("unsupported remote share server scheme"),
    }

    url.set_path(&format!("/api/device/{}/launch-broker", device_id));
    url.query_pairs_mut()
        .append_pair("deviceSecret", device_secret);

    Ok(url.to_string())
}

#[cfg(test)]
mod tests {
    use super::build_remote_launch_ws_url;

    #[test]
    fn build_remote_launch_ws_url_httpsをwssに変換する() {
        let url = build_remote_launch_ws_url("https://example.com/", "device-id", "secret")
            .unwrap();

        assert_eq!(
            url,
            "wss://example.com/api/device/device-id/launch-broker?deviceSecret=secret"
        );
    }
}
