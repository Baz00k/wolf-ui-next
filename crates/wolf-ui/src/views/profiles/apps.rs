use std::collections::HashMap;

use dioxus::prelude::*;
use wolf_api::lobbies::Lobby;
use wolf_api::profiles::AppListResponse;

use crate::components::{
    AppActionDialog, AppCard, AppCardData, AppCardSkeleton, AppStatusTone, Button, ButtonSize,
    StatusAlert, StatusAlertVariant,
};
use crate::domain::apps::{
    AppFilter, app_actions, filter_label, listen_for_lobby_events, load_apps_state,
    selected_app_data, sorted_apps,
};
use crate::input::{
    ActionHint, UiAction, UiHint, action_hint_from_json, action_hints, use_ui_action,
};

const LOADING_CARD_COUNT: usize = 5;
const APP_CAROUSEL_CLASS: &str =
    "w-full snap-x snap-mandatory overflow-x-auto overflow-y-visible [scrollbar-width:none]";
const APP_CAROUSEL_TRACK_CLASS: &str = "mx-auto flex min-w-max items-center gap-5 px-[calc(50vw-7rem)] py-16 md:px-[calc(50vw-8rem)] lg:gap-6 lg:px-[calc(50vw-9rem)] xl:gap-7 xl:px-[calc(50vw-10rem)] 2xl:px-[calc(50vw-12rem)]";

#[component]
pub fn ProfileApps(profile_id: String) -> Element {
    let resource_profile_id = profile_id.clone();
    let events_profile_id = profile_id.clone();
    let navigator = use_navigator();
    let selected_index = use_signal(|| 0usize);
    let filter = use_signal(|| AppFilter::Default);
    let action_app = use_signal(|| None::<AppCardData>);
    let mut lobbies = use_signal(Vec::<Lobby>::new);
    let mut pending_focus_index = use_signal(|| None::<usize>);

    use_effect(move || {
        if let Some(index) = pending_focus_index() {
            pending_focus_index.write().take();
            focus_app(index);
        }
    });

    let back_action = use_ui_action(UiAction::Cancel, "Back", move || {
        navigator.go_back();
    });
    let filter_focus_action = use_ui_action(UiAction::Menu, "Filters", move || {
        let _ = document::eval(
            "document.querySelector('[data-filter-button=\"true\"]')?.focus({ preventScroll: true });",
        );
    });
    let scope_actions = action_hints([
        action_hint_from_json(&back_action),
        action_hint_from_json(&filter_focus_action),
        ActionHint::new(UiHint::Navigate, "Navigate"),
    ]);
    let carousel_actions = action_hints([
        ActionHint::new(UiHint::Navigate, "Navigate"),
        action_hint_from_json(&back_action),
        action_hint_from_json(&filter_focus_action),
    ]);

    let mut apps = use_resource(move || {
        let profile_id = resource_profile_id.clone();

        async move { load_apps_state(profile_id).await }
    });

    use_effect(move || {
        if let Some(Ok(state)) = &*apps.read() {
            lobbies.set(state.lobbies.clone());
        }
    });

    use_effect(move || {
        let profile_id = events_profile_id.clone();
        spawn(async move {
            listen_for_lobby_events(profile_id, lobbies).await;
        });
    });

    rsx! {
        div { class: "min-h-screen overflow-hidden bg-background text-foreground",
            div { class: "pointer-events-none fixed inset-0 bg-[radial-gradient(circle_at_50%_50%,oklch(0.75_0.12_250/0.12),transparent_34%),linear-gradient(180deg,oklch(1_0_0/0.04),transparent_45%)]" }
            section {
                class: "relative flex min-h-screen flex-col px-6 pb-28 pt-8 sm:px-10 lg:px-16",
                "data-focus-scope": "true",
                "data-scope-actions": scope_actions,
                AppsHeader { filter, selected_index, pending_focus_index }
                div {
                    class: "-mx-6 flex min-h-0 flex-1 items-center justify-center overflow-visible sm:-mx-10 lg:-mx-16",
                    "data-focus-scope": "true",
                    "data-scope-actions": carousel_actions,
                    match &*apps.read_unchecked() {
                        Some(Ok(state)) => rsx! {
                            AppsContent {
                                response: AppListResponse { success: true, apps: state.apps.clone() },
                                lobbies: lobbies(),
                                images: state.images.clone(),
                                covers: state.covers.clone(),
                                filter: filter(),
                                selected_index,
                                action_app,
                            }
                        },
                        Some(Err(error)) => rsx! {
                            StatusAlert {
                                title: "Apps unavailable".to_string(),
                                message: format!("Wolf did not return the app list. {error}"),
                                variant: StatusAlertVariant::Error,
                                Button {
                                    size: ButtonSize::Lg,
                                    class: "rounded-full uppercase tracking-widest",
                                    onclick: move |_| apps.restart(),
                                    "Retry"
                                }
                            }
                        },
                        None => rsx! { AppsLoading {} },
                    }
                }
                match &*apps.read_unchecked() {
                    Some(Ok(state)) => rsx! {
                        SelectedAppMeta {
                            app: selected_app_data(&state.apps, &lobbies(), &state.images, &state.covers, filter(), selected_index()),
                        }
                    },
                    _ => rsx! {},
                }
            }
            if let Some(app) = action_app() {
                AppActionDialog {
                    actions: app_actions(&app),
                    app: app.clone(),
                    onselect: move |_| close_modal(action_app, selected_index()),
                    onclose: move |_| close_modal(action_app, selected_index()),
                }
            }
        }
    }
}

