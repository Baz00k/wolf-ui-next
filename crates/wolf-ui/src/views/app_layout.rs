use dioxus::prelude::*;

use crate::Route;
use crate::components::{ActionFooter, ToastViewport};

#[component]
pub fn AppLayout() -> Element {
    rsx! {
        main { class: "grid h-screen w-screen overflow-x-hidden grid-rows-[minmax(0,1fr)_auto]",
            div {
                class: "h-full w-full overflow-x-hidden overflow-y-auto scrollbar-hide",
                "data-focus-root": "true",
                Outlet::<Route> {}
            }
            ActionFooter {}
            ToastViewport {}
        }
    }
}
