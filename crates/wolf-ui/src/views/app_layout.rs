use dioxus::prelude::*;

use crate::Route;

#[component]
pub fn AppLayout() -> Element {
    rsx! {
        main { class: "min-h-screen bg-background text-foreground",
            Outlet::<Route> {}
        }
    }
}
