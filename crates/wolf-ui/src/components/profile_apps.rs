use std::collections::HashMap;

use dioxus::prelude::*;
use wolf_api::lobbies::Lobby;
use wolf_api::profiles::AppListResponse;

use crate::components::primitives::{
    Button, ButtonSize, ButtonVariant, StatusAlert, StatusAlertVariant,
};
use crate::components::{AppCard, AppCardData, AppCardSkeleton, SessionShutdownControl};
use crate::domain::apps::{AppFilter, filter_label, sorted_apps};
use crate::input::navigate_hint;

pub const APP_GRID_VIEWPORT_CLASS: &str = "h-full w-full overflow-y-auto overflow-x-hidden scroll-pt-4 scroll-pb-4 scrollbar-hide sm:scroll-pt-5 sm:scroll-pb-5 lg:scroll-pt-6 lg:scroll-pb-6";
pub const APP_GRID_CLASS: &str = "mx-auto grid w-full max-w-[min(100%,calc(22rem*6+2rem*5))] grid-cols-[repeat(auto-fill,minmax(min(100%,14rem),18rem))] justify-center gap-4 p-2 sm:grid-cols-[repeat(auto-fill,minmax(16rem,20rem))] sm:gap-5 sm:p-3 xl:grid-cols-[repeat(auto-fill,minmax(18rem,22rem))] xl:gap-6 lg:p-4 2xl:gap-8 2xl:p-5";

const LOADING_CARD_COUNT: usize = 8;

#[component]
pub fn AppsHeader(
    filter: Signal<AppFilter>,
    selected_index: Signal<usize>,
    pending_focus_index: Signal<Option<usize>>,
) -> Element {
    rsx! {
        header {
            id: "apps-top-bar",
            class: "grid grid-cols-[1fr_auto] items-start gap-4",
            "data-focus-scope": "true",
            "data-focus-region": "top-bar",
            "data-scope-actions": navigate_hint("Navigate"),
            div { class: "min-w-0",
                h1 { class: "text-4xl font-bold tracking-tight lg:text-5xl", "Applications" }
            }
            div { class: "flex min-w-0 items-center justify-end gap-3 sm:gap-4",
                AppsFilter { filter, selected_index, pending_focus_index }
                SessionShutdownControl {}
            }
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
            class: "rounded-lg flex gap-1 border border-border bg-card/70 p-1 font-mono text-xs font-semibold uppercase tracking-widest text-muted-foreground shadow-lg shadow-black/20",
            for item in [AppFilter::Available, AppFilter::Alphabetical] {
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
        "px-3 py-2 font-mono text-xs font-semibold uppercase tracking-widest"
    } else {
        "px-3 py-2 font-mono text-xs font-semibold uppercase tracking-widest text-muted-foreground"
    };

    rsx! {
        Button {
            variant: if active { ButtonVariant::Default } else { ButtonVariant::Ghost },
            size: ButtonSize::Sm,
            class,
            action_label: label.clone(),
            onclick: move |event| onclick.call(event),
            "{label}"
        }
    }
}

#[component]
pub fn AppsLoading() -> Element {
    rsx! {
        div { class: APP_GRID_VIEWPORT_CLASS,
            div { class: APP_GRID_CLASS,
                for _ in 0..LOADING_CARD_COUNT {
                    AppCardSkeleton {}
                }
            }
        }
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
            div {
                class: "flex justify-center items-center h-full",
                StatusAlert {
                    title: Some("Apps unavailable".to_string()),
                    message: "Wolf returned an unsuccessful app response. Try again once the service is ready.".to_string(),
                    variant: StatusAlertVariant::Error,
                }
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
            div {
                class: "flex justify-center items-center h-full",
                StatusAlert {
                    title: Some("No apps found".to_string()),
                    message: "Add apps to this Wolf profile before launching a Moonlight session.".to_string(),
                    variant: StatusAlertVariant::Info,
                }
            }
        };
    }

    if *selected_index.peek() >= apps.len() {
        selected_index.set(apps.len().saturating_sub(1));
    }

    rsx! {
        div { class: APP_GRID_VIEWPORT_CLASS,
            div {
                class: APP_GRID_CLASS,
                for (index, app) in apps.iter().cloned().enumerate() {
                    div { key: "{app.id}",
                        AppCard {
                            app: app.clone(),
                            index,
                            autofocus: index == 0,
                            onfocus: move |_| {
                                if *selected_index.peek() != index {
                                    selected_index.set(index);
                                }
                            },
                            onclick: move |_| {
                                if *selected_index.peek() != index {
                                    selected_index.set(index);
                                }
                                on_app_click.call((index, app.clone()));
                            },
                        }
                    }
                }
            }
        }
    }
}
