use serde::{Serialize, de::DeserializeOwned};

use crate::{endpoints, error::ApiError, types};

#[derive(Clone, Debug)]
pub struct WolfApi {
    base_url: String,
    http_client: reqwest::Client,
}

impl WolfApi {
    #[must_use]
    pub fn new(base_url: impl Into<String>, http_client: reqwest::Client) -> Self {
        Self {
            base_url: base_url.into(),
            http_client,
        }
    }

    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    #[must_use]
    pub fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    pub fn profiles(&self) -> endpoints::Profiles<'_> {
        endpoints::profiles::Profiles::new(self)
    }

    pub fn utils(&self) -> endpoints::Utils<'_> {
        endpoints::utils::Utils::new(self)
    }

    pub(crate) async fn get_json<T>(&self, path: &str) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
    {
        let response = self
            .http_client
            .get(self.url(path))
            .send()
            .await
            .map_err(ApiError::from_reqwest)?;

        decode_json(response).await
    }

    pub(crate) async fn get_bytes_with_query<T>(
        &self,
        path: &str,
        query: &T,
    ) -> Result<Vec<u8>, ApiError>
    where
        T: Serialize + ?Sized,
    {
        let response = self
            .http_client
            .get(self.url(path))
            .query(query)
            .send()
            .await
            .map_err(ApiError::from_reqwest)?;

        decode_bytes(response).await
    }

    fn url(&self, path: &str) -> String {
        let mut url = String::with_capacity(self.base_url.len() + path.len());
        url.push_str(&self.base_url);
        url.push_str(path);
        url
    }
}

pub(crate) async fn decode_json<T>(response: reqwest::Response) -> Result<T, ApiError>
where
    T: DeserializeOwned,
{
    let status = response.status();

    if status.is_success() {
        return response.json().await.map_err(ApiError::from_reqwest);
    }

    let body = response.text().await.map_err(ApiError::from_reqwest)?;
    if let Ok(error) = serde_json::from_str::<types::WolfApiGenericErrorResponse>(&body) {
        return Err(ApiError::Wolf {
            status,
            message: error.error,
        });
    }

    Err(ApiError::Status { status, body })
}

pub(crate) async fn decode_bytes(response: reqwest::Response) -> Result<Vec<u8>, ApiError> {
    let status = response.status();

    if status.is_success() {
        return response
            .bytes()
            .await
            .map(|bytes| bytes.to_vec())
            .map_err(ApiError::from_reqwest);
    }

    let body = response.text().await.map_err(ApiError::from_reqwest)?;
    if let Ok(error) = serde_json::from_str::<types::WolfApiGenericErrorResponse>(&body) {
        return Err(ApiError::Wolf {
            status,
            message: error.error,
        });
    }

    Err(ApiError::Status { status, body })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_wrapper_keeps_configured_base_url() {
        let http_client = reqwest::Client::new();
        let client = WolfApi::new("http://wolf.test", http_client);

        assert_eq!(client.base_url(), "http://wolf.test");
    }

    #[test]
    fn endpoint_urls_join_base_and_path() {
        let http_client = reqwest::Client::new();
        let client = WolfApi::new("http://wolf.test", http_client);

        assert_eq!(
            client.url("/api/v1/profiles"),
            "http://wolf.test/api/v1/profiles"
        );
    }
}
