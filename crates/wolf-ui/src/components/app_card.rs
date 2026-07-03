use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons::HiCube;

use crate::components::primitives::{Badge, BadgeVariant, Card, CardTrigger, Skeleton};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppStatusKind {
    Ready,
    Playing,
    MissingImage,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppStatusTone {
    Ready,
    Warning,
}

#[derive(Clone, PartialEq, Eq)]
pub struct AppStatus {
    pub kind: AppStatusKind,
    pub label: String,
    pub tone: AppStatusTone,
}

#[derive(Clone, PartialEq)]
pub struct AppCardData {
    pub id: String,
    pub title: String,
    pub runner: String,
    pub source: wolf_api::profiles::App,
    pub status: AppStatus,
    pub supports_hdr: bool,
    pub cover_src: Option<String>,
}

#[component]
pub fn AppCard(
    app: AppCardData,
    index: usize,
    autofocus: bool,
    onfocus: EventHandler<FocusEvent>,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let show_runner = app.runner != "Docker";
    let is_unavailable = app.status.kind == AppStatusKind::MissingImage;
    let is_playing = app.status.kind == AppStatusKind::Playing;

    rsx! {
        CardTrigger {
            class: "text-left transition-transform duration-300",
            action_label: "Actions",
            index,
            autofocus,
            onfocus,
            onclick,
            Card { class: "relative h-full w-full overflow-visible border-border/70 text-muted-foreground opacity-80 shadow-black/25 transition duration-300 ease-out group-focus-visible:border-foreground group-focus-visible:text-card-foreground group-focus-visible:opacity-100 group-focus-visible:shadow-[0_1.5rem_3rem_oklch(0_0_0/0.5)] group-focus-visible:ring-4 group-focus-visible:ring-ring/60 group-focus-visible:ring-offset-2 group-focus-visible:ring-offset-background",
                div { class: "pointer-events-none absolute inset-x-4 top-4 z-20 flex items-start justify-between gap-3 text-xs font-semibold uppercase tracking-widest text-muted-foreground md:inset-x-5 md:top-5 xl:inset-x-6 xl:top-6",
                    if show_runner {
                        span { "{app.runner}" }
                    } else {
                        span {}
                    }
                    div { class: "flex flex-col items-end gap-2",
                        if app.supports_hdr {
                            Badge { variant: BadgeVariant::Warning, class: "px-2", "HDR" }
                        }
                        if is_unavailable {
                            StatusBadge { label: "Not installed" }
                        }
                        if is_playing {
                            StatusBadge { label: "Playing", pulse: true }
                        }
                    }
                }
                div { class: "relative flex h-full items-center justify-center overflow-hidden rounded-4xl",
                    if let Some(cover_src) = app.cover_src.as_ref() {
                        img {
                            class: "absolute inset-0 h-full w-full rounded-4xl object-cover transition duration-300",
                            class: if is_unavailable { "grayscale brightness-40" },
                            src: cover_src.clone(),
                            loading: "lazy",
                            draggable: "false",
                        }
                    } else {
                        div { class: "flex h-20 w-20 items-center justify-center rounded-3xl border border-border/70 bg-background/70 text-muted-foreground shadow-inner transition duration-300 md:h-24 md:w-24 lg:h-28 lg:w-28 xl:h-32 xl:w-32 2xl:h-36 2xl:w-36",
                            Icon {
                                icon: HiCube,
                                class: "w-24 h-24",
                                width: None,
                                height: None,
                            }
                        }
                    }
                    div { class: "pointer-events-none absolute inset-x-0 bottom-0 z-10 bg-gradient-to-t from-black/80 via-black/45 to-transparent px-6 pb-6 pt-8 lg:px-8",
                        h2 { class: "overflow-hidden truncate line-clamp-2 text-3xl font-black leading-tight tracking-tight text-white",
                            "{app.title}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn StatusBadge(label: &'static str, #[props(default)] pulse: bool) -> Element {
    rsx! {
        Badge { variant: BadgeVariant::Overlay, class: "gap-2 py-1.5",
            if pulse {
                span { class: "h-2 w-2 animate-pulse rounded-full bg-emerald-400 shadow-lg shadow-emerald-400/50" }
            }
            span { class: "text-xs font-semibold uppercase tracking-widest text-muted-foreground",
                "{label}"
            }
        }
    }
}

#[component]
pub fn AppCardSkeleton() -> Element {
    rsx! {
        article { class: "flex aspect-[3/4] w-full animate-pulse flex-col overflow-hidden rounded-4xl border border-border bg-card/60 opacity-70 shadow-2xl shadow-black/20",
            div { class: "relative flex flex-1 items-center justify-center rounded-4xl",
                Skeleton { class: "absolute inset-0 rounded-4xl" }
                div { class: "absolute inset-x-4 top-4 z-20 flex items-start justify-between gap-3 md:inset-x-5 md:top-5 xl:inset-x-6 xl:top-6",
                    Skeleton { class: "h-5 w-16 rounded-full" }
                    Skeleton { class: "h-7 w-28 rounded-full" }
                }
                Skeleton { class: "h-20 w-20 rounded-3xl md:h-24 md:w-24 lg:h-28 lg:w-28 xl:h-32 xl:w-32 2xl:h-36 2xl:w-36" }
                div { class: "absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/80 via-black/45 to-transparent px-6 pb-6 pt-8 lg:px-8",
                    Skeleton { class: "h-9 w-3/5 rounded-full" }
                }
            }
        }
    }
}
