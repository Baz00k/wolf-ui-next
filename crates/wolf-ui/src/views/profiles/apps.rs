use dioxus::prelude::*;

use crate::api::ApiContext;
use crate::input::{UiAction, use_ui_action};

#[component]
pub fn ProfileApps(profile_id: String) -> Element {
    let navigator = use_navigator();
    let back_action = use_ui_action(UiAction::Cancel, "Back", move || {
        navigator.go_back();
    });

    let apps = use_resource(move || {
        let profile_id = profile_id.clone();

        async move {
            ApiContext::consume()
                .profiles()
                .apps(&profile_id)
                .await
                .map_err(|error| error.to_string())
        }
    });

    // Original: src/Scenes/Main/Body/Apps/AppList.tscn
    // Shows a list of apps for the selected profile.
    rsx! {
        section {
            class: "flex min-h-screen flex-col bg-background px-8 py-12 text-foreground sm:px-12 lg:px-20",
            "data-focus-scope": "true",
            "data-scope-actions": back_action,
            div { class: "flex flex-1 items-center justify-center",
                match &*apps.read_unchecked() {
                    Some(Ok(_)) => rsx! {
                        "Loaded apps!"
                    },
                    Some(Err(error)) => rsx! { "Error loading apps: {error}"  },
                    None => rsx! { "Loading apps..." },
                }
            }
        }
    }
}
