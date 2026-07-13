use dioxus::prelude::*;

use crate::input::{UiAction, native_action};

/// Gamepad-navigable interactive element.
///
/// Renders a router [`Link`] when `to` is set, otherwise a `button`. Owns the
/// shared focus plumbing: `data-focusable`, `data-actions` and autofocus.
#[component]
pub fn Focusable(
    #[props(default)] class: String,
    #[props(default)] data_slot: Option<String>,
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
    let autofocus = autofocus.then_some("true");
    let handle_click = move |event: MouseEvent| {
        if disabled {
            event.prevent_default();
            return;
        }
        if let Some(handler) = onclick {
            handler.call(event);
        }
    };
    let handle_mounted = move |_: MountedEvent| {
        if autofocus.is_some() {
            let _ = document::eval("window.__wolfUiFocusAutofocus?.();");
        }
    };

    if let Some(to) = to {
        return rsx! {
            Link {
                to,
                class,
                "data-slot": data_slot.clone(),
                "data-focusable": (!disabled).then_some("true"),
                "data-autofocus": autofocus,
                "data-actions": actions,
                "data-grid-index": index,
                aria_disabled: disabled.then_some("true"),
                tabindex: disabled.then_some("-1"),
                onclick: handle_click,
                onmounted: handle_mounted,
                {children}
            }
        };
    }

    rsx! {
        button {
            r#type: "button",
            class,
            "data-slot": data_slot,
            "data-focusable": "true",
            "data-autofocus": autofocus,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn render(component: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(component);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn focusable_button_renders_focus_metadata() {
        let html = render(|| {
            rsx! {
                Focusable {
                    data_slot: "test-trigger",
                    action_label: "Launch",
                    index: 2,
                    autofocus: true,
                    "Play"
                }
            }
        });

        assert!(html.contains("<button"));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"data-slot="test-trigger""#));
        assert!(html.contains(r#"data-focusable="true""#));
        assert!(html.contains(r#"data-autofocus="true""#));
        assert!(html.contains(r#"data-grid-index="2""#));
        assert!(html.contains("Launch"));
    }
}
