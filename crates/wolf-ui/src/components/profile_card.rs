use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons::{HiLockClosed, HiUser};

use crate::components::primitives::{Card, Skeleton};
use crate::input::{UiAction, native_action};

#[derive(Clone, PartialEq)]
pub struct ProfileCardData {
    pub id: String,
    pub name: String,
    pub avatar_src: Option<String>,
    pub is_pin_locked: bool,
}

#[component]
pub fn ProfileCard(profile: ProfileCardData, autofocus: bool, to: String) -> Element {
    let status_label = if profile.is_pin_locked {
        "PIN protected profile"
    } else {
        "Open profile"
    };
    let accessible_label = format!("Select {}, {}", profile.name, status_label);
    let actions = native_action(UiAction::Accept, "Select");

    rsx! {
        Link {
            to,
            class: "group relative flex aspect-[3/4] w-full text-center outline-none transition duration-200 ease-out hover:-translate-y-1 focus:-translate-y-1 active:scale-105",
            "data-focusable": "true",
            "data-actions": actions,
            aria_label: "{accessible_label}",
            onmounted: move |event: MountedEvent| async move {
                if autofocus {
                    let _ = event.data().set_focus(true).await;
                }
            },
            Card { class: "relative flex h-full w-full flex-col items-center overflow-hidden shadow-black/30 transition duration-200 ease-out group-hover:border-foreground/35 group-hover:bg-accent group-focus:border-foreground group-focus:ring-4 group-focus:ring-ring/20".to_string(),
                div { class: "flex flex-1 items-center justify-center pt-8 md:pt-10 lg:pt-12 2xl:pt-16",
                    div {
                        class: "mb-6 flex h-44 w-44 items-center justify-center overflow-hidden rounded-full bg-muted text-muted-foreground transition group-hover:bg-secondary group-hover:text-secondary-foreground group-focus:bg-secondary group-focus:text-secondary-foreground",
                        if let Some(avatar_src) = profile.avatar_src.as_ref() {
                            img {
                                class: "h-full w-full object-cover",
                                src: avatar_src.clone(),
                                alt: "{profile.name}",
                                loading: "lazy",
                                draggable: "false",
                            }
                        } else {
                            Icon {
                                icon: HiUser,
                                class: "h-24 w-24 mb-2",
                                width: None,
                                height: None,
                                title: Some("Profile".to_string()),
                            }
                        }
                    }
                }
                div { class: "relative flex w-full flex-col items-center px-6 pb-12 md:px-7 lg:px-8",
                    h2 { class: "max-w-full truncate text-4xl font-bold tracking-tight", "{profile.name}" }
                }
                if profile.is_pin_locked {
                    span { class: "absolute right-4 top-4 inline-flex items-center gap-1.5 rounded-full border border-border bg-background/50 px-3 py-1 text-xs font-semibold uppercase tracking-widest text-muted-foreground lg:right-5 lg:top-5",
                        Icon {
                            icon: HiLockClosed,
                            class: "h-3.5 w-3.5",
                            width: None,
                            height: None,
                            title: Some("PIN protected".to_string()),
                        }
                        "PIN"
                    }
                }
            }
        }
    }
}

#[component]
pub fn ProfileCardSkeleton() -> Element {
    rsx! {
        Card { class: "relative flex aspect-[3/4] w-full animate-pulse flex-col items-center overflow-hidden shadow-black/20".to_string(),
            div { class: "flex flex-1 items-center justify-center pt-8 md:pt-10 lg:pt-12 2xl:pt-16",
                Skeleton { class: "mb-6 h-44 w-44 rounded-full" }
            }
            div { class: "relative flex w-full flex-col items-center px-6 pb-12 md:px-7 lg:px-8",
                Skeleton { class: "h-10 w-40 rounded-full" }
            }
        }
    }
}
