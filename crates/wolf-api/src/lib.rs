//! Generated Wolf API data bindings and a configured client for the local Wolf daemon.
//!
//! The Rust data model in [`types`] is generated from Wolf's vendored OpenAPI 3.1 schema.
//! The hand-written [`WolfApi`] facade is the stable API application code should use.

mod client;
mod config;
mod endpoints;
mod error;
mod transport;

pub mod types;

pub use endpoints::{apps, docker, events, lobbies, profiles, sessions};

pub use client::WolfApi;
pub use config::{
    ApiTransport, ClientConfig, DEFAULT_BASE_URL, DEFAULT_CONNECT_TIMEOUT, DEFAULT_MAX_RETRIES,
    DEFAULT_READ_TIMEOUT, DEFAULT_REQUEST_TIMEOUT, DEFAULT_UNIX_SOCKET_PATH, WOLF_API_BASE_URL_ENV,
    WOLF_SOCKET_PATH_ENV,
};
pub use error::{ApiError, ClientBuildError};
pub use transport::reqwest_client;

pub fn client() -> Result<WolfApi, ClientBuildError> {
    client_with_config(ClientConfig::default())
}

pub fn client_with_config(config: ClientConfig) -> Result<WolfApi, ClientBuildError> {
    let transport = config.transport_ref();
    match transport {
        ApiTransport::UnixSocket => tracing::debug!(
            "Wolf API client connecting via Unix socket: {} (base URL: {})",
            config.unix_socket_path_ref().display(),
            config.base_url_ref()
        ),
        ApiTransport::Tcp => tracing::debug!(
            "Wolf API client connecting via TCP: {}",
            config.base_url_ref()
        ),
    }

    let http_client = reqwest_client(&config)?;
    let unix_socket_path =
        (transport == ApiTransport::UnixSocket).then(|| config.unix_socket_path_ref().to_owned());
    Ok(WolfApi::with_transport(
        config.into_base_url(),
        http_client,
        transport,
        unix_socket_path,
    ))
}
