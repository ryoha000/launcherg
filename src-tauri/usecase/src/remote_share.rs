use anyhow::Context;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteShareThumbnailInput {
    pub content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteShareWorkInput {
    pub work_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub erogamescape_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<RemoteShareThumbnailInput>,
}

impl RemoteShareWorkInput {
    pub fn dedupe_key(&self) -> String {
        match self.erogamescape_id {
            Some(erogamescape_id) => format!("egs:{}", erogamescape_id),
            None => format!("work:{}", self.work_id),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteShareUploadTarget {
    pub work_id: String,
    pub dedupe_key: String,
    pub image_key: String,
    pub upload_url: String,
    pub content_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteShareUploadedImage {
    pub dedupe_key: String,
    pub image_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrepareSyncRequest {
    device_secret: String,
    works: Vec<RemoteShareWorkInput>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrepareSyncResponse {
    device_id: String,
    upload_targets: Vec<RemoteShareUploadTarget>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommitSyncRequest {
    device_secret: String,
    works: Vec<RemoteShareWorkInput>,
    uploaded_images: Vec<RemoteShareUploadedImage>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitSyncResponse {
    pub device_id: String,
    pub synced_count: i32,
    pub last_synced_at: String,
}

#[derive(Clone)]
pub struct RemoteShareUseCase {
    client: Client,
}

impl Default for RemoteShareUseCase {
    fn default() -> Self {
        Self::new()
    }
}

impl RemoteShareUseCase {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn register_device(
        &self,
        server_base_url: &str,
        device_secret: &str,
    ) -> anyhow::Result<String> {
        let endpoint = format!("{}/api/device/register", trim_server_base_url(server_base_url));
        let response = self
            .client
            .post(endpoint)
            .json(&RegisterDeviceRequest {
                device_secret: device_secret.to_string(),
            })
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("register device failed: {}", body));
        }

        Ok(response.json::<RegisterDeviceResponse>().await?.device_id)
    }

    pub async fn prepare_sync_works(
        &self,
        server_base_url: &str,
        device_id: &str,
        device_secret: &str,
        works: Vec<RemoteShareWorkInput>,
    ) -> anyhow::Result<Vec<RemoteShareUploadTarget>> {
        let endpoint = format!(
            "{}/api/device/{}/works/sync/prepare",
            trim_server_base_url(server_base_url),
            device_id,
        );
        let response = self
            .client
            .post(endpoint)
            .json(&PrepareSyncRequest {
                device_secret: device_secret.to_string(),
                works,
            })
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("prepare sync failed: {}", body));
        }

        Ok(response.json::<PrepareSyncResponse>().await?.upload_targets)
    }

    pub async fn upload_thumbnail(
        &self,
        upload_url: &str,
        content_type: &str,
        path: &Path,
    ) -> anyhow::Result<()> {
        let bytes = std::fs::read(path).with_context(|| {
            format!("failed to read thumbnail file: {}", path.display())
        })?;
        let response = self
            .client
            .put(upload_url)
            .header("Content-Type", content_type)
            .body(bytes)
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("thumbnail upload failed: {}", body));
        }

        Ok(())
    }

    pub async fn commit_sync_works(
        &self,
        server_base_url: &str,
        device_id: &str,
        device_secret: &str,
        works: Vec<RemoteShareWorkInput>,
        uploaded_images: Vec<RemoteShareUploadedImage>,
    ) -> anyhow::Result<CommitSyncResponse> {
        let endpoint = format!(
            "{}/api/device/{}/works/sync/commit",
            trim_server_base_url(server_base_url),
            device_id,
        );
        let response = self
            .client
            .post(endpoint)
            .json(&CommitSyncRequest {
                device_secret: device_secret.to_string(),
                works,
                uploaded_images,
            })
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("commit sync failed: {}", body));
        }

        Ok(response.json::<CommitSyncResponse>().await?)
    }

    pub fn build_share_url(&self, server_base_url: &str, device_id: &str) -> anyhow::Result<String> {
        let base = trim_server_base_url(server_base_url);
        let parsed = url::Url::parse(&format!("{}/", base))?;
        Ok(parsed
            .join(device_id)
            .context("failed to build share url")?
            .to_string())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterDeviceRequest {
    device_secret: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterDeviceResponse {
    device_id: String,
}

fn trim_server_base_url(value: &str) -> String {
    value.trim().trim_end_matches('/').to_string()
}

#[cfg(test)]
mod tests {
    use super::RemoteShareUseCase;

    #[test]
    fn build_share_url_末尾スラッシュを除去する() {
        let usecase = RemoteShareUseCase::new();
        let url = usecase
            .build_share_url("https://example.com/", "device-id")
            .unwrap();

        assert_eq!(url, "https://example.com/device-id");
    }

    #[test]
    fn dedupe_key_erogamescape_id_を優先する() {
        let work = super::RemoteShareWorkInput {
            work_id: "work-1".to_string(),
            title: "Title".to_string(),
            erogamescape_id: Some(123),
            thumbnail: None,
        };

        assert_eq!(work.dedupe_key(), "egs:123");
    }
}
