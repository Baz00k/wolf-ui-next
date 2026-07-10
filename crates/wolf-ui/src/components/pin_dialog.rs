use dioxus::prelude::Key;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons::{HiCheck, HiLockClosed};

use crate::components::primitives::{
    Button, ButtonSize, ButtonVariant, CardContent, CardFooter, Numpad,
};
use crate::components::{ActionDialog, DialogCancelButton};
use crate::input::{UiCommand, use_ui_action};

const MAX_PIN_DIGITS: usize = 8;

#[component]
pub fn PinInputDialog(
    title: String,
    #[props(default = "Enter PIN".to_string())] description: String,
    #[props(default = "Unlock".to_string())] submit_label: String,
    onsubmit: EventHandler<Vec<i64>>,
    oncancel: EventHandler<()>,
) -> Element {
    let mut pin = use_signal(Vec::<i64>::new);
    let close_actions = use_ui_action(UiCommand::Cancel, "Cancel", move || oncancel.call(()));
    let onkeydown = move |event: KeyboardEvent| match event.key() {
        Key::Character(value) => {
            if let Some(digit) = value
                .chars()
                .next()
                .filter(|_| value.len() == 1)
                .and_then(|value| value.to_digit(10))
                .map(i64::from)
                .filter(|digit| (0..=9).contains(digit))
            {
                event.prevent_default();
                push_digit(pin, digit);
            }
        }
        Key::Backspace => {
            event.prevent_default();
            pin.write().pop();
        }
        _ => {}
    };

    rsx! {
        ActionDialog {
            title,
            description,
            scope_actions: close_actions,
            class: "max-w-md",
            CardContent { class: "space-y-4 sm:space-y-5",
                div { onkeydown,
                    PinDisplay { len: pin().len() }
                    Numpad {
                        ondigit: move |digit| push_digit(pin, digit),
                        onbackspace: move |_| {
                            pin.write().pop();
                        },
                        onclear: move |_| pin.write().clear(),
                    }
                }
            }
            CardFooter { class: "grid grid-cols-2 gap-3",
                DialogCancelButton { disabled: false, onclick: move |_| oncancel.call(()) }
                Button {
                    variant: ButtonVariant::Default,
                    size: ButtonSize::Xl,
                    class: "w-full",
                    action_label: submit_label.clone(),
                    disabled: pin().is_empty(),
                    onclick: move |_| submit_pin(pin(), onsubmit),
                    Icon {
                        icon: HiCheck,
                        class: "h-5 w-5",
                        width: None,
                        height: None,
                    }
                    "{submit_label}"
                }
            }
        }
    }
}

fn push_digit(mut pin: Signal<Vec<i64>>, digit: i64) {
    if pin.peek().len() < MAX_PIN_DIGITS {
        pin.write().push(digit);
    }
}

fn submit_pin(pin: Vec<i64>, onsubmit: EventHandler<Vec<i64>>) {
    if !pin.is_empty() {
        onsubmit.call(pin);
    }
}

#[component]
pub fn PinProtectQuestionDialog(
    onanswer: EventHandler<bool>,
    oncancel: EventHandler<()>,
) -> Element {
    let close_actions = use_ui_action(UiCommand::Cancel, "Cancel", move || oncancel.call(()));

    rsx! {
        ActionDialog { title: "Protect lobby?".to_string(), scope_actions: close_actions,
            CardContent { class: "space-y-4",
                p { class: "text-base font-medium leading-7 text-muted-foreground",
                    "Add a PIN so other players must enter it before joining this co-op lobby."
                }
            }
            CardFooter { class: "grid grid-cols-2 gap-3",
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::Xl,
                    class: "w-full",
                    action_label: "No PIN".to_string(),
                    autofocus: true,
                    onclick: move |_| onanswer.call(false),
                    "No PIN"
                }
                Button {
                    variant: ButtonVariant::Default,
                    size: ButtonSize::Xl,
                    class: "w-full",
                    action_label: "Add PIN".to_string(),
                    onclick: move |_| onanswer.call(true),
                    Icon {
                        icon: HiLockClosed,
                        class: "h-5 w-5",
                        width: None,
                        height: None,
                    }
                    "Add PIN"
                }
            }
        }
    }
}

#[component]
fn PinDisplay(len: usize) -> Element {
    rsx! {
        div { class: "overflow-hidden rounded-2xl border border-border bg-background/70 px-6 py-4 text-center shadow-inner shadow-black/20 sm:py-5",
            div { class: "mx-auto flex max-w-full items-center justify-center gap-3 sm:gap-4",
                for index in 0..MAX_PIN_DIGITS {
                    span { class: if index < len { "h-3 w-3 rounded-full bg-foreground sm:h-4 sm:w-4" } else { "h-3 w-3 rounded-full border border-muted-foreground/40 sm:h-4 sm:w-4" } }
                }
            }
            p { class: "mt-2 text-sm font-medium text-muted-foreground sm:mt-3",
                "{len}/{MAX_PIN_DIGITS} digits"
            }
            p { class: "mt-1 text-xs font-medium text-muted-foreground",
                "Use the keypad, keyboard, or controller"
            }
        }
    }
}
