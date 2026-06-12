use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons::{HiPlay, HiStop, HiX};
use wolf_api::lobbies::Lobby;

use crate::components::primitives::{
    Button, ButtonSize, ButtonVariant, Card, CardContent, CardFooter, CardHeader, Dialog,
    DialogDescription, DialogHeader, DialogTitle, Spinner, ToastContext, ToastOptions, use_toasts,
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
        Dialog { scope_actions: close_actions,
            Card { class: "w-full max-w-lg overflow-hidden shadow-black/50".to_string(),
                CardHeader {
                    DialogHeader {
                        DialogTitle { "{lobby.name}" }
                        DialogDescription { "Select lobby action" }
                    }
                }
                CardContent { class: "space-y-3".to_string(),
                    LobbyActionRow {
                        label: "Join".to_string(),
                        tone: "text-emerald-400".to_string(),
                        autofocus: true,
                        action: LobbyAction::Join,
                        loading: loading_action() == Some(LobbyAction::Join),
                        disabled: is_loading,
                        onselect: {
                            let lobby_id = lobby.id.clone();
                            move |action| run_lobby_action(lobby_id.clone(), action, action_context)
                        },
                    }
                    LobbyActionRow {
                        label: "Stop".to_string(),
                        tone: "text-red-400".to_string(),
                        action: LobbyAction::Stop,
                        loading: loading_action() == Some(LobbyAction::Stop),
                        disabled: is_loading,
                        onselect: {
                            let lobby_id = lobby.id.clone();
                            move |action| run_lobby_action(lobby_id.clone(), action, action_context)
                        },
                    }
                }
                CardFooter {
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Xl,
                        class: "w-full text-muted-foreground focus:bg-accent focus:text-accent-foreground".to_string(),
                        action_label: "Cancel".to_string(),
                        disabled: is_loading,
                        onclick: move |_| onclose.call(()),
                        Icon { icon: HiX, class: "mr-2 h-5 w-5", width: None, height: None }
                        "Cancel"
                    }
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

#[component]
fn LobbyActionRow(
    label: String,
    tone: String,
    #[props(default)] autofocus: bool,
    action: LobbyAction,
    loading: bool,
    disabled: bool,
    onselect: EventHandler<LobbyAction>,
) -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Ghost,
            size: ButtonSize::Xl,
            class: "w-full justify-start border border-transparent text-left hover:border-foreground/30 focus:border-foreground focus:bg-accent focus:text-accent-foreground".to_string(),
            action_label: label.clone(),
            autofocus,
            disabled,
            onclick: move |_| onselect.call(action),
            if loading {
                Spinner { class: "h-5 w-5".to_string() }
            } else {
                LobbyActionIcon { action, tone: tone.clone() }
            }
            "{label}"
        }
    }
}

#[component]
fn LobbyActionIcon(action: LobbyAction, tone: String) -> Element {
    match action {
        LobbyAction::Join => rsx! {
            Icon { icon: HiPlay, class: "h-5 w-5 {tone}", width: None, height: None }
        },
        LobbyAction::Stop => rsx! {
            Icon { icon: HiStop, class: "h-5 w-5 {tone}", width: None, height: None }
        },
    }
}

fn lobby_error_message(action: LobbyAction) -> &'static str {
    match action {
        LobbyAction::Join => "Lobby join failed. Try again.",
        LobbyAction::Stop => "Lobby stop failed. Try again.",
    }
}
