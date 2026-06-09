use dioxus::prelude::*;
use wolf_api::profiles::{Profile, ProfileListResponse};

use crate::Route;
use crate::api::ApiContext;
use crate::components::primitives::{Button, ButtonSize, StatusAlert, StatusAlertVariant};
use crate::components::{ProfileCard, ProfileCardSkeleton, SessionShutdownControl};
use crate::domain::image_loader::load_image_src;
use crate::input::navigate_hint;

const CARD_SKELETON_COUNT: usize = 3;

#[component]
pub fn Profiles() -> Element {
    let mut profiles = use_resource(move || async move {
        ApiContext::consume().profiles().list().await.map_err(|_| {
            "Profiles could not be loaded. Check that Wolf is running, then try again.".to_string()
        })
    });

    rsx! {
        div { class: "h-full",
            section {
                class: "relative flex h-full flex-col px-8 py-12 sm:px-12 lg:px-20",
                ProfilesHeader {}
                div {
                    class: "-mx-8 flex flex-1 items-center justify-center py-10 sm:-mx-12 lg:-mx-20",
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
fn ProfilesHeader() -> Element {
    rsx! {
        header {
            class: "grid grid-cols-[1fr_auto_1fr] items-start gap-4 text-center",
            "data-focus-scope": "true",
            "data-focus-region": "top-bar",
            div {}
            h1 { class: "text-4xl font-bold tracking-tight lg:text-6xl 2xl:text-7xl", "Who's playing?" }
            div { class: "justify-self-end", SessionShutdownControl {} }
        }
    }
}

#[component]
fn ProfilesLoading() -> Element {
    rsx! {
        div { class: "scrollbar-hide w-full overflow-x-auto",
            div { class: "mx-auto flex w-max min-w-fit snap-x snap-mandatory gap-5 px-8 py-8 md:gap-6 md:px-12 xl:gap-7 xl:px-20 2xl:gap-10 2xl:px-28",
                div { class: "flex min-w-[calc(100vw-4rem)] justify-center gap-5 md:min-w-[calc(100vw-6rem)] md:gap-6 xl:min-w-[calc(100vw-10rem)] xl:gap-7 2xl:min-w-[calc(100vw-14rem)] 2xl:gap-10",
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
        div { class: "scrollbar-hide w-full overflow-x-auto",
            div {
                class: "mx-auto flex w-max min-w-fit snap-x snap-mandatory gap-5 px-8 py-8 md:gap-6 md:px-12 xl:gap-7 xl:px-20 2xl:gap-10 2xl:px-28",
                role: "list",
                aria_label: "Profiles",
                div { class: "flex min-w-[calc(100vw-4rem)] justify-center gap-5 md:min-w-[calc(100vw-6rem)] md:gap-6 xl:min-w-[calc(100vw-10rem)] xl:gap-7 2xl:min-w-[calc(100vw-14rem)] 2xl:gap-10",
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
