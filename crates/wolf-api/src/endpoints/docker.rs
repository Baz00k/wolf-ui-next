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
        let path = "/api/v1/docker/images/inspect";
        let context = self.api.request_context("GET", path);
        let response = self
            .api
            .get_response_with_query(path, &[("image_name", image_name)])
            .await?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(false);
        }

        if response.status().is_success() {
            return Ok(true);
        }

        let error = crate::client::response_error(response).await;
        self.api
            .log_api_error(context, &error, "Wolf API response failed");
        Err(error)
    }

    pub async fn pull_image<F>(
        &self,
        image_name: &str,
        mut on_progress: F,
    ) -> Result<bool, ApiError>
    where
        F: FnMut(f64),
    {
        let response = self
            .api
            .post_stream(
                "/api/v1/docker/images/pull",
                &crate::types::WolfApiDockerPullImageRequest {
                    image_name: image_name.to_string(),
                },
            )
            .await?;

        let context = self
            .api
            .request_context("POST", "/api/v1/docker/images/pull");
        let result = read_pull_stream(response, &mut on_progress).await;
        result.inspect_err(|error| {
            self.api
                .log_api_error(context, error, "Wolf API stream failed")
        })
    }
}

async fn read_pull_stream<F>(
    response: reqwest::Response,
    on_progress: &mut F,
) -> Result<bool, ApiError>
where
    F: FnMut(f64),
{
    let mut response = response.bytes_stream();
    let mut buffer = String::new();
    let mut progress = PullProgressState::default();
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

            let event =
                serde_json::from_str::<PullProgress>(&line).map_err(ApiError::from_serde_json)?;
            if event.success.unwrap_or(false) {
                on_progress(100.0);
                return Ok(downloaded);
            }

            let Some(layer_id) = event.layer_id else {
                continue;
            };

            downloaded = true;
            if let Some(progress) =
                progress.update_layer(layer_id, event.current_progress, event.total)
            {
                on_progress(progress);
            }
        }
    }

    Err(ApiError::Status {
        status: StatusCode::OK,
        body: "Docker pull stream ended before Wolf reported success".to_string(),
    })
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

#[derive(Default)]
struct PullProgressState {
    layers: Vec<LayerProgress>,
    last_current: i64,
    last_progress: f64,
    unpacking: bool,
}

impl PullProgressState {
    fn update_layer(&mut self, layer_id: String, current_progress: i64, total: i64) -> Option<f64> {
        upsert_layer(&mut self.layers, layer_id, current_progress, total);
        let current = self.layers.iter().map(|layer| layer.current).sum::<i64>();
        let total = self.layers.iter().map(|layer| layer.total).sum::<i64>();
        if total <= 500 {
            return None;
        }

        if self.last_current > 0 && self.last_current > current + self.last_current * 3 / 10 {
            self.unpacking = true;
        }
        self.last_current = current;

        let progress = if self.unpacking {
            50.0 + (current as f64 * 50.0 / total as f64)
        } else {
            current as f64 * 50.0 / total as f64
        };
        self.last_progress = self.last_progress.max(progress.clamp(0.0, 99.0));
        Some(self.last_progress)
    }
}

fn upsert_layer(layers: &mut Vec<LayerProgress>, id: String, current: i64, total: i64) {
    if let Some(layer) = layers.iter_mut().find(|layer| layer.id == id) {
        layer.current = current;
        layer.total = total;
    } else {
        layers.push(LayerProgress { id, current, total });
    }
}

#[cfg(test)]
mod tests {
    use super::PullProgressState;

    #[test]
    fn progress_does_not_decrease_when_layer_current_regresses() {
        let mut progress = PullProgressState::default();

        let first = progress.update_layer("layer-a".to_string(), 800, 1000);
        let second = progress.update_layer("layer-a".to_string(), 700, 1000);

        assert_eq!(first, Some(40.0));
        assert_eq!(second, first);
    }

    #[test]
    fn progress_does_not_decrease_when_new_layer_changes_total() {
        let mut progress = PullProgressState::default();

        let first = progress.update_layer("layer-a".to_string(), 1000, 1000);
        let second = progress.update_layer("layer-b".to_string(), 10, 1000);

        assert_eq!(first, Some(50.0));
        assert_eq!(second, first);
    }

    #[test]
    fn progress_never_reaches_complete_before_success_event() {
        let mut progress = PullProgressState::default();

        let value = progress.update_layer("layer-a".to_string(), 2000, 1000);

        assert_eq!(value, Some(99.0));
    }
}
