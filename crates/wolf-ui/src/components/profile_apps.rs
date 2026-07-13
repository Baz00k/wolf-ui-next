use std::collections::HashMap;

use dioxus::prelude::*;
use wolf_api::lobbies::Lobby;
use wolf_api::profiles::AppListResponse;

use crate::components::primitives::{
    Button, ButtonSize, ButtonVariant, CardGrid, CardGridViewport, StatusAlert, StatusAlertVariant,
};
use crate::components::{AppCard, AppCardData, AppCardSkeleton, SessionShutdownControl};
use crate::domain::apps::{AppFilter, filter_label, sorted_apps};

const LOADING_CARD_COUNT: usize = 8;
const APP_GRID_COLUMNS: usize = 6;

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
            div { class: "min-w-0",
                h1 { class: "text-4xl font-bold tracking-tight sm:text-5xl", "Applications" }
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
            class: "flex gap-2 rounded-lg border border-border bg-card/70 p-1 shadow-lg shadow-black/20",
            for item in [AppFilter::Available, AppFilter::Alphabetical] {
                Button {
                    variant: if filter() == item { ButtonVariant::Default } else { ButtonVariant::Ghost },
                    size: ButtonSize::Sm,
                    class: "font-mono font-semibold uppercase tracking-wider",
                    action_label: filter_label(item),
                    onclick: move |_| {
                        selected_index.set(0);
                        filter.set(item);
                        pending_focus_index.set(Some(0));
                    },
                    {filter_label(item)}
                }
            }
        }
    }
}

#[component]
pub fn AppsLoading() -> Element {
    rsx! {
        CardGridViewport {
            CardGrid { columns: APP_GRID_COLUMNS, class: "py-6 sm:py-8 md:py-10",
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
            div { class: "flex h-full items-center justify-center",
                StatusAlert {
                    title: Some("Apps unavailable".to_string()),
                    message: "Wolf returned an unsuccessful app response. Try again once the service is ready."
                        .to_string(),
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
            div { class: "flex h-full items-center justify-center",
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
        CardGridViewport {
            CardGrid { columns: APP_GRID_COLUMNS, class: "py-6 sm:py-8 md:py-10",
                for (index, app) in apps.iter().cloned().enumerate() {
                    AppCard {
                        key: "{app.id}",
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
