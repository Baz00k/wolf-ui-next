use dioxus::prelude::*;

#[component]
pub fn Spinner(#[props(default = String::from("h-8 w-8"))] class: String) -> Element {
    rsx! {
        div {
            class: "{class} animate-spin rounded-full border-2 border-muted border-t-foreground",
            role: "status",
            aria_label: "Loading",
        }
    }
}
