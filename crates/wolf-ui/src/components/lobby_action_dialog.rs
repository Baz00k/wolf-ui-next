use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons::{HiPlay, HiStop};
use wolf_api::lobbies::Lobby;

use crate::components::primitives::{
    CardContent, CardFooter, ToastContext, ToastOptions, use_toasts,
};
use crate::components::{ActionDialog, ActionDialogItem, DialogCancelButton, PinInputDialog};
use crate::input::{UiAction, use_ui_action};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LobbyAction {
    Join,
    Stop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PinPrompt {
    /// Enter the lobby PIN, which is sent to the API.
    LobbyPin(LobbyAction),
    /// Enter the owner profile PIN, verified locally before stopping.
    ProfilePin,
}

#[component]
pub fn LobbyActionDialog(
    lobby: Lobby,
    owner_pin: Option<Vec<i64>>,
    mut join_runner: Action<(String, Option<Vec<i64>>), bool>,
    mut stop_runner: Action<(String, Option<Vec<i64>>), bool>,
    onstopped: EventHandler<String>,
    onclose: EventHandler<()>,
) -> Element {
    let loading_action = use_signal(|| None::<LobbyAction>);
    let mut pin_prompt = use_signal(|| None::<PinPrompt>);
    let is_loading = loading_action().is_some();
    let toasts = use_toasts();
    let action_context = LobbyActionContext {
        loading_action,
        join_runner,
        stop_runner,
        toasts,
        onstopped,
        onclose,
    };
    let close_actions = use_ui_action(UiAction::Cancel, "Cancel", move || {
        if loading_action().is_none() {
            onclose.call(());
        }
    });

    rsx! {
        ActionDialog {
            title: lobby.name.clone(),
            description: "Select lobby action".to_string(),
            scope_actions: close_actions,
            CardContent { class: "space-y-3",
                ActionDialogItem {
                    label: "Join",
                    autofocus: true,
                    loading: loading_action() == Some(LobbyAction::Join),
                    disabled: is_loading,
                    onclick: {
                        let lobby_id = lobby.id.clone();
                        move |_| request_or_run_lobby_action(lobby_id.clone(), lobby.pin_required, None, LobbyAction::Join, pin_prompt, action_context)
                    },
                    Icon { icon: HiPlay, class: "h-5 w-5 text-emerald-400", width: None, height: None }
                }
                ActionDialogItem {
                    label: "Stop",
                    loading: loading_action() == Some(LobbyAction::Stop),
                    disabled: is_loading,
                    onclick: {
                        let lobby_id = lobby.id.clone();
                        let owner_pin = owner_pin.clone();
                        move |_| request_or_run_lobby_action(lobby_id.clone(), lobby.pin_required, owner_pin.clone(), LobbyAction::Stop, pin_prompt, action_context)
                    },
                    Icon { icon: HiStop, class: "h-5 w-5 text-red-400", width: None, height: None }
                }
            }
            CardFooter {
                DialogCancelButton {
                    disabled: is_loading,
                    onclick: move |_| onclose.call(()),
                }
            }
        }
        match pin_prompt() {
            Some(PinPrompt::LobbyPin(action)) => rsx! {
                PinInputDialog {
                    title: "Lobby PIN required".to_string(),
                    description: lobby_pin_description(action).to_string(),
                    submit_label: lobby_pin_submit_label(action).to_string(),
                    onsubmit: {
                        let lobby_id = lobby.id.clone();
                        move |pin| {
                            pin_prompt.set(None);
                            run_lobby_action(lobby_id.clone(), action, Some(pin), action_context);
                        }
                    },
                    oncancel: move |_| pin_prompt.set(None),
                }
            },
            Some(PinPrompt::ProfilePin) => rsx! {
                PinInputDialog {
                    title: "PIN required".to_string(),
                    description: "Enter the profile PIN to stop it".to_string(),
                    submit_label: "Stop".to_string(),
                    onsubmit: {
                        let lobby_id = lobby.id.clone();
                        let owner_pin = owner_pin.clone();
                        let mut toasts = toasts;
                        move |pin: Vec<i64>| {
                            pin_prompt.set(None);
                            if owner_pin.as_ref() == Some(&pin) {
                                run_lobby_action(lobby_id.clone(), LobbyAction::Stop, None, action_context);
                            } else {
                                toasts.show("Incorrect PIN. Try again.", ToastOptions::error());
                            }
                        }
                    },
                    oncancel: move |_| pin_prompt.set(None),
                }
            },
            None => rsx! {},
        }
    }
}

#[derive(Clone, Copy)]
struct LobbyActionContext {
    loading_action: Signal<Option<LobbyAction>>,
    join_runner: Action<(String, Option<Vec<i64>>), bool>,
    stop_runner: Action<(String, Option<Vec<i64>>), bool>,
    toasts: ToastContext,
    onstopped: EventHandler<String>,
    onclose: EventHandler<()>,
}

fn request_or_run_lobby_action(
    lobby_id: String,
    pin_required: bool,
    owner_pin: Option<Vec<i64>>,
    action: LobbyAction,
    mut pin_prompt: Signal<Option<PinPrompt>>,
    context: LobbyActionContext,
) {
    if pin_required {
        pin_prompt.set(Some(PinPrompt::LobbyPin(action)));
        return;
    }

    if action == LobbyAction::Stop && owner_pin.is_some() {
        pin_prompt.set(Some(PinPrompt::ProfilePin));
        return;
    }

    run_lobby_action(lobby_id, action, None, context);
}

fn run_lobby_action(
    lobby_id: String,
    action: LobbyAction,
    pin: Option<Vec<i64>>,
    context: LobbyActionContext,
) {
    let LobbyActionContext {
        mut loading_action,
        mut join_runner,
        mut stop_runner,
        mut toasts,
        onstopped,
        onclose,
    } = context;

    if loading_action().is_some() {
        return;
    }

    loading_action.set(Some(action));
    join_runner.reset();
    stop_runner.reset();

    spawn(async move {
        let stopped_lobby_id = lobby_id.clone();
        let result: Result<bool, Option<String>> = match action {
            LobbyAction::Join => {
                join_runner.call(lobby_id, pin).await;
                match join_runner.value() {
                    Some(Ok(joined)) => Ok(joined()),
                    Some(Err(error)) => Err(Some(error.to_string())),
                    None => Err(None),
                }
            }
            LobbyAction::Stop => {
                stop_runner.call(lobby_id, pin).await;
                match stop_runner.value() {
                    Some(Ok(stopped)) => Ok(stopped()),
                    Some(Err(error)) => Err(Some(error.to_string())),
                    None => Err(None),
                }
            }
        };

        loading_action.set(None);

        match result {
            Ok(_) => {
                if action == LobbyAction::Stop {
                    onstopped.call(stopped_lobby_id);
                    toasts.show("Lobby stopped.", None);
                }
            }
            Err(error) => toasts.show(
                lobby_error_message(action, error.as_deref()),
                ToastOptions::error(),
            ),
        }

        onclose.call(());
    });
}

fn lobby_error_message(action: LobbyAction, error: Option<&str>) -> String {
    let Some(error) = error else {
        return format!(
            "{} did not complete. Try again.",
            lobby_action_label(action)
        );
    };

    if error.eq_ignore_ascii_case("Invalid PIN") {
        return "Incorrect PIN. Try again.".to_string();
    }

    if error.eq_ignore_ascii_case("Request timed out") {
        return format!("{} timed out. Try again.", lobby_action_label(action));
    }

    if error.eq_ignore_ascii_case("Lobby or session not found") {
        return "Lobby or session not found. Refresh and try again.".to_string();
    }

    format!("{} failed: {error}", lobby_action_label(action))
}

fn lobby_action_label(action: LobbyAction) -> &'static str {
    match action {
        LobbyAction::Join => "Lobby join",
        LobbyAction::Stop => "Lobby stop",
    }
}

fn lobby_pin_description(action: LobbyAction) -> &'static str {
    match action {
        LobbyAction::Join => "Enter the lobby PIN to join",
        LobbyAction::Stop => "Enter the lobby PIN to stop it",
    }
}

fn lobby_pin_submit_label(action: LobbyAction) -> &'static str {
    match action {
        LobbyAction::Join => "Join",
        LobbyAction::Stop => "Stop",
    }
}
