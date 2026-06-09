use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum ClientBuildError {
    #[error("failed to build Wolf API HTTP client")]
    HttpClient(#[from] reqwest::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Wolf API request timed out")]
    Timeout,
    #[error("Wolf API transport error: {0}")]
    Transport(reqwest::Error),
    #[error("Wolf API returned HTTP {status}: {body}")]
    Status { status: StatusCode, body: String },
    #[error("Wolf API returned HTTP {status}: {message}")]
    Wolf { status: StatusCode, message: String },
    #[error("failed to decode Wolf API response: {0}")]
    Decode(reqwest::Error),
    #[error("failed to decode Wolf API event: {0}")]
    EventDecode(serde_json::Error),
}

impl ApiError {
    pub(crate) fn from_reqwest(error: reqwest::Error) -> Self {
        let is_timeout = error.is_timeout();
        let is_decode = error.is_decode();
        let error = error.without_url();

        if is_timeout {
            Self::Timeout
        } else if is_decode {
            Self::Decode(error)
        } else {
            Self::Transport(error)
        }
    }

    pub(crate) fn from_serde_json(error: serde_json::Error) -> Self {
        Self::EventDecode(error)
    }
}
