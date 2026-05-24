use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const DEFAULT_BASE_URL: &str = "http://wolf.local";
pub const DEFAULT_UNIX_SOCKET_PATH: &str = "/var/run/wolf/wolf.sock";
pub const WOLF_API_BASE_URL_ENV: &str = "WOLF_API_BASE_URL";
pub const WOLF_SOCKET_PATH_ENV: &str = "WOLF_SOCKET_PATH";
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
pub const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(10);
pub const DEFAULT_MAX_RETRIES: u32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApiTransport {
    UnixSocket,
    Tcp,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientConfig {
    base_url: String,
    unix_socket_path: PathBuf,
    transport: ApiTransport,
    connect_timeout: Duration,
    request_timeout: Duration,
    read_timeout: Duration,
    max_retries: u32,
}

impl Default for ClientConfig {
    fn default() -> Self {
        let base_url_env = std::env::var(WOLF_API_BASE_URL_ENV).ok();

        Self {
            transport: transport_from_base_url_env(base_url_env.as_deref()),
            base_url: base_url_from_env(base_url_env.as_deref()),
            unix_socket_path: socket_path_from_env(std::env::var_os(WOLF_SOCKET_PATH_ENV)),
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
            read_timeout: DEFAULT_READ_TIMEOUT,
            max_retries: DEFAULT_MAX_RETRIES,
        }
    }
}

impl ClientConfig {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    #[must_use]
    pub fn transport(mut self, transport: ApiTransport) -> Self {
        self.transport = transport;
        self
    }

    #[must_use]
    pub fn unix_socket_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.unix_socket_path = path.into();
        self
    }

    #[must_use]
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    #[must_use]
    pub fn request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    #[must_use]
    pub fn read_timeout(mut self, timeout: Duration) -> Self {
        self.read_timeout = timeout;
        self
    }

    #[must_use]
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    #[must_use]
    pub fn base_url_ref(&self) -> &str {
        &self.base_url
    }

    #[must_use]
    pub(crate) fn into_base_url(self) -> String {
        self.base_url
    }

    #[must_use]
    pub fn unix_socket_path_ref(&self) -> &Path {
        &self.unix_socket_path
    }

    #[must_use]
    pub fn transport_ref(&self) -> ApiTransport {
        self.transport
    }

    #[must_use]
    pub fn connect_timeout_ref(&self) -> Duration {
        self.connect_timeout
    }

    #[must_use]
    pub fn request_timeout_ref(&self) -> Duration {
        self.request_timeout
    }

    #[must_use]
    pub fn read_timeout_ref(&self) -> Duration {
        self.read_timeout
    }

    #[must_use]
    pub fn max_retries_ref(&self) -> u32 {
        self.max_retries
    }
}

fn socket_path_from_env(value: Option<OsString>) -> PathBuf {
    value.map_or_else(|| PathBuf::from(DEFAULT_UNIX_SOCKET_PATH), PathBuf::from)
}

fn base_url_from_env(value: Option<&str>) -> String {
    value
        .filter(|base_url| !base_url.is_empty())
        .map_or_else(|| DEFAULT_BASE_URL.to_owned(), ToOwned::to_owned)
}

fn transport_from_base_url_env(value: Option<&str>) -> ApiTransport {
    if value.is_some_and(|base_url| !base_url.is_empty()) {
        ApiTransport::Tcp
    } else {
        ApiTransport::UnixSocket
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_targets_local_wolf_socket() {
        let config = ClientConfig::default();

        assert_eq!(config.base_url_ref(), DEFAULT_BASE_URL);
        assert_eq!(config.transport_ref(), ApiTransport::UnixSocket);
        assert_eq!(
            config.unix_socket_path_ref(),
            Path::new(DEFAULT_UNIX_SOCKET_PATH)
        );
        assert_eq!(config.connect_timeout_ref(), DEFAULT_CONNECT_TIMEOUT);
        assert_eq!(config.request_timeout_ref(), DEFAULT_REQUEST_TIMEOUT);
        assert_eq!(config.read_timeout_ref(), DEFAULT_READ_TIMEOUT);
        assert_eq!(config.max_retries_ref(), DEFAULT_MAX_RETRIES);
    }

    #[test]
    fn config_builder_overrides_connection_policy() {
        let config = ClientConfig::new()
            .base_url("http://wolf.test")
            .transport(ApiTransport::Tcp)
            .unix_socket_path("/tmp/wolf.sock")
            .connect_timeout(Duration::from_millis(250))
            .request_timeout(Duration::from_secs(3))
            .read_timeout(Duration::from_secs(4))
            .max_retries(1);

        assert_eq!(config.base_url_ref(), "http://wolf.test");
        assert_eq!(config.transport_ref(), ApiTransport::Tcp);
        assert_eq!(config.unix_socket_path_ref(), Path::new("/tmp/wolf.sock"));
        assert_eq!(config.connect_timeout_ref(), Duration::from_millis(250));
        assert_eq!(config.request_timeout_ref(), Duration::from_secs(3));
        assert_eq!(config.read_timeout_ref(), Duration::from_secs(4));
        assert_eq!(config.max_retries_ref(), 1);
    }

    #[test]
    fn socket_path_uses_env_value_when_present() {
        let path = socket_path_from_env(Some(OsString::from("/tmp/custom-wolf.sock")));

        assert_eq!(path, Path::new("/tmp/custom-wolf.sock"));
    }

    #[test]
    fn socket_path_falls_back_to_default_when_env_missing() {
        let path = socket_path_from_env(None);

        assert_eq!(path, Path::new(DEFAULT_UNIX_SOCKET_PATH));
    }

    #[test]
    fn base_url_uses_env_value_when_present() {
        let base_url = base_url_from_env(Some("http://localhost:8080"));

        assert_eq!(base_url, "http://localhost:8080");
    }

    #[test]
    fn base_url_falls_back_to_default_when_env_missing_or_empty() {
        assert_eq!(base_url_from_env(None), DEFAULT_BASE_URL);
        assert_eq!(base_url_from_env(Some("")), DEFAULT_BASE_URL);
    }

    #[test]
    fn tcp_transport_is_selected_when_base_url_env_is_present() {
        assert_eq!(
            transport_from_base_url_env(Some("http://localhost:8080")),
            ApiTransport::Tcp
        );
    }

    #[test]
    fn unix_socket_transport_is_selected_when_base_url_env_is_missing_or_empty() {
        assert_eq!(transport_from_base_url_env(None), ApiTransport::UnixSocket);
        assert_eq!(
            transport_from_base_url_env(Some("")),
            ApiTransport::UnixSocket
        );
    }
}
