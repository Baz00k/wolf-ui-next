use std::time::Duration;

use dioxus::prelude::*;
use tokio::time::sleep;
use wolf_api::lobbies::Lobby;
use wolf_api::profiles::AppListResponse;

use crate::components::primitives::{Button, ButtonSize, StatusAlert, StatusAlertVariant};
use crate::components::{
    AppAction, AppActionDialog, AppCardData, AppsContent, AppsHeader, AppsLoading,
};
use crate::domain::app_actions::run_app_action;
use crate::domain::apps::{AppFilter, app_actions, listen_for_lobby_events, load_apps_state};
use crate::input::{ActionHint, UiAction, UiHint, action_hints, use_ui_action_hint};

const ACTIVE_POLL_DELAY: Duration = Duration::from_secs(15);
const BACKGROUND_POLL_DELAY: Duration = Duration::from_mins(1);

#[component]
pub fn ProfileApps(profile_id: String) -> Element {
    let resource_profile_id = profile_id.clone();
    let events_profile_id = profile_id.clone();
    let selected_index = use_signal(|| 0usize);
    let filter = use_signal(|| AppFilter::Available);
    let mut action_app = use_signal(|| None::<AppCardData>);
    let mut lobbies = use_signal(Vec::<Lobby>::new);
    let mut pending_focus_index = use_signal(|| None::<usize>);

    use_effect(move || {
        if let Some(index) = pending_focus_index() {
            pending_focus_index.write().take();
            focus_app(index);
        }
    });

    let sort_focus_action = use_ui_action_hint(UiAction::Menu, "Sort", move || {
        let _ = document::eval(
            "document.querySelector('#apps-filter button')?.focus({ preventScroll: true });",
        );
    });
    let grid_actions = action_hints([
        ActionHint::new(UiHint::Navigate, "Navigate"),
        sort_focus_action,
    ]);
    let mut apps = use_resource(move || {
        let profile_id = resource_profile_id.clone();

        async move { load_apps_state(profile_id).await }
    });
    let action_profile_id = profile_id.clone();
    let action_runner = use_action(
        move |app: AppCardData,
              action: AppAction,
              pin: Option<Vec<i64>>,
              mut progress: Signal<Option<f64>>| {
            let profile_id = action_profile_id.clone();
            async move {
                let result = run_app_action(profile_id, app, action, pin, move |value| {
                    progress.set(Some(value));
                })
                .await
                .map_err(std::io::Error::other);
                apps.restart();
                result
            }
        },
    );

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
        div { class: "flex h-full min-h-0 flex-col bg-background px-6 pt-6 text-foreground sm:px-8 sm:pt-8 lg:px-12 lg:pt-10",
            AppsHeader { filter, selected_index, pending_focus_index }
            section {
                class: "relative min-h-0 flex-1 pt-4 sm:pt-5 lg:pt-6",
                div {
                    class: "h-full w-full",
                    "data-focus-scope": "true",
                    "data-focus-region": "main",
                    "data-scope-actions": grid_actions,
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
                                on_app_click: move |(index, app): (usize, AppCardData)| {
                                    action_app.set(Some(app));
                                    scroll_to_app(index);
                                },
                            }
                        },
                        Some(Err(message)) => rsx! {
                            StatusAlert {
                                title: Some("Apps unavailable".to_string()),
                                message: message.clone(),
                                variant: StatusAlertVariant::Error,
                                Button {
                                    size: ButtonSize::Lg,
                                    onclick: move |_| apps.restart(),
                                    "Retry"
                                }
                            }
                        },
                        None => rsx! { AppsLoading {} },
                    }
                }
            }
            if let Some(app) = action_app() {
                AppActionDialog {
                    actions: app_actions(&app),
                    app: app.clone(),
                    action_runner,
                    onclose: move |_| action_app.set(None),
                }
            }
        }
    }
}

fn focus_app(index: usize) {
    let _ = document::eval(&format!(
        "requestAnimationFrame(() => requestAnimationFrame(() => window.__wolfUiFocusSelector?.('[data-grid-index=\\\"{}\\\"]', {{ inline: 'nearest' }})));",
        index
    ));
}

fn scroll_to_app(index: usize) {
    let _ = document::eval(&format!(
        "window.__wolfUiScrollSelectorIntoView?.('[data-grid-index=\\\"{}\\\"]', {{ block: 'nearest', inline: 'nearest' }});",
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
