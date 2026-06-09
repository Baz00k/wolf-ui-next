use dioxus::prelude::*;

#[component]
pub fn Skeleton(#[props(default = String::from("h-4 w-full"))] class: String) -> Element {
    rsx! {
        div { class: "animate-pulse rounded-md bg-muted {class}" }
    }
}
