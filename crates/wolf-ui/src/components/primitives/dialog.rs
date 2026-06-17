use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons::HiX;
use tw_merge::tw_merge;

use crate::components::primitives::{Button, ButtonSize, ButtonVariant, Card, CardHeader, Spinner};

#[component]
pub fn Dialog(
    #[props(default)] class: String,
    #[props(default)] scope_actions: String,
    children: Element,
) -> Element {
    use_hook(capture_dialog_opener);
    use_drop(restore_dialog_opener);
    let class = tw_merge!(
        "fixed inset-0 z-100 flex items-center justify-center bg-black/75 backdrop-blur-sm px-5",
        class,
    );

    rsx! {
        div {
            class,
            "data-focus-scope": "true",
            "data-focus-trap": "true",
            "data-scope-actions": scope_actions,
            tabindex: "-1",
            {children}
        }
    }
}

fn capture_dialog_opener() {
    let _ = document::eval(
        r#"
        window.__wolfUiDialogRestoreElement = document.activeElement?.matches?.('[data-focusable="true"]')
            ? document.activeElement
            : null;
        "#,
    );
}

fn restore_dialog_opener() {
    let _ = document::eval(
        r#"
        requestAnimationFrame(() => requestAnimationFrame(() => {
            const element = window.__wolfUiDialogRestoreElement;
            window.__wolfUiDialogRestoreElement = null;

            if (element?.isConnected) {
                const style = window.getComputedStyle(element);
                const rect = element.getBoundingClientRect();
                const visible = style.display !== "none"
                    && style.visibility !== "hidden"
                    && !element.matches(':disabled,[aria-disabled="true"],[inert]')
                    && rect.width > 0
                    && rect.height > 0;

                if (visible && window.__wolfUiFocusElement?.(element, { inline: "nearest" })) {
                    return;
                }
            }

            window.__wolfUiEnsureFocusableActiveElement?.();
        }));
        "#,
    );
}

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

#[component]
pub fn DialogHeader(#[props(default)] class: String, children: Element) -> Element {
    let class = tw_merge!("min-w-0 flex-1", class);

    rsx! {
        div { class,
            {children}
        }
    }
}

#[component]
pub fn DialogTitle(#[props(default)] class: String, children: Element) -> Element {
    let class = tw_merge!("truncate text-2xl font-bold tracking-tight", class);

    rsx! {
        h2 { class,
            {children}
        }
    }
}

#[component]
pub fn DialogDescription(#[props(default)] class: String, children: Element) -> Element {
    let class = tw_merge!(
        "font-mono text-xs font-semibold uppercase tracking-widest text-muted-foreground",
        class,
    );

    rsx! {
        p { class,
            {children}
        }
    }
}
