use dioxus::prelude::*;

use crate::Route;

#[component]
pub fn AppLayout() -> Element {
    rsx! {
        main {
            Outlet::<Route> {}
        }
    }
}
