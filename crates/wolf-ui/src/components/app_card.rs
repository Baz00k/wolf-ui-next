use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons::HiCube;

use crate::components::{Card, Skeleton};
use crate::input::{UiAction, native_action};

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
    selected: bool,
    autofocus: bool,
    onfocus: EventHandler<FocusEvent>,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let actions = native_action(UiAction::Accept, "Actions");
    let scale_class = if selected {
        "scale-100 hover:scale-105"
    } else {
        "scale-90 hover:scale-95"
    };
    let glow_class = if selected { "opacity-100" } else { "opacity-0" };
    let accessible_label = format!("{} app, {}, open actions", app.title, app.status.label);
    let show_runner = app.runner != "Docker";

    rsx! {
        button {
            class: "group relative h-72 w-56 shrink-0 snap-center scroll-mx-10 border-0 p-0 text-left outline-none transition-transform duration-300 ease-out active:scale-95 md:h-80 md:w-64 lg:h-96 lg:w-72 xl:h-[28rem] xl:w-80 2xl:h-[34rem] 2xl:w-96 {scale_class}",
            "data-focusable": "true",
            "data-app-index": "{index}",
            "data-actions": actions,
            aria_label: "{accessible_label}",
            onfocus: move |event| onfocus.call(event),
            onclick: move |event| onclick.call(event),
            onmounted: move |event: MountedEvent| async move {
                if autofocus {
                    let _ = event.data().set_focus(true).await;
                }
            },
            Card { class: app_card_class(selected).to_string(),
                div { class: "pointer-events-none absolute -inset-10 -z-10 rounded-4xl bg-[radial-gradient(circle_at_50%_65%,oklch(1_0_0/0.22),transparent_62%)] blur-2xl transition-opacity duration-300 {glow_class}" }
                div { class: "pointer-events-none absolute inset-0 overflow-hidden rounded-4xl bg-[radial-gradient(circle_at_50%_10%,oklch(1_0_0/0.16),transparent_38%),linear-gradient(180deg,oklch(1_0_0/0.04),transparent_60%)] transition-opacity duration-300 {glow_class}" }
                div { class: "pointer-events-none absolute inset-x-4 top-4 z-10 flex items-center justify-between text-xs font-semibold uppercase tracking-widest text-muted-foreground md:inset-x-5 md:top-5 xl:inset-x-6 xl:top-6",
                    if show_runner {
                        span { "{app.runner}" }
                    } else {
                        span {}
                    }
                    if app.supports_hdr {
                        span { class: "rounded-full border border-yellow-400/30 bg-yellow-400/10 px-2 py-1 text-yellow-300", "HDR" }
                    }
                }
                div { class: "relative flex h-full items-center justify-center overflow-hidden rounded-4xl",
                    if let Some(cover_src) = app.cover_src.as_ref() {
                        img {
                            class: "absolute inset-0 h-full w-full rounded-4xl object-cover",
                            src: cover_src.clone(),
                            alt: "{app.title}",
                            loading: "lazy",
                            draggable: "false",
                        }
                    } else {
                        div { class: "flex h-20 w-20 items-center justify-center rounded-3xl border border-border/70 bg-background/70 text-muted-foreground shadow-inner transition duration-300 md:h-24 md:w-24 lg:h-28 lg:w-28 xl:h-32 xl:w-32 2xl:h-36 2xl:w-36",
                            Icon {
                                icon: HiCube,
                                class: "h-10 w-10 md:h-12 md:w-12 lg:h-14 lg:w-14 xl:h-16 xl:w-16 2xl:h-20 2xl:w-20",
                                width: None,
                                height: None,
                                title: Some("Application".to_string()),
                            }
                        }
                    }
                }
            }
        }
    }
}

fn app_card_class(selected: bool) -> &'static str {
    if selected {
        "relative h-full w-full overflow-visible border-foreground text-card-foreground opacity-100 shadow-[0_2.5rem_4rem_oklch(0_0_0/0.55),0_0_3.5rem_oklch(1_0_0/0.18)] transition-[transform,opacity,border-color,box-shadow,background-color] duration-300 ease-out group-focus:ring-4 group-focus:ring-ring/25"
    } else {
        "relative h-full w-full overflow-visible border-border/70 text-muted-foreground opacity-70 shadow-black/30 transition-[transform,opacity,border-color,box-shadow,background-color] duration-300 ease-out group-focus:ring-4 group-focus:ring-ring/25"
    }
}

#[component]
pub fn AppCardSkeleton() -> Element {
    rsx! {
        article { class: "flex h-72 w-56 shrink-0 animate-pulse snap-center flex-col overflow-hidden rounded-4xl border border-border bg-card/60 opacity-70 shadow-2xl shadow-black/20 md:h-80 md:w-64 lg:h-96 lg:w-72 xl:h-[28rem] xl:w-80 2xl:h-[34rem] 2xl:w-96",
            div { class: "relative flex flex-1 items-center justify-center rounded-4xl",
                Skeleton { class: "absolute inset-0 rounded-4xl" }
                Skeleton { class: "h-20 w-20 rounded-3xl md:h-24 md:w-24 lg:h-28 lg:w-28 xl:h-32 xl:w-32 2xl:h-36 2xl:w-36" }
            }
        }
    }
}
