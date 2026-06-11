use serde::{Serialize, de::DeserializeOwned};
use std::time::{Duration, Instant};

use crate::{endpoints, error::ApiError, types};

const STREAM_REQUEST_TIMEOUT: Duration = Duration::from_hours(12);

#[derive(Clone, Copy, Debug)]
pub(crate) struct RequestContext<'api> {
    method: &'static str,
    path: &'api str,
    base_url: &'api str,
}

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

    pub fn profiles(&self) -> endpoints::Profiles<'_> {
        endpoints::profiles::Profiles::new(self)
    }

    pub fn lobbies(&self) -> endpoints::Lobbies<'_> {
        endpoints::lobbies::Lobbies::new(self)
    }

    pub fn sessions(&self) -> endpoints::Sessions<'_> {
        endpoints::sessions::Sessions::new(self)
    }

    pub fn docker(&self) -> endpoints::Docker<'_> {
        endpoints::docker::Docker::new(self)
    }

    pub fn events(&self) -> endpoints::Events<'_> {
        endpoints::events::Events::new(self)
    }

    pub fn utils(&self) -> endpoints::Utils<'_> {
        endpoints::utils::Utils::new(self)
    }

    pub(crate) async fn get_json<T>(&self, path: &str) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
    {
        let context = self.request_context("GET", path);
        let response = self
            .send(context, self.http_client.get(self.url(path)))
            .await?;

        self.decode_json(context, response).await
    }

    pub(crate) async fn post_json<B, T>(&self, path: &str, body: &B) -> Result<T, ApiError>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let context = self.request_context("POST", path);
        let response = self
            .send(context, self.http_client.post(self.url(path)).json(body))
            .await?;

        self.decode_json(context, response).await
    }

    pub(crate) async fn post_stream<B>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<reqwest::Response, ApiError>
    where
        B: Serialize + ?Sized,
    {
        let context = self.request_context("POST", path);
        let response = self
            .send(
                context,
                self.http_client
                    .post(self.url(path))
                    .json(body)
                    .timeout(STREAM_REQUEST_TIMEOUT),
            )
            .await?;

        if response.status().is_success() {
            return Ok(response);
        }

        let error = response_error(response).await;
        self.log_api_error(context, &error, "Wolf API response failed");
        Err(error)
    }

    pub(crate) async fn get_bytes_with_query<T>(
        &self,
        path: &str,
        query: &T,
    ) -> Result<Vec<u8>, ApiError>
    where
        T: Serialize + ?Sized,
    {
        let context = self.request_context("GET", path);
        let response = self
            .send(context, self.http_client.get(self.url(path)).query(query))
            .await?;

        self.decode_bytes(context, response).await
    }

    pub(crate) async fn get_stream_response(
        &self,
        path: &str,
    ) -> Result<reqwest::Response, ApiError> {
        let context = self.request_context("GET", path);
        self.send(
            context,
            self.http_client
                .get(self.url(path))
                .timeout(STREAM_REQUEST_TIMEOUT),
        )
        .await
    }

    pub(crate) async fn get_response_with_query<T>(
        &self,
        path: &str,
        query: &T,
    ) -> Result<reqwest::Response, ApiError>
    where
        T: Serialize + ?Sized,
    {
        let context = self.request_context("GET", path);
        self.send(context, self.http_client.get(self.url(path)).query(query))
            .await
    }

    pub(crate) fn url(&self, path: &str) -> String {
        let mut url = String::with_capacity(self.base_url.len() + path.len());
        url.push_str(&self.base_url);
        url.push_str(path);
        url
    }

    pub(crate) fn request_context<'api>(
        &'api self,
        method: &'static str,
        path: &'api str,
    ) -> RequestContext<'api> {
        RequestContext {
            method,
            path,
            base_url: &self.base_url,
        }
    }

    pub(crate) fn log_api_error(
        &self,
        context: RequestContext<'_>,
        error: &ApiError,
        message: &'static str,
    ) {
        tracing::error!(
            method = context.method,
            path = context.path,
            base_url = context.base_url,
            %error,
            message
        );
    }

    async fn send(
        &self,
        context: RequestContext<'_>,
        request: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, ApiError> {
        let started_at = Instant::now();
        let response = request.send().await.map_err(ApiError::from_reqwest);
        let elapsed_ms = started_at.elapsed().as_millis();

        match &response {
            Ok(response) => tracing::debug!(
                method = context.method,
                path = context.path,
                base_url = context.base_url,
                status = %response.status(),
                elapsed_ms,
                "Wolf API response received"
            ),
            Err(error) => tracing::error!(
                method = context.method,
                path = context.path,
                base_url = context.base_url,
                elapsed_ms,
                %error,
                "Wolf API request failed"
            ),
        }

        response
    }

    async fn decode_json<T>(
        &self,
        context: RequestContext<'_>,
        response: reqwest::Response,
    ) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
    {
        let result = decode_json(response).await;
        result.inspect_err(|error| self.log_api_error(context, error, "Wolf API response failed"))
    }

    async fn decode_bytes(
        &self,
        context: RequestContext<'_>,
        response: reqwest::Response,
    ) -> Result<Vec<u8>, ApiError> {
        let result = decode_bytes(response).await;
        result.inspect_err(|error| self.log_api_error(context, error, "Wolf API response failed"))
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

pub(crate) async fn response_error(response: reqwest::Response) -> ApiError {
    let status = response.status();
    let body = match response.text().await {
        Ok(body) => body,
        Err(error) => return ApiError::from_reqwest(error),
    };

    if let Ok(error) = serde_json::from_str::<types::WolfApiGenericErrorResponse>(&body) {
        return ApiError::Wolf {
            status,
            message: error.error,
        };
    }

    ApiError::Status { status, body }
}
