use dioxus::prelude::*;
use wolf_api::profiles::{Profile, ProfileListResponse};

use crate::Route;
use crate::api::ApiContext;
use crate::components::primitives::{Button, ButtonSize, StatusAlert, StatusAlertVariant};
use crate::components::{ProfileCard, ProfileCardSkeleton, SessionShutdownControl};
use crate::domain::image_loader::load_image_src;
use crate::input::navigate_hint;

const CARD_SKELETON_COUNT: usize = 3;
const PROFILE_GRID_CLASS: &str = "mx-auto grid w-full max-w-[min(100%,calc(22rem*5+2rem*4))] grid-cols-[repeat(auto-fit,minmax(min(100%,14rem),18rem))] justify-center gap-4 p-2 sm:grid-cols-[repeat(auto-fit,minmax(16rem,20rem))] sm:gap-5 sm:p-3 xl:grid-cols-[repeat(auto-fit,minmax(18rem,22rem))] xl:gap-6 lg:p-4 2xl:gap-8 2xl:p-5";

#[component]
pub fn Profiles() -> Element {
    let mut profiles = use_resource(move || async move {
        ApiContext::consume().profiles().list().await.map_err(|_| {
            "Profiles could not be loaded. Check that Wolf is running, then try again.".to_string()
        })
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
                        Some(Ok(response)) => rsx! {
                            ProfilesContent { response: response.clone() }
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

#[component]
fn ProfilesLoading() -> Element {
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
fn ProfilesContent(response: ProfileListResponse) -> Element {
    if !response.success {
        return rsx! {
            StatusAlert {
                title: Some("Profiles unavailable".to_string()),
                message: "Wolf returned an unsuccessful profiles response. Try again once the service is ready.".to_string(),
                variant: StatusAlertVariant::Error,
            }
        };
    }

    if response.profiles.is_empty() {
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
            div {
                class: PROFILE_GRID_CLASS,
                role: "list",
                aria_label: "Profiles",
                for (index, profile) in response.profiles.iter().cloned().enumerate() {
                    div { role: "listitem",
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

fn profile_card_data(profile: Profile) -> crate::components::profile_card::ProfileCardData {
    crate::components::profile_card::ProfileCardData {
        id: profile.id,
        name: profile.name,
        avatar_src: None,
        is_pin_locked: profile.pin.is_some(),
    }
}
