use serde::de::DeserializeOwned;

use crate::{ApiError, WolfApi, types};

#[derive(Clone, Debug, PartialEq)]
pub enum WolfEvent {
    LobbyCreated(Box<types::RflReflectorWolfCoreEventsLobbyReflType>),
    LobbyStopped(String),
    Other(String),
}

#[derive(Clone, Copy, Debug)]
pub struct Events<'api> {
    api: &'api WolfApi,
}

impl<'api> Events<'api> {
    pub(crate) fn new(api: &'api WolfApi) -> Self {
        Self { api }
    }

    pub async fn connect(&self) -> Result<reqwest::Response, ApiError> {
        self.api
            .http_client()
            .get(self.api.url("/api/v1/events"))
            .send()
            .await
            .map_err(ApiError::from_reqwest)
    }
}

pub fn parse_event(event: String, data: String) -> Result<WolfEvent, ApiError> {
    match event.as_str() {
        "wolf::core::events::CreateLobbyEvent" => {
            parse_json(&data).map(|event| WolfEvent::LobbyCreated(Box::new(event)))
        }
        "wolf::core::events::StopLobbyEvent" => {
            parse_json::<types::WolfCoreEventsStopLobbyEvent>(&data)
                .map(|event| WolfEvent::LobbyStopped(event.lobby_id))
        }
        _ => Ok(WolfEvent::Other(event)),
    }
}

fn parse_json<T>(data: &str) -> Result<T, ApiError>
where
    T: DeserializeOwned,
{
    serde_json::from_str(data).map_err(ApiError::from_serde_json)
}
