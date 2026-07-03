use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons::HiLogout;

use crate::components::primitives::{
    Button, ButtonSize, ButtonVariant, CardContent, CardFooter, Spinner, StatusAlert,
    StatusAlertVariant,
};
use crate::components::{ActionDialog, DialogCancelButton};
use crate::domain::session::stop_current_session;
use crate::input::{UiAction, use_ui_action};

#[component]
pub fn SessionShutdownControl() -> Element {
    let mut dialog_open = use_signal(|| false);
    let mut stop_session = use_action(stop_current_session);

    rsx! {
        Button {
            variant: ButtonVariant::ChromeDestructive,
            size: ButtonSize::IconLg,
            action_label: "End session",
            disabled: stop_session.pending(),
            onclick: move |_| dialog_open.set(true),
            Icon {
                icon: HiLogout,
                class: "h-7 w-7 sm:h-8 sm:w-8",
                width: None,
                height: None,
                title: None,
            }
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
        ActionDialog {
            title: "End session?".to_string(),
            scope_actions: cancel_actions,
            class: "max-w-xl",
            CardContent { class: "space-y-4 px-6 py-6 min-h-48",
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
            CardFooter { class: "grid gap-3 sm:grid-cols-2",
                DialogCancelButton { disabled: pending, onclick: move |_| oncancel.call(()) }
                Button {
                    variant: ButtonVariant::Destructive,
                    size: ButtonSize::Xl,
                    action_label: confirm_label,
                    autofocus: true,
                    disabled: pending,
                    onclick: move |_| {
                        let mut stop_session = stop_session;
                        spawn(async move {
                            stop_session.call().await;
                            if let Some(Err(error)) = stop_session.value() {
                                tracing::warn!(
                                    target : "wolf-ui-session", "failed to end session: {error}"
                                );
                            }
                        });
                    },
                    if pending {
                        Spinner { class: "h-5 w-5" }
                    } else {
                        "{confirm_label}"
                    }
                }
            }
        }
    }
}
