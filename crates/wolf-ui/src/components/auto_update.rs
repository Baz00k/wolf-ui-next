use dioxus::prelude::*;

use crate::domain::image_pulls::{PullKind, use_image_pulls};
use crate::domain::settings::{WOLF_UI_IMAGE_TITLE, auto_update_enabled, load_wolf_ui_image_state};

#[component]
pub fn StartupAutoUpdate() -> Element {
    let pulls = use_image_pulls();

    use_hook(move || {
        if !auto_update_enabled() {
            return;
        }

        spawn(async move {
            match load_wolf_ui_image_state().await {
                Ok(state) => pulls.start(
                    state.source,
                    PullKind::StartupUpdate,
                    WOLF_UI_IMAGE_TITLE.to_string(),
                ),
                Err(error) => tracing::warn!(error, "startup auto-update failed"),
            }
        });
    });

    rsx! {}
}
