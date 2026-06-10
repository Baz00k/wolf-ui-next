use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons::{
    HiDownload, HiPlay, HiRefresh, HiStop, HiUserGroup, HiX,
};

use crate::components::app_card::AppCardData;
use crate::components::primitives::{
    Button, ButtonSize, ButtonVariant, Card, CardContent, CardFooter, CardHeader, Dialog,
    DialogDescription, DialogHeader, DialogTitle, Spinner, ToastContext, ToastOptions, use_toasts,
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
        Dialog { label: "Actions for {app.title}".to_string(), scope_actions: close_actions,
            Card { class: "w-full max-w-lg overflow-hidden shadow-black/50".to_string(),
                CardHeader {
                    DialogHeader {
                        DialogTitle { "{app.title}" }
                        DialogDescription { "Select action" }
                    }
                }
                CardContent { class: "space-y-3".to_string(),
                    for (index, action) in actions.iter().copied().enumerate() {
                        ActionRow {
                            label: action_label(action).to_string(),
                            tone: action_tone(action).to_string(),
                            autofocus: index == 0,
                            action,
                            loading: loading_action() == Some(action),
                            disabled: is_loading,
                            onselect: {
                                let app = app.clone();
                                move |action| {
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
                        }
                    }
                    ActionStatus { action: progress_action, progress: progress_value }
                }
                CardFooter {
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Lg,
                        class: "h-14 w-full rounded-2xl text-base text-muted-foreground focus:bg-accent focus:text-accent-foreground".to_string(),
                        action_label: "Cancel".to_string(),
                        disabled: is_loading,
                        onclick: move |_| onclose.call(()),
                        Icon { icon: HiX, class: "mr-2 h-5 w-5", width: None, height: None, title: Some("Cancel".to_string()) }
                        "Cancel"
                    }
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
fn ActionStatus(action: Option<AppAction>, progress: u8) -> Element {
    rsx! {
        div { class: "h-16 pt-2".to_string(),
            if let Some(action) = action {
                div { class: "rounded-2xl border border-yellow-300/30 bg-yellow-300/10 px-4 py-3".to_string(),
                    div { class: "flex items-center justify-between gap-4 text-sm font-bold".to_string(),
                        span { class: "truncate text-yellow-100".to_string(), "{progress_message(action)}" }
                        span { class: "shrink-0 tabular-nums text-yellow-200".to_string(), "{progress}%" }
                    }
                    div { class: "mt-2 h-1.5 overflow-hidden rounded-full bg-background/70".to_string(),
                        div {
                            class: "h-full rounded-full bg-yellow-300 transition-[width] duration-300".to_string(),
                            style: "width: {progress}%;",
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ActionRow(
    label: String,
    tone: String,
    autofocus: bool,
    action: AppAction,
    loading: bool,
    disabled: bool,
    onselect: EventHandler<AppAction>,
) -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Ghost,
            size: ButtonSize::Lg,
            class: "h-16 w-full justify-start rounded-2xl border border-transparent px-5 text-left text-lg font-bold hover:border-foreground/30 focus:border-foreground focus:bg-accent focus:text-accent-foreground".to_string(),
            action_label: label.clone(),
            autofocus,
            disabled,
            onclick: move |_| onselect.call(action),
            if loading {
                Spinner { class: "h-5 w-5".to_string() }
            } else {
                ActionIcon { action, tone: tone.clone(), title: label.clone() }
            }
            "{label}"
        }
    }
}

#[component]
fn ActionIcon(action: AppAction, tone: String, title: String) -> Element {
    match action {
        AppAction::Start => rsx! {
            Icon { icon: HiPlay, class: "h-5 w-5 {tone}", width: None, height: None, title: Some(title) }
        },
        AppAction::Connect => rsx! {
            Icon { icon: HiPlay, class: "h-5 w-5 {tone}", width: None, height: None, title: Some(title) }
        },
        AppAction::StartCoop => rsx! {
            Icon { icon: HiUserGroup, class: "h-5 w-5 {tone}", width: None, height: None, title: Some(title) }
        },
        AppAction::Stop => rsx! {
            Icon { icon: HiStop, class: "h-5 w-5 {tone}", width: None, height: None, title: Some(title) }
        },
        AppAction::CheckUpdate => rsx! {
            Icon { icon: HiRefresh, class: "h-5 w-5 {tone}", width: None, height: None, title: Some(title) }
        },
        AppAction::Download => rsx! {
            Icon { icon: HiDownload, class: "h-5 w-5 {tone}", width: None, height: None, title: Some(title) }
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

fn action_tone(action: AppAction) -> &'static str {
    match action {
        AppAction::Start | AppAction::Connect => "text-emerald-400",
        AppAction::StartCoop => "text-blue-400",
        AppAction::Stop => "text-red-400",
        AppAction::CheckUpdate | AppAction::Download => "text-yellow-300",
    }
}
