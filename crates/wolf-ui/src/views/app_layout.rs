use dioxus::prelude::*;

use crate::Route;
use crate::components::ActionFooter;
use crate::input::{ActionHint, UiAction, UiHint, action_hints, use_ui_action_hint};

#[component]
pub fn AppLayout() -> Element {
    let navigator = use_navigator();
    let _route = use_route::<Route>();
    let mut global_actions = vec![ActionHint::new(UiHint::Navigate, "Navigate")];

    let back_action = use_ui_action_hint(UiAction::Cancel, "Back", move || {
        if navigator.can_go_back() {
            navigator.go_back();
        }
    });

    if navigator.can_go_back() {
        global_actions.push(back_action);
    }

    rsx! {
        main { class: "grid h-screen w-screen overflow-x-hidden grid-rows-[minmax(0,1fr)_auto]",
            div {
                class: "mx-auto h-full w-full max-w-[min(100vw,256rem)] overflow-hidden",
                "data-focus-root": "true",
                "data-scope-actions": action_hints(global_actions),
                Outlet::<Route> {}
            }
            ActionFooter {}
        }
    }
}
