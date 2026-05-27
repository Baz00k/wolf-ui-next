use futures_util::StreamExt;
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{ApiError, WolfApi};

#[derive(Clone, Copy, Debug)]
pub struct Docker<'api> {
    api: &'api WolfApi,
}

impl<'api> Docker<'api> {
    pub(crate) fn new(api: &'api WolfApi) -> Self {
        Self { api }
    }

    pub async fn image_exists(&self, image_name: &str) -> Result<bool, ApiError> {
        let response = self
            .api
            .http_client()
            .get(self.api.url("/api/v1/docker/images/inspect"))
            .query(&[("image_name", image_name)])
            .send()
            .await
            .map_err(ApiError::from_reqwest)?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(false);
        }

        if response.status().is_success() {
            return Ok(true);
        }

        crate::client::decode_bytes(response).await.map(|_| true)
    }

    pub async fn pull_image<F>(
        &self,
        image_name: &str,
        mut on_progress: F,
    ) -> Result<bool, ApiError>
    where
        F: FnMut(f64),
    {
        let mut response = self
            .api
            .post_stream(
                "/api/v1/docker/images/pull",
                &crate::types::WolfApiDockerPullImageRequest {
                    image_name: image_name.to_string(),
                },
            )
            .await?
            .bytes_stream();
        let mut buffer = String::new();
        let mut layers = Vec::<LayerProgress>::new();
        let mut last_current = 0;
        let mut unpacking = false;
        let mut downloaded = false;

        on_progress(0.0);

        while let Some(chunk) = response.next().await {
            buffer.push_str(&String::from_utf8_lossy(
                &chunk.map_err(ApiError::from_reqwest)?,
            ));

            while let Some(index) = buffer.find('\n') {
                let line = buffer[..index].trim().to_string();
                buffer.replace_range(..=index, "");

                if line.is_empty() {
                    continue;
                }

                let event = serde_json::from_str::<PullProgress>(&line)
                    .map_err(ApiError::from_serde_json)?;
                if event.success.unwrap_or(false) {
                    on_progress(100.0);
                    return Ok(downloaded);
                }

                let Some(layer_id) = event.layer_id else {
                    continue;
                };

                downloaded = true;
                upsert_layer(&mut layers, layer_id, event.current_progress, event.total);
                let current = layers.iter().map(|layer| layer.current).sum::<i64>();
                let total = layers.iter().map(|layer| layer.total).sum::<i64>();
                if total <= 500 {
                    continue;
                }

                if last_current > 0 && last_current > current + last_current * 3 / 10 {
                    unpacking = true;
                }
                last_current = current;

                let progress = if unpacking {
                    50.0 + (current as f64 * 50.0 / total as f64)
                } else {
                    current as f64 * 50.0 / total as f64
                };
                on_progress(progress.clamp(0.0, 99.0));
            }
        }

        Err(ApiError::Status {
            status: StatusCode::OK,
            body: "Docker pull stream ended before Wolf reported success".to_string(),
        })
    }
}

#[derive(Deserialize)]
struct PullProgress {
    success: Option<bool>,
    layer_id: Option<String>,
    #[serde(default)]
    current_progress: i64,
    #[serde(default)]
    total: i64,
}

struct LayerProgress {
    id: String,
    current: i64,
    total: i64,
}

fn upsert_layer(layers: &mut Vec<LayerProgress>, id: String, current: i64, total: i64) {
    if let Some(layer) = layers.iter_mut().find(|layer| layer.id == id) {
        layer.current = current;
        layer.total = total;
    } else {
        layers.push(LayerProgress { id, current, total });
    }
}
