use crate::{ApiError, WolfApi, types};

pub type App = types::App;
pub type AppListResponse = types::AppListResponse;
pub type Profile = types::Profile;
pub type ProfileListResponse = types::ProfileListResponse;

#[derive(Clone, Copy, Debug)]
pub struct Profiles<'api> {
    api: &'api WolfApi,
}

impl<'api> Profiles<'api> {
    pub(crate) fn new(api: &'api WolfApi) -> Self {
        Self { api }
    }

    pub async fn list(&self) -> Result<ProfileListResponse, ApiError> {
        self.api.get_json("/api/v1/profiles").await
    }

    pub async fn apps(&self, profile_id: &str) -> Result<AppListResponse, ApiError> {
        let profiles = self.list().await?;
        let apps = profiles
            .profiles
            .into_iter()
            .find(|profile| profile.id == profile_id)
            .map(|profile| profile.apps)
            .unwrap_or_default();

        Ok(AppListResponse {
            success: profiles.success,
            apps,
        })
    }
}
