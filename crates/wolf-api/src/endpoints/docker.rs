use reqwest::StatusCode;

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
}
