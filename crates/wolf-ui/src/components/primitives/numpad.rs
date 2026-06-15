use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons::{HiBackspace, HiTrash};

use crate::components::primitives::{Button, ButtonSize, ButtonVariant};

#[component]
pub fn Numpad(
    ondigit: EventHandler<i64>,
    onbackspace: EventHandler<()>,
    onclear: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "mt-5 grid grid-cols-3 gap-3",
            for digit in 1..=9 {
                NumpadDigit { digit, autofocus: digit == 1, ondigit }
            }
            Button {
                variant: ButtonVariant::Menu,
                size: ButtonSize::Xl,
                class: "h-16 px-3 text-sm sm:text-base",
                action_label: "Delete digit".to_string(),
                onclick: move |_| onbackspace.call(()),
                Icon { icon: HiBackspace, class: "h-5 w-5", width: None, height: None }
                "Delete"
            }
            NumpadDigit { digit: 0, autofocus: false, ondigit }
            Button {
                variant: ButtonVariant::Menu,
                size: ButtonSize::Xl,
                class: "h-16 px-3 text-sm sm:text-base",
                action_label: "Clear input".to_string(),
                onclick: move |_| onclear.call(()),
                Icon { icon: HiTrash, class: "h-5 w-5", width: None, height: None }
                "Clear"
            }
        }
    }
}

#[component]
fn NumpadDigit(digit: i64, autofocus: bool, ondigit: EventHandler<i64>) -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Secondary,
            size: ButtonSize::Xl,
            class: "h-16 text-2xl font-bold",
            autofocus,
            action_label: format!("Enter {digit}"),
            onclick: move |_| ondigit.call(digit),
            "{digit}"
        }
    }
}
