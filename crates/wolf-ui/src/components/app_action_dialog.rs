use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons::{
    HiDownload, HiPlay, HiRefresh, HiStop, HiUserGroup,
};

use crate::components::app_card::AppCardData;
use crate::components::pin_dialog::{PinInputDialog, PinProtectQuestionDialog};
use crate::components::primitives::{
    CardContent, CardFooter, ProgressPanel, ProgressTone, ToastContext, ToastOptions, use_toasts,
};
use crate::components::{ActionDialog, ActionDialogItem, DialogCancelButton};
use crate::input::{UiCommand, use_ui_action};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppAction {
    Start,
    Connect,
    StartCoop,
    Stop,
    CheckUpdate,
    Download,
}

type ActionRunner = Action<
    (
        AppCardData,
        AppAction,
        Option<Vec<i64>>,
        Signal<Option<f64>>,
    ),
    bool,
>;

#[derive(Clone, Copy)]
struct DialogActionContext {
    loading_action: Signal<Option<AppAction>>,
    progress: Signal<Option<f64>>,
    action_runner: ActionRunner,
    toasts: ToastContext,
    onclose: EventHandler<()>,
}

#[component]
pub fn AppActionDialog(
    app: AppCardData,
    actions: Vec<AppAction>,
    action_runner: ActionRunner,
    onclose: EventHandler<()>,
) -> Element {
    let loading_action = use_signal(|| None::<AppAction>);
    let progress = use_signal(|| None::<f64>);
    let mut pin_prompt = use_signal(|| None::<PinPrompt>);
    let is_loading = loading_action().is_some();
    let progress_value = progress().unwrap_or(0.0).round() as u8;
    let progress_action = loading_action().filter(|action| shows_progress(*action));
    let toasts = use_toasts();
    let close_actions = use_ui_action(UiCommand::Cancel, "Cancel", move || {
        if loading_action().is_none() {
            onclose.call(());
        }
    });
    let ctx = DialogActionContext {
        loading_action,
        progress,
        action_runner,
        toasts,
        onclose,
    };

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
                                request_or_start_action(app.clone(), action, pin_prompt, ctx);
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
                DialogCancelButton { disabled: is_loading, onclick: move |_| onclose.call(()) }
            }
        }
        if let Some(prompt) = pin_prompt() {
            match prompt {
                PinPrompt::ProtectCoop => rsx! {
                    PinProtectQuestionDialog {
                        onanswer: move |protect| {
                            if protect {
                                pin_prompt.set(Some(PinPrompt::EnterPin(AppAction::StartCoop)));
                            } else {
                                pin_prompt.set(None);
                                start_dialog_action(app.clone(), AppAction::StartCoop, None, ctx);
                            }
                        },
                        oncancel: move |_| pin_prompt.set(None),
                    }
                },
                PinPrompt::EnterPin(action) => rsx! {
                    PinInputDialog {
                        title: pin_title(action).to_string(),
                        description: pin_description(action).to_string(),
                        submit_label: pin_submit_label(action).to_string(),
                        onsubmit: move |pin| {
                            pin_prompt.set(None);
                            start_dialog_action(app.clone(), action, Some(pin), ctx);
                        },
                        oncancel: move |_| {
                            if action == AppAction::StartCoop {
                                pin_prompt.set(Some(PinPrompt::ProtectCoop));
                            } else {
                                pin_prompt.set(None);
                            }
                        },
                    }
                },
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PinPrompt {
    ProtectCoop,
    EnterPin(AppAction),
}

fn request_or_start_action(
    app: AppCardData,
    action: AppAction,
    mut pin_prompt: Signal<Option<PinPrompt>>,
    ctx: DialogActionContext,
) {
    if ctx.loading_action.read().is_some() {
        return;
    }

    if action == AppAction::StartCoop {
        pin_prompt.set(Some(PinPrompt::ProtectCoop));
        return;
    }

    start_dialog_action(app, action, None, ctx);
}

fn start_dialog_action(
    app: AppCardData,
    action: AppAction,
    pin: Option<Vec<i64>>,
    ctx: DialogActionContext,
) {
    let DialogActionContext {
        mut loading_action,
        mut progress,
        mut action_runner,
        mut toasts,
        onclose,
    } = ctx;

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
        action_runner.call(app, action, pin, progress).await;
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
            Err(error) => toasts.show(error_message(action, &error), ToastOptions::error()),
        }

        onclose.call(());
    });
}

#[component]
fn ActionIcon(action: AppAction) -> Element {
    match action {
        AppAction::Start | AppAction::Connect => rsx! {
            Icon {
                icon: HiPlay,
                class: "h-5 w-5 text-emerald-400",
                width: None,
                height: None,
            }
        },
        AppAction::StartCoop => rsx! {
            Icon {
                icon: HiUserGroup,
                class: "h-5 w-5 text-blue-400",
                width: None,
                height: None,
            }
        },
        AppAction::Stop => rsx! {
            Icon {
                icon: HiStop,
                class: "h-5 w-5 text-red-400",
                width: None,
                height: None,
            }
        },
        AppAction::CheckUpdate => rsx! {
            Icon {
                icon: HiRefresh,
                class: "h-5 w-5 text-yellow-300",
                width: None,
                height: None,
            }
        },
        AppAction::Download => rsx! {
            Icon {
                icon: HiDownload,
                class: "h-5 w-5 text-yellow-300",
                width: None,
                height: None,
            }
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
        AppAction::CheckUpdate if downloaded => Some("Image updated."),
        AppAction::CheckUpdate if !downloaded => Some("Image is up to date."),
        AppAction::Download => Some("Image downloaded."),
        _ => None,
    }
}

fn error_message(action: AppAction, error: &impl std::fmt::Display) -> String {
    let message = error.to_string();
    if message.eq_ignore_ascii_case("Request timed out") {
        return format!("{} timed out. Try again.", action_label(action));
    }

    if matches!(action, AppAction::StartCoop)
        && message.eq_ignore_ascii_case("Lobby or session not found")
    {
        return "Lobby or session not found. Refresh and try again.".to_string();
    }

    format!("{} failed. Try again.", action_label(action))
}

fn pin_title(action: AppAction) -> &'static str {
    match action {
        AppAction::StartCoop => "Choose lobby PIN",
        _ => "PIN required",
    }
}

fn pin_description(action: AppAction) -> &'static str {
    match action {
        AppAction::StartCoop => "Players need this PIN to join",
        _ => "Enter PIN",
    }
}

fn pin_submit_label(action: AppAction) -> &'static str {
    match action {
        AppAction::StartCoop => "Create",
        _ => "Submit",
    }
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
