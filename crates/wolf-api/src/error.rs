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
}

impl ApiError {
    pub(crate) fn from_reqwest(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            Self::Timeout
        } else if error.is_decode() {
            Self::Decode(error)
        } else {
            Self::Transport(error)
        }
    }
}
