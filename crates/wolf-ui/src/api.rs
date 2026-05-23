use dioxus::{core::consume_context, hooks::use_context_provider};

#[derive(Clone)]
pub struct ApiContext {
    api: wolf_api::WolfApi,
}

impl ApiContext {
    pub fn provide() {
        use_context_provider(|| Self {
            api: wolf_api::client().expect("build Wolf API client"),
        });
    }

    pub fn consume() -> wolf_api::WolfApi {
        consume_context::<Self>().api
    }
}
