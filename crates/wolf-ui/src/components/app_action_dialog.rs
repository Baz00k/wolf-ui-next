use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons::{
    HiDownload, HiPlay, HiRefresh, HiStop, HiUserGroup,
};

use crate::components::app_card::AppCardData;
use crate::components::primitives::{
    ActionDialog, ActionDialogItem, CardContent, CardFooter, DialogCancelButton, ProgressPanel,
    ProgressTone, ToastContext, ToastOptions, use_toasts,
};
use crate::input::{UiAction, use_ui_action};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppAction {
    Start,
    Connect,
    StartCoop,
    Stop,
    CheckUpdate,
    Download,
}

#[component]
pub fn AppActionDialog(
    app: AppCardData,
    actions: Vec<AppAction>,
    mut action_runner: Action<(AppCardData, AppAction, Signal<Option<f64>>), bool>,
    onclose: EventHandler<()>,
) -> Element {
    let loading_action = use_signal(|| None::<AppAction>);
    let progress = use_signal(|| None::<f64>);
    let is_loading = loading_action().is_some();
    let progress_value = progress().unwrap_or(0.0).round() as u8;
    let progress_action = loading_action().filter(|action| shows_progress(*action));
    let toasts = use_toasts();
    let close_actions = use_ui_action(UiAction::Cancel, "Cancel", move || {
        if loading_action().is_none() {
            onclose.call(());
        }
    });

    rsx! {
        ActionDialog {
            title: app.title.clone(),
            description: "Select action".to_string(),
            scope_actions: close_actions,
            CardContent { class: "space-y-3",
                for (index, action) in actions.iter().copied().enumerate() {
                    ActionDialogItem {
                        label: action_label(action),
                        autofocus: index == 0,
                        loading: loading_action() == Some(action),
                        disabled: is_loading,
                        onclick: {
                            let app = app.clone();
                            move |_| {
                                start_dialog_action(
                                    app.clone(),
                                    action,
                                    loading_action,
                                    progress,
                                    action_runner,
                                    toasts,
                                    onclose,
                                );
                            }
                        },
                        ActionIcon { action }
                    }
                }
                div { class: "h-16 pt-2",
                    if let Some(action) = progress_action {
                        ProgressPanel {
                            label: progress_message(action),
                            progress: progress_value,
                            tone: ProgressTone::Warning,
                        }
                    }
                }
            }
            CardFooter {
                DialogCancelButton {
                    disabled: is_loading,
                    onclick: move |_| onclose.call(()),
                }
            }
        }
    }
}

fn start_dialog_action(
    app: AppCardData,
    action: AppAction,
    mut loading_action: Signal<Option<AppAction>>,
    mut progress: Signal<Option<f64>>,
    mut action_runner: Action<(AppCardData, AppAction, Signal<Option<f64>>), bool>,
    mut toasts: ToastContext,
    onclose: EventHandler<()>,
) {
    if loading_action().is_some() {
        return;
    }

    loading_action.set(Some(action));
    progress.set(if shows_progress(action) {
        Some(0.0)
    } else {
        None
    });
    action_runner.reset();

    spawn(async move {
        action_runner.call(app, action, progress).await;
        let result = action_runner
            .value()
            .unwrap_or_else(|| Err(std::io::Error::other("Action did not complete.").into()));
        loading_action.set(None);
        progress.set(None);

        match result {
            Ok(downloaded) => {
                if let Some(message) = success_message(action, downloaded()) {
                    toasts.show(message, None);
                }
            }
            Err(_) => toasts.show(error_message(action), ToastOptions::error()),
        }

        onclose.call(());
    });
}

#[component]
fn ActionIcon(action: AppAction) -> Element {
    match action {
        AppAction::Start | AppAction::Connect => rsx! {
            Icon { icon: HiPlay, class: "h-5 w-5 text-emerald-400", width: None, height: None }
        },
        AppAction::StartCoop => rsx! {
            Icon { icon: HiUserGroup, class: "h-5 w-5 text-blue-400", width: None, height: None }
        },
        AppAction::Stop => rsx! {
            Icon { icon: HiStop, class: "h-5 w-5 text-red-400", width: None, height: None }
        },
        AppAction::CheckUpdate => rsx! {
            Icon { icon: HiRefresh, class: "h-5 w-5 text-yellow-300", width: None, height: None }
        },
        AppAction::Download => rsx! {
            Icon { icon: HiDownload, class: "h-5 w-5 text-yellow-300", width: None, height: None }
        },
    }
}

fn action_label(action: AppAction) -> &'static str {
    match action {
        AppAction::Start => "Start",
        AppAction::Connect => "Connect",
        AppAction::StartCoop => "Start Co-op",
        AppAction::Stop => "Stop",
        AppAction::CheckUpdate => "Check for Update",
        AppAction::Download => "Download",
    }
}

fn success_message(action: AppAction, downloaded: bool) -> Option<&'static str> {
    match action {
        AppAction::CheckUpdate if !downloaded => Some("Image is up to date."),
        _ => None,
    }
}

fn error_message(action: AppAction) -> String {
    format!("{} failed. Try again.", action_label(action))
}

fn progress_message(action: AppAction) -> &'static str {
    match action {
        AppAction::CheckUpdate => "Updating image",
        AppAction::Download => "Downloading image",
        _ => "Working",
    }
}

fn shows_progress(action: AppAction) -> bool {
    matches!(action, AppAction::CheckUpdate | AppAction::Download)
}
