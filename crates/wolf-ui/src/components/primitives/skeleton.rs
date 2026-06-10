use dioxus::prelude::*;
use tw_merge::tw_merge;

#[component]
pub fn Skeleton(#[props(default)] class: String) -> Element {
    let class = tw_merge!("h-4 w-full animate-pulse rounded-md bg-muted", class);

    rsx! {
        div { class }
    }
}
