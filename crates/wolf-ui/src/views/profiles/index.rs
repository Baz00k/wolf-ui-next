use dioxus::prelude::*;

use crate::api::ApiContext;

#[component]
pub fn Profiles() -> Element {
    let profiles = use_resource(move || async move {
        ApiContext::consume()
            .profiles()
            .list()
            .await
            .map_err(|error| error.to_string())
    });

    // Original: src/Scenes/Main/Body/Users/UserList.tscn
    // Default view when starting moonlight session.
    // Shows a list of profiles and lobbies (if any lobbies are available)
    // Original lobbies: src/Scenes/Main/Body/Lobby/LobbiesContainer.tscn
    // Profiles and lobbies can require pin code authentication.
    rsx! {
        match &*profiles.read_unchecked() {
            Some(Ok(_)) => rsx! {
               p { "Loaded profiles" }
            },
            Some(Err(error)) => rsx! {
                p { "Error loading profiles: {error}" }
            },
            None => rsx! {
                p { "Loading..." }
            },
        }
    }
}
