use std::collections::HashMap;

use dioxus::prelude::*;
use wolf_api::lobbies::Lobby;
use wolf_api::profiles::AppListResponse;

use crate::components::{
    AppCard, AppCardData, AppCardSkeleton, Button, ButtonSize, ButtonVariant, StatusAlert,
    StatusAlertVariant,
};
use crate::domain::apps::{AppFilter, filter_label, sorted_apps};

pub const APP_CAROUSEL_CLASS: &str =
    "w-full snap-x snap-mandatory overflow-x-auto overflow-y-visible [scrollbar-width:none]";
pub const APP_CAROUSEL_TRACK_CLASS: &str = "mx-auto flex min-w-max items-center gap-5 px-[calc(50vw-7rem)] py-16 md:px-[calc(50vw-8rem)] lg:gap-6 lg:px-[calc(50vw-9rem)] xl:gap-7 xl:px-[calc(50vw-10rem)] 2xl:px-[calc(50vw-12rem)]";

const LOADING_CARD_COUNT: usize = 5;

#[component]
pub fn AppsHeader(
    filter: Signal<AppFilter>,
    selected_index: Signal<usize>,
    pending_focus_index: Signal<Option<usize>>,
) -> Element {
    rsx! {
        header { class: "flex items-start justify-between gap-4",
            div {
                h1 { class: "text-2xl font-bold tracking-tight sm:text-4xl lg:text-5xl", "Applications" }
            }
            AppsFilter { filter, selected_index, pending_focus_index }
        }
    }
}

#[component]
fn AppsFilter(
    mut filter: Signal<AppFilter>,
    mut selected_index: Signal<usize>,
    mut pending_focus_index: Signal<Option<usize>>,
) -> Element {
    rsx! {
        div {
            id: "apps-filter",
            class: "rounded-full border border-border bg-card/70 p-1 font-mono text-xs font-semibold uppercase tracking-widest text-muted-foreground shadow-lg shadow-black/20",
            for item in [AppFilter::Default, AppFilter::Available, AppFilter::Alphabetical] {
                FilterButton {
                    active: filter() == item,
                    label: filter_label(item).to_string(),
                    onclick: move |_| {
                        selected_index.set(0);
                        filter.set(item);
                        pending_focus_index.set(Some(0));
                    },
                }
            }
        }
    }
}

#[component]
fn FilterButton(active: bool, label: String, onclick: EventHandler<MouseEvent>) -> Element {
    let class = if active {
        "rounded-full px-3 py-2 font-mono text-xs font-semibold uppercase tracking-widest"
    } else {
        "rounded-full px-3 py-2 font-mono text-xs font-semibold uppercase tracking-widest text-muted-foreground"
    };

    rsx! {
        Button {
            variant: if active { ButtonVariant::Default } else { ButtonVariant::Ghost },
            size: ButtonSize::Sm,
            class,
            action_label: label.clone(),
            aria_pressed: if active { Some("true".to_string()) } else { Some("false".to_string()) },
            onclick: move |event| onclick.call(event),
            "{label}"
        }
    }
}

#[component]
pub fn AppsLoading() -> Element {
    rsx! {
        div { class: APP_CAROUSEL_CLASS,
            div { class: APP_CAROUSEL_TRACK_CLASS,
                for _ in 0..LOADING_CARD_COUNT {
                    AppCardSkeleton {}
                }
            }
        }
        LoadingAppMeta {}
    }
}

#[component]
pub fn AppsContent(
    response: AppListResponse,
    profile_id: String,
    lobbies: Vec<Lobby>,
    images: HashMap<String, bool>,
    covers: HashMap<String, String>,
    filter: AppFilter,
    mut selected_index: Signal<usize>,
    on_app_click: EventHandler<(usize, AppCardData)>,
) -> Element {
    if !response.success {
        return rsx! {
            StatusAlert {
                title: "Apps unavailable".to_string(),
                message: "Wolf returned an unsuccessful app response. Try again once the service is ready.".to_string(),
                variant: StatusAlertVariant::Error,
            }
        };
    }

    let apps = sorted_apps(
        &profile_id,
        response.apps,
        &lobbies,
        &images,
        &covers,
        filter,
    );
    if apps.is_empty() {
        return rsx! {
            StatusAlert {
                title: "No apps found".to_string(),
                message: "Add apps to this Wolf profile before launching a Moonlight session.".to_string(),
                variant: StatusAlertVariant::Info,
            }
        };
    }

    if selected_index() >= apps.len() {
        selected_index.set(apps.len().saturating_sub(1));
    }

    rsx! {
        div { class: APP_CAROUSEL_CLASS,
            div {
                class: APP_CAROUSEL_TRACK_CLASS,
                role: "list",
                aria_label: "Applications",
                for (index, app) in apps.iter().cloned().enumerate() {
                    div { role: "listitem", key: "{app.id}",
                        AppCard {
                            app: app.clone(),
                            index,
                            selected: selected_index() == index,
                            autofocus: index == 0,
                            onfocus: move |_| selected_index.set(index),
                            onclick: move |_| {
                                selected_index.set(index);
                                on_app_click.call((index, app.clone()));
                            },
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn LoadingAppMeta() -> Element {
    rsx! {
        div { class: "pointer-events-none flex h-28 shrink-0 flex-col items-center justify-center gap-3 px-6 text-center",
            div { class: "h-9 w-48 animate-pulse rounded-full bg-muted sm:h-10 sm:w-56 lg:h-11 lg:w-64" }
            div { class: "h-8 w-36 animate-pulse rounded-full border border-border bg-card/70" }
        }
    }
}
