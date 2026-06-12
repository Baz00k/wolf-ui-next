use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons::{HiPlay, HiStop};
use wolf_api::lobbies::Lobby;

use crate::components::primitives::{
    ActionDialog, ActionDialogItem, CardContent, CardFooter, DialogCancelButton, ToastContext,
    ToastOptions, use_toasts,
};
use crate::input::{UiAction, use_ui_action};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LobbyAction {
    Join,
    Stop,
}

#[component]
pub fn LobbyActionDialog(
    lobby: Lobby,
    mut join_runner: Action<(String,), bool>,
    mut stop_runner: Action<(String,), bool>,
    onstopped: EventHandler<String>,
    onclose: EventHandler<()>,
) -> Element {
    let loading_action = use_signal(|| None::<LobbyAction>);
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
                        move |_| run_lobby_action(lobby_id.clone(), LobbyAction::Join, action_context)
                    },
                    Icon { icon: HiPlay, class: "h-5 w-5 text-emerald-400", width: None, height: None }
                }
                ActionDialogItem {
                    label: "Stop",
                    loading: loading_action() == Some(LobbyAction::Stop),
                    disabled: is_loading,
                    onclick: {
                        let lobby_id = lobby.id.clone();
                        move |_| run_lobby_action(lobby_id.clone(), LobbyAction::Stop, action_context)
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
    }
}

#[derive(Clone, Copy)]
struct LobbyActionContext {
    loading_action: Signal<Option<LobbyAction>>,
    join_runner: Action<(String,), bool>,
    stop_runner: Action<(String,), bool>,
    toasts: ToastContext,
    onstopped: EventHandler<String>,
    onclose: EventHandler<()>,
}

fn run_lobby_action(lobby_id: String, action: LobbyAction, context: LobbyActionContext) {
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
        let result = match action {
            LobbyAction::Join => {
                join_runner.call(lobby_id).await;
                join_runner.value().unwrap_or_else(|| {
                    Err(std::io::Error::other("Lobby join did not complete.").into())
                })
            }
            LobbyAction::Stop => {
                stop_runner.call(lobby_id).await;
                stop_runner.value().unwrap_or_else(|| {
                    Err(std::io::Error::other("Lobby stop did not complete.").into())
                })
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
            Err(_) => toasts.show(lobby_error_message(action), ToastOptions::error()),
        }

        onclose.call(());
    });
}

fn lobby_error_message(action: LobbyAction) -> &'static str {
    match action {
        LobbyAction::Join => "Lobby join failed. Try again.",
        LobbyAction::Stop => "Lobby stop failed. Try again.",
    }
}
