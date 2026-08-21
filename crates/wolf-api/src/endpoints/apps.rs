use crate::{ApiError, WolfApi, types};

pub type App = types::App;
pub type AppListResponse = types::AppListResponse;

#[derive(Clone, Copy, Debug)]
pub struct Apps<'api> {
    api: &'api WolfApi,
}

impl<'api> Apps<'api> {
    pub(crate) fn new(api: &'api WolfApi) -> Self {
        Self { api }
    }

    pub async fn list(&self) -> Result<AppListResponse, ApiError> {
        self.api.get_json("/api/v1/apps").await
    }
}
