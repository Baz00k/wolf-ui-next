use dioxus::prelude::*;
use tw_merge::tw_merge;

#[component]
pub fn Spinner(#[props(default)] class: String) -> Element {
    let class = tw_merge!(
        "h-8 w-8 animate-spin rounded-full border-2 border-muted border-t-foreground",
        class
    );

    rsx! {
        div {
            class,
        }
    }
}
