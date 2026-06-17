use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons::HiX;
use tw_merge::tw_merge;

use crate::components::primitives::{
    Button, ButtonSize, ButtonVariant, Card, CardHeader, Dialog, DialogDescription, DialogHeader,
    DialogTitle, Spinner,
};

/// Modal dialog with a titled card, used for action menus and confirmations.
#[component]
pub fn ActionDialog(
    title: String,
    #[props(default)] description: Option<String>,
    #[props(default)] scope_actions: String,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let class = tw_merge!("w-full max-w-lg overflow-hidden shadow-black/50", class);

    rsx! {
        Dialog { scope_actions,
            Card { class,
                CardHeader {
                    DialogHeader {
                        DialogTitle { "{title}" }
                        if let Some(description) = description {
                            DialogDescription { "{description}" }
                        }
                    }
                }
                {children}
            }
        }
    }
}

/// Menu row inside an [`ActionDialog`], with an icon slot and loading state.
#[component]
pub fn ActionDialogItem(
    label: String,
    #[props(default)] autofocus: bool,
    #[props(default)] loading: bool,
    #[props(default)] disabled: bool,
    onclick: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Menu,
            size: ButtonSize::Xl,
            class: "w-full justify-start",
            action_label: label.clone(),
            autofocus,
            disabled,
            onclick: move |event| onclick.call(event),
            if loading {
                Spinner { class: "h-5 w-5" }
            } else {
                {children}
            }
            "{label}"
        }
    }
}

/// Full-width dismiss button for dialog footers.
#[component]
pub fn DialogCancelButton(
    #[props(default = "Cancel".to_string())] label: String,
    #[props(default)] disabled: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Menu,
            size: ButtonSize::Xl,
            class: "w-full text-muted-foreground",
            action_label: label.clone(),
            disabled,
            onclick: move |event| onclick.call(event),
            Icon { icon: HiX, class: "h-5 w-5", width: None, height: None }
            "{label}"
        }
    }
}
