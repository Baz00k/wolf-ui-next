use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use dioxus::prelude::*;
use wolf_api::profiles::{Profile, ProfileListResponse};

use crate::Route;
use crate::api::ApiContext;
use crate::components::{
    Button, ButtonSize, ProfileCard, ProfileCardSkeleton, Spinner, StatusAlert, StatusAlertVariant,
};

const CARD_SKELETON_COUNT: usize = 3;

#[component]
pub fn Profiles() -> Element {
    let mut profiles = use_resource(move || async move {
        ApiContext::consume()
            .profiles()
            .list()
            .await
            .map_err(|error| error.to_string())
    });

    rsx! {
        div { class: "min-h-screen overflow-hidden bg-background text-foreground",
            div { class: "pointer-events-none fixed inset-0 bg-[radial-gradient(circle_at_50%_100%,oklch(1_0_0/0.1),transparent_42%),linear-gradient(180deg,oklch(1_0_0/0.02),transparent_45%)]" }
            section { class: "relative flex min-h-screen flex-col px-8 py-12 sm:px-12 lg:px-20",
                ProfilesHeader { is_loading: profiles.read_unchecked().is_none() }
                div { class: "-mx-8 flex flex-1 items-center justify-center py-10 sm:-mx-12 lg:-mx-20",
                    match &*profiles.read_unchecked() {
                        Some(Ok(response)) => rsx! {
                            ProfilesContent { response: response.clone() }
                        },
                        Some(Err(error)) => rsx! {
                            StatusAlert {
                                title: "Profiles unavailable".to_string(),
                                message: format!("Wolf did not return the profiles list. {error}"),
                                variant: StatusAlertVariant::Error,
                                Button {
                                    size: ButtonSize::Lg,
                                    class: "rounded-full uppercase tracking-[0.18em]",
                                    onclick: move |_| profiles.restart(),
                                    "Retry"
                                }
                            }
                        },
                        None => rsx! { ProfilesLoading {} },
                    }
                }
            }
        }
    }
}

#[component]
fn ProfilesHeader(is_loading: bool) -> Element {
    rsx! {
        header { class: "flex flex-col items-center justify-center gap-4 text-center",
            h1 { class: "text-4xl font-bold tracking-tight sm:text-6xl", "Who's playing?" }
            if is_loading {
                div { class: "flex items-center gap-3 rounded-full border border-border bg-card px-4 py-2 text-sm font-semibold text-muted-foreground",
                    Spinner { class: "h-4 w-4" }
                    "Loading profiles"
                }
            }
        }
    }
}

#[component]
fn ProfilesLoading() -> Element {
    rsx! {
        div { class: "w-full overflow-x-auto [scrollbar-width:none]",
            div { class: "mx-auto flex w-max min-w-fit snap-x snap-mandatory gap-7 px-8 py-8 sm:px-12 lg:px-20",
                div { class: "flex min-w-[calc(100vw-4rem)] justify-center gap-7 sm:min-w-[calc(100vw-6rem)] lg:min-w-[calc(100vw-10rem)]",
                    for _ in 0..CARD_SKELETON_COUNT {
                        ProfileCardSkeleton {}
                    }
                }
            }
        }
    }
}

#[component]
fn ProfilesContent(response: ProfileListResponse) -> Element {
    if !response.success {
        return rsx! {
            StatusAlert {
                title: "Profiles unavailable".to_string(),
                message: "Wolf returned an unsuccessful profiles response. Try again once the service is ready.".to_string(),
                variant: StatusAlertVariant::Error,
            }
        };
    }

    if response.profiles.is_empty() {
        return rsx! {
            StatusAlert {
                title: "No profiles found".to_string(),
                message: "Create a Wolf profile before launching a Moonlight session.".to_string(),
                variant: StatusAlertVariant::Info,
            }
        };
    }

    rsx! {
        div { class: "w-full overflow-x-auto [scrollbar-width:none]",
            div {
                class: "mx-auto flex w-max min-w-fit snap-x snap-mandatory gap-7 px-8 py-8 sm:px-12 lg:px-20",
                role: "list",
                aria_label: "Profiles",
                div { class: "flex min-w-[calc(100vw-4rem)] justify-center gap-7 sm:min-w-[calc(100vw-6rem)] lg:min-w-[calc(100vw-10rem)]",
                    for (index, profile) in response.profiles.iter().cloned().enumerate() {
                        div { class: "snap-center", role: "listitem",
                            ProfileCardLoader {
                                profile,
                                autofocus: index == 0,
                            }
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

        async move {
            if icon_path.is_empty() {
                return None;
            }

            if is_absolute_url(&icon_path) {
                return Some(icon_path);
            }

            ApiContext::consume()
                .utils()
                .icon(&icon_path)
                .await
                .ok()
                .map(|bytes| format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
        }
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

fn is_absolute_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}

fn profile_card_data(profile: Profile) -> crate::components::profile_card::ProfileCardData {
    crate::components::profile_card::ProfileCardData {
        id: profile.id,
        name: profile.name,
        avatar_src: None,
        is_pin_locked: profile.pin.is_some(),
    }
}
