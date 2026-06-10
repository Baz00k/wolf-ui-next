use dioxus::prelude::*;

use crate::Route;
use crate::components::ActionFooter;
use crate::components::primitives::ToastViewport;
use crate::input::{UiAction, use_ui_action};

#[component]
pub fn AppLayout() -> Element {
    let navigator = use_navigator();
    let _route = use_route::<Route>();
    let back_action = use_ui_action(UiAction::Cancel, "Back", move || {
        if navigator.can_go_back() {
            navigator.go_back();
        }
    });
    let global_actions = if navigator.can_go_back() {
        back_action
    } else {
        "[]".to_string()
    };

    rsx! {
        main { class: "grid h-screen w-screen overflow-x-hidden grid-rows-[minmax(0,1fr)_auto]",
            div {
                class: "mx-auto h-full w-full max-w-[min(100vw,256rem)] overflow-hidden",
                "data-focus-root": "true",
                "data-scope-actions": global_actions,
                Outlet::<Route> {}
            }
            ActionFooter {}
            ToastViewport {}
        }
    }
}
