//! Generated Wolf API data bindings and a configured client for the local Wolf daemon.
//!
//! The Rust data model in [`types`] is generated directly from Wolf's OpenAPI 3.1 schema at build time.
//! The hand-written [`WolfApi`] facade is the stable API application code should use.

mod client;
mod config;
mod endpoints;
mod error;
mod transport;

pub mod types {
    #![allow(clippy::all)]
    #![allow(missing_docs)]

    include!(concat!(env!("OUT_DIR"), "/types.rs"));
}

pub use endpoints::profiles;

pub use client::WolfApi;
pub use config::{
    ClientConfig, DEFAULT_BASE_URL, DEFAULT_CONNECT_TIMEOUT, DEFAULT_MAX_RETRIES,
    DEFAULT_READ_TIMEOUT, DEFAULT_REQUEST_TIMEOUT, DEFAULT_UNIX_SOCKET_PATH,
};
pub use error::{ApiError, ClientBuildError};
pub use transport::reqwest_client;

pub fn client() -> Result<WolfApi, ClientBuildError> {
    client_with_config(ClientConfig::default())
}

pub fn client_with_config(config: ClientConfig) -> Result<WolfApi, ClientBuildError> {
    let http_client = reqwest_client(&config)?;
    Ok(WolfApi::new(config.into_base_url(), http_client))
}

#[cfg(test)]
mod tests {
    use super::types;

    #[test]
    fn generated_types_round_trip_wolf_json_shape() {
        let response = types::WolfApiAppListResponse {
            success: true,
            apps: Vec::new(),
        };

        let json = serde_json::to_string(&response).expect("serialize response");
        let parsed: types::WolfApiAppListResponse =
            serde_json::from_str(&json).expect("deserialize response");

        assert_eq!(parsed.success, response.success);
        assert_eq!(parsed.apps.len(), response.apps.len());
    }
}
