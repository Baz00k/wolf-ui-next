use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons::{HiDownload, HiPlay, HiRefresh, HiUserGroup, HiX};

use crate::components::{
    Button, ButtonSize, ButtonVariant, Card, CardContent, CardFooter, CardHeader, Dialog,
    DialogDescription, DialogHeader, DialogTitle, app_card::AppCardData,
};
use crate::input::{UiAction, use_ui_action};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    Start,
    StartCoop,
    CheckUpdate,
    Download,
}

#[component]
pub fn AppActionDialog(
    app: AppCardData,
    actions: Vec<AppAction>,
    onselect: EventHandler<AppAction>,
    onclose: EventHandler<()>,
) -> Element {
    let close_actions = use_ui_action(UiAction::Cancel, "Cancel", move || onclose.call(()));

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
                            onselect,
                        }
                    }
                }
                CardFooter {
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Lg,
                        class: "h-14 w-full rounded-2xl text-base text-muted-foreground focus:bg-accent focus:text-accent-foreground".to_string(),
                        action_label: "Cancel".to_string(),
                        onclick: move |_| onclose.call(()),
                        Icon { icon: HiX, class: "mr-2 h-5 w-5", width: None, height: None, title: Some("Cancel".to_string()) }
                        "Cancel"
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
    onselect: EventHandler<AppAction>,
) -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Ghost,
            size: ButtonSize::Lg,
            class: "h-16 w-full justify-start rounded-2xl border border-transparent px-5 text-left text-lg font-bold hover:border-foreground/30 focus:border-foreground focus:bg-accent focus:text-accent-foreground".to_string(),
            action_label: label.clone(),
            autofocus,
            onclick: move |_| onselect.call(action),
            match action {
                AppAction::Start => rsx! {
                    Icon { icon: HiPlay, class: "h-5 w-5 {tone}", width: None, height: None, title: Some(label.clone()) }
                },
                AppAction::StartCoop => rsx! {
                    Icon { icon: HiUserGroup, class: "h-5 w-5 {tone}", width: None, height: None, title: Some(label.clone()) }
                },
                AppAction::CheckUpdate => rsx! {
                    Icon { icon: HiRefresh, class: "h-5 w-5 {tone}", width: None, height: None, title: Some(label.clone()) }
                },
                AppAction::Download => rsx! {
                    Icon { icon: HiDownload, class: "h-5 w-5 {tone}", width: None, height: None, title: Some(label.clone()) }
                },
            }
            "{label}"
        }
    }
}

fn action_label(action: AppAction) -> &'static str {
    match action {
        AppAction::Start => "Start",
        AppAction::StartCoop => "Start Co-op",
        AppAction::CheckUpdate => "Check for Update",
        AppAction::Download => "Download",
    }
}

fn action_tone(action: AppAction) -> &'static str {
    match action {
        AppAction::Start => "text-emerald-400",
        AppAction::StartCoop => "text-blue-400",
        AppAction::CheckUpdate | AppAction::Download => "text-yellow-300",
    }
}
