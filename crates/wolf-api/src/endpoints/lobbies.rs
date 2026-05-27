use serde::Serialize;

use crate::{ApiError, WolfApi, types};

pub type Lobby = types::RflReflectorWolfCoreEventsLobbyReflType;
pub type LobbyListResponse = types::WolfApiLobbiesResponse;

#[derive(Clone, Copy, Debug)]
pub struct Lobbies<'api> {
    api: &'api WolfApi,
}

impl<'api> Lobbies<'api> {
    pub(crate) fn new(api: &'api WolfApi) -> Self {
        Self { api }
    }

    pub async fn list(&self) -> Result<LobbyListResponse, ApiError> {
        self.api.get_json("/api/v1/lobbies").await
    }

    pub async fn create(
        &self,
        lobby: &types::WolfApiCreateLobbyRequest,
    ) -> Result<types::WolfApiLobbyCreateResponse, ApiError> {
        self.api.post_json("/api/v1/lobbies/create", lobby).await
    }

    pub async fn join(
        &self,
        lobby_id: String,
        moonlight_session_id: String,
        pin: Option<Vec<i64>>,
    ) -> Result<types::WolfApiGenericSuccessResponse, ApiError> {
        self.api
            .post_json(
                "/api/v1/lobbies/join",
                &JoinLobbyRequest {
                    lobby_id,
                    moonlight_session_id,
                    pin,
                },
            )
            .await
    }

    pub async fn stop(
        &self,
        lobby_id: String,
        pin: Option<Vec<i64>>,
    ) -> Result<types::WolfApiGenericSuccessResponse, ApiError> {
        self.api
            .post_json(
                "/api/v1/lobbies/stop",
                &types::WolfCoreEventsStopLobbyEvent { lobby_id, pin },
            )
            .await
    }
}

#[derive(Serialize)]
struct JoinLobbyRequest {
    lobby_id: String,
    moonlight_session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pin: Option<Vec<i64>>,
}
