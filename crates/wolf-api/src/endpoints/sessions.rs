use serde::Serialize;

use crate::{ApiError, WolfApi, types};

pub type Session = types::RflReflectorWolfCoreEventsStreamSessionReflType;
pub type SessionListResponse = types::WolfApiStreamSessionListResponse;
pub type StopSessionResponse = types::WolfApiGenericSuccessResponse;

#[derive(Clone, Copy, Debug)]
pub struct Sessions<'api> {
    api: &'api WolfApi,
}

impl<'api> Sessions<'api> {
    pub(crate) fn new(api: &'api WolfApi) -> Self {
        Self { api }
    }

    pub async fn list(&self) -> Result<SessionListResponse, ApiError> {
        self.api.get_json("/api/v1/sessions").await
    }

    pub async fn stop(&self, session_id: &str) -> Result<StopSessionResponse, ApiError> {
        self.api
            .post_json("/api/v1/sessions/stop", &StopSessionRequest { session_id })
            .await
    }

    pub async fn by_client_id(&self, client_id: &str) -> Result<Option<Session>, ApiError> {
        Ok(self
            .list()
            .await?
            .sessions
            .into_iter()
            .find(|session| session.client_id.as_deref() == Some(client_id)))
    }
}

#[derive(Serialize)]
struct StopSessionRequest<'request> {
    session_id: &'request str,
}
