use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons::HiLogout;

use crate::components::primitives::{
    Button, ButtonSize, ButtonVariant, Card, CardContent, CardFooter, CardHeader, Dialog,
    DialogHeader, DialogTitle, Spinner, StatusAlert, StatusAlertVariant,
};
use crate::domain::session::stop_current_session;
use crate::input::{UiAction, use_ui_action};

#[component]
pub fn SessionShutdownControl() -> Element {
    let mut dialog_open = use_signal(|| false);
    let mut stop_session = use_action(stop_current_session);

    rsx! {
        Button {
            variant: ButtonVariant::Ghost,
            size: ButtonSize::Icon,
            class: "h-11 w-11 rounded-full border border-border/70 bg-card/70 text-muted-foreground shadow-lg shadow-black/20 backdrop-blur transition hover:border-destructive/60 hover:bg-destructive/15 hover:text-destructive-foreground focus:border-destructive/70 focus:bg-destructive/15 focus:text-destructive-foreground sm:h-12 sm:w-12".to_string(),
            action_label: "End session".to_string(),
            disabled: stop_session.pending(),
            onclick: move |_| dialog_open.set(true),
            Icon { icon: HiLogout, class: "h-5 w-5", width: None, height: None, title: None }
        }
        if dialog_open() {
            SessionShutdownDialog {
                stop_session,
                oncancel: move |_| {
                    stop_session.reset();
                    dialog_open.set(false);
                },
            }
        }
    }
}

#[component]
fn SessionShutdownDialog(mut stop_session: Action<(), ()>, oncancel: EventHandler<()>) -> Element {
    let pending = stop_session.pending();
    let failed = matches!(stop_session.value(), Some(Err(_)));
    let cancel_actions = use_ui_action(UiAction::Cancel, "Cancel", move || {
        if !pending {
            oncancel.call(());
        }
    });

    let confirm_label = if failed { "Retry" } else { "End session" };
    rsx! {
        Dialog { scope_actions: cancel_actions,
            Card { class: "w-full max-w-xl overflow-hidden shadow-black/50".to_string(),
                CardHeader {
                    DialogHeader {
                        DialogTitle { "End session?" }
                    }
                }
                CardContent { class: "space-y-4 px-6 py-6".to_string(),
                    p { class: "text-lg leading-8 text-muted-foreground",
                        "Are you sure you want to end this session? This will disconnect your stream."
                    }
                    if failed {
                        StatusAlert {
                            message: "The session could not be ended. Try again.".to_string(),
                            variant: StatusAlertVariant::Error,
                        }
                    }
                }
                CardFooter { class: "grid gap-3 sm:grid-cols-2".to_string(),
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Lg,
                        class: "h-14 rounded-2xl text-base text-muted-foreground focus:bg-accent focus:text-accent-foreground".to_string(),
                        action_label: "Cancel".to_string(),
                        disabled: pending,
                        onclick: move |_| oncancel.call(()),
                        "Cancel"
                    }
                    Button {
                        variant: ButtonVariant::Destructive,
                        size: ButtonSize::Lg,
                        class: "h-14 rounded-2xl text-base font-bold".to_string(),
                        action_label: confirm_label.to_string(),
                        autofocus: true,
                        disabled: pending,
                        onclick: move |_| {
                            let mut stop_session = stop_session;
                            spawn(async move {
                                stop_session.call().await;
                                if let Some(Err(error)) = stop_session.value() {
                                    tracing::warn!(target: "wolf-ui-session", "failed to end session: {error}");
                                }
                            });
                        },
                        if pending {
                            Spinner { class: "h-5 w-5".to_string() }
                        } else {
                            "{confirm_label}"
                        }
                    }
                }
            }
        }
    }
}
