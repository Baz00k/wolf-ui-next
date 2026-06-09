use crate::config::{ApiTransport, ClientConfig};

pub fn reqwest_client(config: &ClientConfig) -> Result<reqwest::Client, reqwest::Error> {
    let builder = reqwest::ClientBuilder::new()
        .connect_timeout(config.connect_timeout_ref())
        .timeout(config.request_timeout_ref())
        .read_timeout(config.read_timeout_ref())
        .retry(retry_policy(config));

    match config.transport_ref() {
        ApiTransport::UnixSocket => builder.unix_socket(config.unix_socket_path_ref()).build(),
        ApiTransport::Tcp => builder.build(),
    }
}

fn retry_policy(config: &ClientConfig) -> reqwest::retry::Builder {
    reqwest::retry::for_host(host_for_retry_scope(config.base_url_ref()))
        .max_retries_per_request(config.max_retries_ref())
        .classify_fn(|req_rep| match (req_rep.method(), req_rep.status()) {
            (&reqwest::Method::GET, Some(status)) if status.is_server_error() => {
                req_rep.retryable()
            }
            (&reqwest::Method::GET, None) => req_rep.retryable(),
            _ => req_rep.success(),
        })
}

fn host_for_retry_scope(base_url: &str) -> String {
    reqwest::Url::parse(base_url)
        .ok()
        .and_then(|url| url.host_str().map(str::to_owned))
        .unwrap_or_default()
}
