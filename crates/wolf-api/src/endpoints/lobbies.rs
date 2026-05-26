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
}
