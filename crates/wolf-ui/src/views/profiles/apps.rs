use dioxus::prelude::*;

use crate::api::ApiContext;

#[component]
pub fn ProfileApps(profile_id: String) -> Element {
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
        match &*apps.read_unchecked() {
            Some(Ok(_)) => rsx! {
                "Loaded apps!"
            },
            Some(Err(error)) => rsx! { "Error loading apps: {error}"  },
            None => rsx! { "Loading apps..." },
        }
    }
}
