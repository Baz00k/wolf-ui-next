use dioxus::prelude::*;

use crate::components::primitives::{ToastOptions, use_toasts};
use crate::domain::settings::{auto_update_enabled, run_startup_auto_update};

#[component]
pub fn StartupAutoUpdate() -> Element {
    let mut toasts = use_toasts();

    use_hook(move || {
        if !auto_update_enabled() {
            return;
        }

        spawn(async move {
            match run_startup_auto_update().await {
                Ok(true) => toasts.show(
                    "A new Wolf UI image was downloaded. Restart to apply the update.",
                    ToastOptions::default().persistent().dismissible(),
                ),
                Ok(false) => {}
                Err(error) => tracing::warn!(error, "startup auto-update failed"),
            }
        });
    });

    rsx! {}
}