#[component]
fn AppsHeader(
    mut filter: Signal<AppFilter>,
    mut selected_index: Signal<usize>,
    mut pending_focus_index: Signal<Option<usize>>,
) -> Element {
    rsx! {
        header { class: "flex items-start justify-between gap-4",
            div {
                h1 { class: "text-2xl font-bold tracking-tight sm:text-4xl lg:text-5xl", "Applications" }
            }
            div { class: "rounded-full border border-border bg-card/70 p-1 font-mono text-xs font-semibold uppercase tracking-widest text-muted-foreground shadow-lg shadow-black/20",
                for item in [AppFilter::Default, AppFilter::Available, AppFilter::Alphabetical] {
                    button {
                        class: filter_chip_class(filter() == item),
                        "data-focusable": "true",
                        "data-filter-button": "true",
                        "aria-pressed": if filter() == item { "true" } else { "false" },
                        "data-actions": action_hints([ActionHint::new(UiHint::Accept, filter_label(item))]),
                        onclick: move |_| {
                            selected_index.set(0);
                            filter.set(item);
                            pending_focus_index.set(Some(0));
                        },
                        "{filter_label(item)}"
                    }
                }
            }
        }
    }
}

#[component]
fn AppsLoading() -> Element {
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
fn AppsContent(
    response: AppListResponse,
    lobbies: Vec<Lobby>,
    images: HashMap<String, bool>,
    covers: HashMap<String, String>,
    filter: AppFilter,
    mut selected_index: Signal<usize>,
    mut action_app: Signal<Option<AppCardData>>,
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

    let apps = sorted_apps(response.apps, &lobbies, &images, &covers, filter);
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
                                action_app.set(Some(app.clone()));
                                scroll_to_app(index);
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

#[component]
fn SelectedAppMeta(app: Option<AppCardData>) -> Element {
    let Some(app) = app else {
        return rsx! {};
    };
    let badge_class = status_badge_class(app.status.tone);

    rsx! {
        div { class: "pointer-events-none flex h-24 shrink-0 flex-col items-center justify-center gap-3 px-6 text-center lg:h-28",
            h2 { class: "max-w-[80vw] truncate text-3xl font-black tracking-tight md:text-4xl xl:text-5xl 2xl:text-6xl", "{app.title}" }
            div { class: "inline-flex items-center gap-3 rounded-full border px-4 py-2 font-mono text-xs font-bold uppercase tracking-widest shadow-2xl shadow-black/30 {badge_class}",
                span { class: "h-2 w-2 rounded-full bg-current shadow-[0_0_16px_currentColor]" }
                span { "{app.status.label}" }
            }
        }
    }
}

fn close_modal(mut action_app: Signal<Option<AppCardData>>, selected_index: usize) {
    action_app.set(None);
    focus_app(selected_index);
}

fn focus_app(index: usize) {
    let _ = document::eval(&format!(
        "requestAnimationFrame(() => requestAnimationFrame(() => window.__wolfUiFocusSelector?.('[data-app-index=\\\"{}\\\"]', {{ inline: 'center' }})));",
        index
    ));
}

fn scroll_to_app(index: usize) {
    let _ = document::eval(&format!(
        "window.__wolfUiScrollSelectorIntoHorizontalView?.('[data-app-index=\\\"{}\\\"]', {{ inline: 'center' }});",
        index
    ));
}

fn filter_chip_class(active: bool) -> &'static str {
    if active {
        "inline-flex rounded-full bg-primary px-3 py-2 text-primary-foreground outline-none ring-0 focus:ring-2 focus:ring-ring/50"
    } else {
        "inline-flex rounded-full px-3 py-2 outline-none ring-0 transition hover:bg-accent hover:text-accent-foreground focus:ring-2 focus:ring-ring/50"
    }
}

fn status_badge_class(tone: AppStatusTone) -> &'static str {
    match tone {
        AppStatusTone::Ready => "border-emerald-400/30 bg-emerald-400/10 text-emerald-300",
        AppStatusTone::Warning => "border-yellow-300/30 bg-yellow-300/10 text-yellow-200",
    }
}
