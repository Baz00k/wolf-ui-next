use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons::HiArrowLeft;

use crate::components::primitives::{Button, ButtonSize, ButtonVariant};

/// Pointer-friendly mirror of the global Cancel action (Esc / gamepad B).
///
/// Renders nothing when there is no history to go back to (the root screen).
#[component]
pub fn BackButton() -> Element {
    let navigator = use_navigator();

    if !navigator.can_go_back() {
        return rsx! {};
    }

    rsx! {
        Button {
            variant: ButtonVariant::Outline,
            size: ButtonSize::IconLg,
            class: "shrink-0",
            action_label: "Back",
            onclick: move |_| {
                if navigator.can_go_back() {
                    navigator.go_back();
                }
            },
            Icon {
                icon: HiArrowLeft,
                class: "h-7 w-7 sm:h-8 sm:w-8",
                width: None,
                height: None,
                title: None,
            }
            span { class: "sr-only", "Back" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Routable)]
    enum TestRoute {
        #[route("/")]
        Root {},
    }

    #[component]
    fn TestApp() -> Element {
        rsx! { Router::<TestRoute> {} }
    }

    #[component]
    fn Root() -> Element {
        rsx! { BackButton {} }
    }

    #[test]
    fn back_button_is_hidden_without_history() {
        let mut dom = VirtualDom::new(TestApp);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        assert!(!html.contains("data-focusable"));
    }
}
