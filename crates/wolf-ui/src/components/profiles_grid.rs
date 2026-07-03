use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons::{HiUser, HiUserGroup};
use wolf_api::lobbies::Lobby;
use wolf_api::profiles::Profile;

use crate::Route;
use crate::api::ApiContext;
use crate::components::PinInputDialog;
use crate::components::persona_card::{PersonaCard, PersonaCardSkeleton};
use crate::components::primitives::{
    Badge, BadgeVariant, CardGrid, StatusAlert, StatusAlertVariant, ToastOptions, use_toasts,
};
use crate::domain::image_loader::load_image_src;
use crate::domain::profiles::ProfilesState;

const CARD_SKELETON_COUNT: usize = 3;

#[component]
pub fn ProfilesLoading() -> Element {
    rsx! {
        ProfilesGrid {
            for _ in 0..CARD_SKELETON_COUNT {
                PersonaCardSkeleton {}
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
                message: "Wolf returned an unsuccessful profiles response. Try again once the service is ready."
                    .to_string(),
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
        ProfilesGrid {
            for (index, profile) in response.profiles.iter().cloned().enumerate() {
                ProfileCard { key: "{profile.id}", profile, autofocus: index == 0 }
            }
            for (index, lobby) in lobbies.iter().cloned().enumerate() {
                LobbyCard {
                    key: "{lobby.id}",
                    lobby: lobby.clone(),
                    profiles: response.profiles.clone(),
                    autofocus: response.profiles.is_empty() && index == 0,
                    onclick: move |_| on_lobby_click.call(lobby.clone()),
                }
            }
        }
    }
}

#[component]
fn ProfilesGrid(children: Element) -> Element {
    rsx! {
        div { class: "my-auto flex min-h-full w-full items-center justify-center py-6 sm:py-8 lg:py-10",
            CardGrid { columns: 5, fit: true, {children} }
        }
    }
}

#[component]
fn ProfileCard(profile: Profile, autofocus: bool) -> Element {
    let navigator = use_navigator();
    let mut show_pin_prompt = use_signal(|| false);
    let toasts = use_toasts();
    let icon_path = profile.icon_png_path.trim().to_string();
    let avatar = use_resource(move || {
        let icon_path = icon_path.clone();

        async move { load_image_src(&ApiContext::consume(), &icon_path).await }
    });
    let avatar_src = avatar.read().clone().flatten();
    let route = Route::ProfileApps {
        profile_id: profile.id.clone(),
    }
    .to_string();
    let pin = profile.pin.clone();

    rsx! {
        PersonaCard {
            name: profile.name.clone(),
            to: if pin.is_none() { Some(route.clone()) } else { None },
            onclick: if pin.is_some() { Some(EventHandler::new(move |_| show_pin_prompt.set(true))) } else { None },
            autofocus,
            pin_locked: profile.pin.is_some(),
            avatar: rsx! {
                if let Some(avatar_src) = avatar_src {
                    img {
                        class: "h-full w-full object-cover",
                        src: avatar_src,
                        loading: "lazy",
                        draggable: "false",
                    }
                } else {
                    Icon {
                        icon: HiUser,
                        class: "mb-2 h-24 w-24",
                        width: None,
                        height: None,
                    }
                }
            },
        }
        if show_pin_prompt() {
            PinInputDialog {
                title: "Profile PIN required".to_string(),
                description: format!("Enter the PIN for {}", profile.name),
                submit_label: "Unlock".to_string(),
                onsubmit: move |entered_pin| {
                    if Some(entered_pin) == pin {
                        navigator.push(route.clone());
                    } else {
                        let mut toasts = toasts;
                        toasts.show("Incorrect PIN. Try again.", ToastOptions::error());
                    }
                    show_pin_prompt.set(false);
                },
                oncancel: move |_| show_pin_prompt.set(false),
            }
        }
    }
}

#[component]
fn LobbyCard(
    lobby: Lobby,
    profiles: Vec<Profile>,
    autofocus: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let connected_users = lobby.connected_sessions.len();

    rsx! {
        PersonaCard {
            name: lobby_name(&lobby, &profiles),
            autofocus,
            pin_locked: lobby.pin_required,
            onclick: move |event| onclick.call(event),
            avatar: rsx! {
                Icon {
                    icon: HiUserGroup,
                    class: "mb-2 h-24 w-24",
                    width: None,
                    height: None,
                }
            },
            badges: rsx! {
                Badge {
                    variant: BadgeVariant::Success,
                    class: "absolute left-4 top-4 lg:left-5 lg:top-5",
                    Icon {
                        icon: HiUserGroup,
                        class: "h-3.5 w-3.5",
                        width: None,
                        height: None,
                    }
                    "{connected_users}"
                }
            },
        }
    }
}

fn lobby_name(lobby: &Lobby, profiles: &[Profile]) -> String {
    let creator = profiles
        .iter()
        .find(|profile| profile.id == lobby.started_by_profile_id)
        .map(|profile| profile.name.as_str());

    match creator {
        Some(creator) if creator.ends_with('s') => format!("{creator}' lobby"),
        Some(creator) => format!("{creator}'s lobby"),
        None => lobby.name.clone(),
    }
}
