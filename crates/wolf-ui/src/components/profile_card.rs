use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons::{HiLockClosed, HiUser};

use crate::components::Skeleton;
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
            class: "group relative flex h-96 w-72 shrink-0 flex-col items-center overflow-hidden rounded-4xl border border-border bg-card text-center text-card-foreground shadow-2xl shadow-black/30 outline-none transition duration-200 ease-out hover:-translate-y-1 hover:border-foreground/35 hover:bg-accent focus:-translate-y-1 focus:border-foreground focus:ring-4 focus:ring-ring/20 active:scale-[1.05]",
            "data-focusable": "true",
            "data-actions": actions,
            aria_label: "{accessible_label}",
            onmounted: move |event: MountedEvent| async move {
                if autofocus {
                    let _ = event.data().set_focus(true).await;
                }
            },
            div { class: "pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_50%_12%,oklch(1_0_0/0.08),transparent_36%)] opacity-0 transition group-hover:opacity-100 group-focus:opacity-100" }
            div { class: "flex flex-1 items-center justify-center pt-12",
                div {
                    class: "flex h-32 w-32 items-center justify-center overflow-hidden rounded-full bg-muted text-muted-foreground transition group-hover:bg-secondary group-hover:text-secondary-foreground group-focus:bg-secondary group-focus:text-secondary-foreground",
                    if let Some(avatar_src) = profile.avatar_src.as_ref() {
                        img {
                            class: "h-full w-full object-cover",
                            src: avatar_src.clone(),
                            alt: "{profile.name}",
                            loading: "lazy",
                        }
                    } else {
                        Icon {
                            icon: HiUser,
                            class: "h-14 w-14",
                            width: None,
                            height: None,
                            title: Some("Profile".to_string()),
                        }
                    }
                }
            }
            div { class: "relative flex w-full flex-col items-center px-8 pb-10",
                h2 { class: "max-w-full truncate text-4xl font-bold tracking-tight", "{profile.name}" }
            }
            if profile.is_pin_locked {
                span { class: "absolute right-5 top-5 inline-flex items-center gap-1.5 rounded-full border border-border bg-background/50 px-3 py-1 text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground",
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

#[component]
pub fn ProfileCardSkeleton() -> Element {
    rsx! {
        article { class: "flex h-96 w-72 shrink-0 animate-pulse flex-col items-center justify-center rounded-4xl border border-border bg-card shadow-2xl shadow-black/20",
            Skeleton { class: "mb-24 h-32 w-32 rounded-full" }
            Skeleton { class: "h-9 w-36 rounded-full" }
        }
    }
}
