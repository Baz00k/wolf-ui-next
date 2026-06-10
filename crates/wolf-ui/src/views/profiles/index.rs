use dioxus::prelude::*;
use wolf_api::lobbies::Lobby;

use crate::components::primitives::{Button, ButtonSize, StatusAlert, StatusAlertVariant};
use crate::components::{
    LobbyActionDialog, ProfilesContent, ProfilesLoading, SessionShutdownControl,
};
use crate::domain::app_actions::{join_lobby, stop_lobby};
use crate::domain::profiles::{listen_for_lobby_events, load_profiles_state};
use crate::input::navigate_hint;

#[component]
pub fn Profiles() -> Element {
    let mut lobbies = use_signal(Vec::<Lobby>::new);
    let mut action_lobby = use_signal(|| None::<Lobby>);
    let mut profiles = use_resource(move || async move { load_profiles_state().await });
    let join_runner = use_action(move |lobby_id: String| async move {
        join_lobby(lobby_id).await.map_err(std::io::Error::other)
    });
    let stop_runner = use_action(move |lobby_id: String| async move {
        stop_lobby(lobby_id).await.map_err(std::io::Error::other)
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
            section {
                class: "relative flex h-full min-h-0 flex-col px-6 pt-8 sm:px-10 sm:pt-10 lg:px-16 lg:pt-12",
                ProfilesHeader {}
                div {
                    class: "flex min-h-0 flex-1 overflow-y-auto overflow-x-hidden scroll-pt-6 scroll-pb-6 scrollbar-hide sm:scroll-pt-8 sm:scroll-pb-8 lg:scroll-pt-10 lg:scroll-pb-10",
                    "data-focus-scope": "true",
                    "data-focus-region": "main",
                    "data-scope-actions": navigate_hint("Navigate"),
                    match &*profiles.read_unchecked() {
                        Some(Ok(state)) => rsx! {
                            ProfilesContent {
                                state: state.clone(),
                                lobbies: lobbies(),
                                on_lobby_click: move |lobby: Lobby| action_lobby.set(Some(lobby)),
                            }
                        },
                        Some(Err(message)) => rsx! {
                            StatusAlert {
                                title: Some("Profiles unavailable".to_string()),
                                message: message.clone(),
                                variant: StatusAlertVariant::Error,
                                Button {
                                    size: ButtonSize::Lg,
                                    onclick: move |_| profiles.restart(),
                                    "Retry"
                                }
                            }
                        },
                        None => rsx! { ProfilesLoading {} },
                    }
                }
            }
            if let Some(lobby) = action_lobby() {
                LobbyActionDialog {
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

#[component]
fn ProfilesHeader() -> Element {
    rsx! {
        header {
            class: "grid grid-cols-[1fr_auto_1fr] items-start gap-4 text-center",
            "data-focus-scope": "true",
            "data-focus-region": "top-bar",
            "data-scope-actions": navigate_hint("Navigate"),
            div {}
            h1 { class: "text-5xl font-bold tracking-tight lg:text-6xl 2xl:text-7xl", "Who's playing?" }
            div { class: "justify-self-end", SessionShutdownControl {} }
        }
    }
}
