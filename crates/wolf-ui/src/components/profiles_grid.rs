use dioxus::prelude::*;
use wolf_api::lobbies::Lobby;
use wolf_api::profiles::Profile;

use crate::Route;
use crate::api::ApiContext;
use crate::components::lobby_card::LobbyCardData;
use crate::components::primitives::{StatusAlert, StatusAlertVariant};
use crate::components::{LobbyCard, ProfileCard, ProfileCardData, ProfileCardSkeleton};
use crate::domain::image_loader::load_image_src;
use crate::domain::profiles::ProfilesState;

const CARD_SKELETON_COUNT: usize = 3;
const PROFILE_GRID_CLASS: &str = "mx-auto grid w-full max-w-[min(100%,calc(22rem*5+2rem*4))] grid-cols-[repeat(auto-fit,minmax(min(100%,14rem),18rem))] justify-center gap-4 p-2 sm:grid-cols-[repeat(auto-fit,minmax(16rem,20rem))] sm:gap-5 sm:p-3 xl:grid-cols-[repeat(auto-fit,minmax(18rem,22rem))] xl:gap-6 lg:p-4 2xl:gap-8 2xl:p-5";

#[component]
pub fn ProfilesLoading() -> Element {
    rsx! {
        div { class: "my-auto flex min-h-full w-full items-center justify-center py-6 sm:py-8 lg:py-10",
            div { class: PROFILE_GRID_CLASS,
                for _ in 0..CARD_SKELETON_COUNT {
                    ProfileCardSkeleton {}
                }
            }
        }
    }
}

#[component]
pub fn ProfilesContent(
    state: ProfilesState,
    lobbies: Vec<Lobby>,
    on_lobby_click: EventHandler<Lobby>,
) -> Element {
    let response = state.profiles;
    if !response.success {
        return rsx! {
            StatusAlert {
                title: Some("Profiles unavailable".to_string()),
                message: "Wolf returned an unsuccessful profiles response. Try again once the service is ready.".to_string(),
                variant: StatusAlertVariant::Error,
            }
        };
    }

    if response.profiles.is_empty() && lobbies.is_empty() {
        return rsx! {
            StatusAlert {
                title: Some("No profiles found".to_string()),
                message: "Create a Wolf profile before launching a Moonlight session.".to_string(),
                variant: StatusAlertVariant::Info,
            }
        };
    }

    rsx! {
        div { class: "my-auto flex min-h-full w-full items-center justify-center py-6 sm:py-8 lg:py-10",
            div { class: PROFILE_GRID_CLASS,
                for (index, profile) in response.profiles.iter().cloned().enumerate() {
                    div {
                        ProfileCardLoader {
                            profile,
                            autofocus: index == 0,
                        }
                    }
                }
                for (index, lobby) in lobbies.iter().cloned().enumerate() {
                    div { key: "{lobby.id}",
                        LobbyCardLoader {
                            lobby: lobby.clone(),
                            profiles: response.profiles.clone(),
                            autofocus: response.profiles.is_empty() && index == 0,
                            onclick: move |_| on_lobby_click.call(lobby.clone()),
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ProfileCardLoader(profile: Profile, autofocus: bool) -> Element {
    let profile_id = profile.id.clone();
    let icon_path = profile.icon_png_path.trim().to_string();
    let avatar = use_resource(move || {
        let icon_path = icon_path.clone();

        async move { load_image_src(&ApiContext::consume(), &icon_path).await }
    });

    let mut profile = profile_card_data(profile);
    profile.avatar_src = avatar.read().clone().flatten();

    rsx! {
        ProfileCard {
            profile,
            autofocus,
            to: Route::ProfileApps { profile_id }.to_string(),
        }
    }
}

#[component]
fn LobbyCardLoader(
    lobby: Lobby,
    profiles: Vec<Profile>,
    autofocus: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        LobbyCard {
            lobby: lobby_card_data(lobby, &profiles),
            autofocus,
            onclick: move |event| onclick.call(event),
        }
    }
}

fn profile_card_data(profile: Profile) -> ProfileCardData {
    ProfileCardData {
        id: profile.id,
        name: profile.name,
        avatar_src: None,
        is_pin_locked: profile.pin.is_some(),
    }
}

fn lobby_card_data(lobby: Lobby, profiles: &[Profile]) -> LobbyCardData {
    let creator = profiles
        .iter()
        .find(|profile| profile.id == lobby.started_by_profile_id)
        .map(|profile| profile.name.as_str());

    LobbyCardData {
        id: lobby.id,
        name: creator.map(lobby_name).unwrap_or(lobby.name),
        is_pin_locked: lobby.pin_required,
        connected_users: lobby.connected_sessions.len(),
    }
}

fn lobby_name(creator: &str) -> String {
    if creator.ends_with('s') {
        format!("{creator}' lobby")
    } else {
        format!("{creator}'s lobby")
    }
}
