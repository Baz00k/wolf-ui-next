use std::time::Duration;

use dioxus::prelude::*;
use tokio::time::sleep;
use wolf_api::lobbies::Lobby;
use wolf_api::profiles::AppListResponse;

use crate::components::{
    AppActionDialog, AppCardData, AppsContent, AppsHeader, AppsLoading, Button, ButtonSize,
    SelectedAppMeta, StatusAlert, StatusAlertVariant,
};
use crate::domain::app_actions::{ActionStatuses, error_status, run_app_action};
use crate::domain::apps::{
    AppFilter, app_actions, listen_for_lobby_events, load_apps_state, selected_app_data,
};
use crate::input::{
    ActionHint, UiAction, UiHint, action_hint_from_json, action_hints, use_ui_action,
};

const ACTIVE_POLL_DELAY: Duration = Duration::from_secs(15);
const BACKGROUND_POLL_DELAY: Duration = Duration::from_mins(1);

#[component]
pub fn ProfileApps(profile_id: String) -> Element {
    let resource_profile_id = profile_id.clone();
    let events_profile_id = profile_id.clone();
    let navigator = use_navigator();
    let selected_index = use_signal(|| 0usize);
    let filter = use_signal(|| AppFilter::Default);
    let mut action_app = use_signal(|| None::<AppCardData>);
    let action_statuses = use_signal(ActionStatuses::new);
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
            "document.querySelector('#apps-filter button')?.focus({ preventScroll: true });",
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

    use_effect(move || {
        spawn(async move {
            loop {
                sleep(next_poll_delay().await).await;
                apps.restart();
            }
        });
    });

    rsx! {
        div { class: "min-h-screen overflow-hidden bg-background text-foreground",
            div { class: "pointer-events-none fixed inset-0 wolf-ambient-background-center" }
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
                                profile_id: profile_id.clone(),
                                lobbies: lobbies(),
                                images: state.images.clone(),
                                covers: state.covers.clone(),
                                filter: filter(),
                                selected_index,
                                on_app_click: move |(index, app)| {
                                    action_app.set(Some(app));
                                    scroll_to_app(index);
                                },
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
                            app: selected_app_data(&profile_id, &state.apps, &lobbies(), &state.images, &state.covers, filter(), selected_index()),
                            action_statuses: action_statuses(),
                        }
                    },
                    _ => rsx! {},
                }
            }
            if let Some(app) = action_app() {
                AppActionDialog {
                    actions: app_actions(&app),
                    app: app.clone(),
                    onselect: move |action| {
                        start_profile_app_action(profile_id.clone(), app.clone(), action, action_statuses, apps);
                        close_modal(action_app, selected_index());
                    },
                    onclose: move |_| close_modal(action_app, selected_index()),
                }
            }
        }
    }
}

fn start_profile_app_action(
    profile_id: String,
    app: AppCardData,
    action: crate::components::AppAction,
    mut action_statuses: Signal<ActionStatuses>,
    mut apps: Resource<Result<crate::domain::apps::AppsState, String>>,
) {
    let app_id = app.id.clone();
    action_statuses.write().remove(&app_id);
    spawn(async move {
        match run_app_action(profile_id, app, action, action_statuses).await {
            Ok(_) => {
                action_statuses.write().remove(&app_id);
                apps.restart();
            }
            Err(error) => {
                action_statuses
                    .write()
                    .insert(app_id.clone(), error_status(app_id, error));
                apps.restart();
            }
        }
    });
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

async fn next_poll_delay() -> Duration {
    let mut eval = document::eval(
        r#"
        dioxus.send(document.visibilityState === "visible" && document.hasFocus());
        "#,
    );
    match eval.recv::<bool>().await {
        Ok(true) => ACTIVE_POLL_DELAY,
        Ok(false) | Err(_) => BACKGROUND_POLL_DELAY,
    }
}
