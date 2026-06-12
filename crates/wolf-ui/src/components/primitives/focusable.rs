use dioxus::prelude::*;

use crate::input::{UiAction, native_action};

/// Gamepad-navigable interactive element.
///
/// Renders a router [`Link`] when `to` is set, otherwise a `button`. Owns the
/// shared focus plumbing: `data-focusable`, `data-actions` and autofocus.
#[component]
pub fn Focusable(
    #[props(default)] class: String,
    #[props(default)] to: Option<String>,
    #[props(default = "Select".to_string())] action_label: String,
    #[props(default)] index: Option<usize>,
    #[props(default)] autofocus: bool,
    #[props(default)] disabled: bool,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    #[props(default)] onfocus: Option<EventHandler<FocusEvent>>,
    children: Element,
) -> Element {
    let actions = native_action(UiAction::Accept, action_label);
    let index = index.map(|index| index.to_string());
    let handle_click = move |event| {
        if let Some(handler) = onclick {
            handler.call(event);
        }
    };
    let handle_mounted = move |event: MountedEvent| async move {
        if autofocus {
            let _ = event.data().set_focus(true).await;
        }
    };

    if let Some(to) = to {
        return rsx! {
            Link {
                to,
                class,
                "data-focusable": "true",
                "data-actions": actions,
                "data-grid-index": index,
                onclick: handle_click,
                onmounted: handle_mounted,
                {children}
            }
        };
    }

    rsx! {
        button {
            class,
            "data-focusable": "true",
            "data-actions": actions,
            "data-grid-index": index,
            disabled,
            onclick: handle_click,
            onfocus: move |event| {
                if let Some(handler) = onfocus {
                    handler.call(event);
                }
            },
            onmounted: handle_mounted,
            {children}
        }
    }
}
