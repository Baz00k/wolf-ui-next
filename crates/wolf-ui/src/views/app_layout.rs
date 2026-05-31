use dioxus::prelude::*;

use crate::Route;
use crate::components::ActionFooter;

#[component]
pub fn AppLayout() -> Element {
    rsx! {
        main { class: "relative h-screen overflow-hidden bg-background text-foreground",
            div { class: "h-full",
                Outlet::<Route> {}
            }
            ActionFooter { class: "fixed bottom-12 left-0 right-0 px-6" }
        }
    }
}
