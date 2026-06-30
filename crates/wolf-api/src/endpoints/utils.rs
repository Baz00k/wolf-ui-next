use crate::{ApiError, WolfApi};

#[derive(Clone, Copy, Debug)]
pub struct Utils<'api> {
    api: &'api WolfApi,
}

impl<'api> Utils<'api> {
    pub(crate) fn new(api: &'api WolfApi) -> Self {
        Self { api }
    }

    pub async fn icon(&self, icon_path: &str) -> Result<Vec<u8>, ApiError> {
        self.api
            .get_bytes_with_raw_query("/api/v1/utils/get-icon", &format!("icon_path={icon_path}"))
            .await
    }
}
