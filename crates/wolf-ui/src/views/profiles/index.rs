use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons::HiCog;
use wolf_api::lobbies::Lobby;

use crate::Route;
use crate::components::primitives::{
    Button, ButtonSize, ButtonVariant, StatusAlert, StatusAlertVariant,
};
use crate::components::{
    LobbyActionDialog, ProfilesContent, ProfilesLoading, SessionShutdownControl,
};
use crate::domain::app_actions::{join_lobby, stop_lobby};
use crate::domain::profiles::{ProfilesState, listen_for_lobby_events, load_profiles_state};
use crate::domain::settings::settings_enabled;

#[component]
pub fn Profiles() -> Element {
    let mut lobbies = use_signal(Vec::<Lobby>::new);
    let mut action_lobby = use_signal(|| None::<Lobby>);
    let mut profiles = use_resource(move || async move { load_profiles_state().await });
    let join_runner = use_action(move |lobby_id: String, pin: Option<Vec<i64>>| async move {
        join_lobby(lobby_id, pin).await
    });
    let stop_runner = use_action(move |lobby_id: String, pin: Option<Vec<i64>>| async move {
        stop_lobby(lobby_id, pin).await
    });

    use_effect(move || {
        if let Some(Ok(state)) = &*profiles.read() {
            lobbies.set(state.lobbies.clone());
        }
    });

    use_effect(move || {
        spawn(async move {
            listen_for_lobby_events(lobbies).await;
        });
    });

    rsx! {
        div { class: "h-full min-h-0",
            section { class: "relative flex h-full min-h-0 flex-col px-6 pt-8 sm:px-16 sm:pt-12",
                ProfilesHeader {}
                div {
                    class: "flex min-h-0 flex-1 overflow-y-auto overflow-x-hidden scroll-py-8 scrollbar-hide sm:scroll-py-12",
                    "data-focus-scope": "true",
                    "data-focus-region": "main",
                    match &*profiles.read_unchecked() {
                        Some(Ok(state)) => rsx! {
                            ProfilesContent {
                                state: state.clone(),
                                lobbies: lobbies(),
                                on_lobby_click: move |lobby: Lobby| action_lobby.set(Some(lobby)),
                            }
                        },
                        Some(Err(message)) => rsx! {
                            div { class: "w-full h-full grid place-items-center",
                                StatusAlert {
                                    title: Some("Profiles unavailable".to_string()),
                                    message: message.clone(),
                                    variant: StatusAlertVariant::Error,
                                    Button { size: ButtonSize::Lg, onclick: move |_| profiles.restart(), "Retry" }
                                }
                            }
                        },
                        None => rsx! {
                            ProfilesLoading {}
                        },
                    }
                }
            }
            if let Some(lobby) = action_lobby() {
                LobbyActionDialog {
                    owner_pin: lobby_owner_pin(&profiles, &lobby),
                    lobby,
                    join_runner,
                    stop_runner,
                    onstopped: move |lobby_id: String| {
                        lobbies.with_mut(|items| items.retain(|item| item.id != lobby_id));
                    },
                    onclose: move |_| action_lobby.set(None),
                }
            }
        }
    }
}

fn lobby_owner_pin(
    profiles: &Resource<Result<ProfilesState, String>>,
    lobby: &Lobby,
) -> Option<Vec<i64>> {
    let binding = profiles.read();
    let Some(Ok(state)) = &*binding else {
        return None;
    };
    state
        .profiles
        .profiles
        .iter()
        .find(|profile| profile.id == lobby.started_by_profile_id)
        .and_then(|profile| profile.pin.clone())
}

#[component]
fn ProfilesHeader() -> Element {
    let navigator = use_navigator();
    let show_settings = settings_enabled();

    rsx! {
        header {
            class: "grid grid-cols-[1fr_auto_1fr] items-start gap-4 text-center",
            "data-focus-scope": "true",
            "data-focus-region": "top-bar",
            div { class: "justify-self-start",
                if show_settings {
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::IconLg,
                        action_label: "Open settings".to_string(),
                        onclick: move |_| {
                            navigator.push(Route::SettingsImageUpdates {});
                        },
                        Icon {
                            icon: HiCog,
                            class: "h-7 w-7 sm:h-8 sm:w-8",
                            width: None,
                            height: None,
                            title: None,
                        }
                    }
                }
            }
            h1 { class: "text-5xl font-bold tracking-tight sm:text-6xl md:text-7xl",
                "Who's playing?"
            }
            div { class: "justify-self-end", SessionShutdownControl {} }
        }
    }
}
