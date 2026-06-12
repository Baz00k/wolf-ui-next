use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons::HiLockClosed;

use crate::components::primitives::{Badge, BadgeVariant, Card, CardTrigger, Skeleton};

/// Avatar-style selection card shared by the profile and lobby browsers.
///
/// Navigates to `to` when set, otherwise emits `onclick`. Extra corner badges
/// can be supplied via `badges`; the PIN badge is built in.
#[component]
pub fn PersonaCard(
    name: String,
    #[props(default)] to: Option<String>,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    #[props(default)] autofocus: bool,
    #[props(default)] pin_locked: bool,
    #[props(default = rsx! {})] badges: Element,
    avatar: Element,
) -> Element {
    rsx! {
        CardTrigger {
            class: "text-center",
            to,
            autofocus,
            onclick,
            Card { class: "relative flex h-full w-full flex-col items-center overflow-hidden shadow-black/30 transition duration-200 ease-out group-hover:border-foreground/35 group-hover:bg-accent group-focus:border-foreground group-focus:ring-4 group-focus:ring-ring/60 group-focus:ring-offset-2 group-focus:ring-offset-background",
                PersonaCardMedia {
                    div { class: "mb-6 flex h-44 w-44 items-center justify-center overflow-hidden rounded-full bg-muted text-muted-foreground transition group-hover:bg-secondary group-hover:text-secondary-foreground group-focus:bg-secondary group-focus:text-secondary-foreground",
                        {avatar}
                    }
                }
                PersonaCardLabel {
                    h2 { class: "max-w-full truncate text-4xl font-bold tracking-tight", "{name}" }
                }
                {badges}
                if pin_locked {
                    Badge { variant: BadgeVariant::Neutral, class: "absolute right-4 top-4 lg:right-5 lg:top-5",
                        Icon { icon: HiLockClosed, class: "h-3.5 w-3.5", width: None, height: None }
                        "PIN"
                    }
                }
            }
        }
    }
}

#[component]
pub fn PersonaCardSkeleton() -> Element {
    rsx! {
        Card { class: "relative flex aspect-[3/4] w-full animate-pulse flex-col items-center overflow-hidden shadow-black/20",
            PersonaCardMedia {
                Skeleton { class: "mb-6 h-44 w-44 rounded-full" }
            }
            PersonaCardLabel {
                Skeleton { class: "h-10 w-40 rounded-full" }
            }
        }
    }
}

#[component]
fn PersonaCardMedia(children: Element) -> Element {
    rsx! {
        div { class: "flex flex-1 items-center justify-center pt-8 md:pt-10 lg:pt-12 2xl:pt-16",
            {children}
        }
    }
}

#[component]
fn PersonaCardLabel(children: Element) -> Element {
    rsx! {
        div { class: "relative flex w-full flex-col items-center px-6 pb-12 md:px-7 lg:px-8",
            {children}
        }
    }
}
