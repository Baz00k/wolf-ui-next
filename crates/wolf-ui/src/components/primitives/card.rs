use dioxus::prelude::*;
use tw_merge::tw_merge;

use crate::components::primitives::Focusable;

#[component]
pub fn Card(#[props(default)] class: String, children: Element) -> Element {
    let class = tw_merge!(
        "rounded-4xl border-2 border-border bg-card text-card-foreground shadow-2xl transition duration-200 ease-out group-focus-visible:border-foreground group-focus-visible:ring-4 group-focus-visible:ring-ring/60 group-focus-visible:ring-offset-2 group-focus-visible:ring-offset-background",
        class,
    );

    rsx! {
        div { class, {children} }
    }
}

#[component]
pub fn CardHeader(#[props(default)] class: String, children: Element) -> Element {
    let class = tw_merge!("border-b-2 border-border/70 px-6 py-6", class);

    rsx! {
        div { class, {children} }
    }
}

#[component]
pub fn CardContent(#[props(default)] class: String, children: Element) -> Element {
    let class = tw_merge!("px-4 py-4", class);

    rsx! {
        div { class, {children} }
    }
}

#[component]
pub fn CardFooter(#[props(default)] class: String, children: Element) -> Element {
    let class = tw_merge!("border-t-2 border-border/70 p-3 sm:p-4", class);

    rsx! {
        div { class, {children} }
    }
}

/// Focusable grid-card wrapper with the shared lift-on-focus motion.
#[component]
pub fn CardTrigger(
    #[props(default)] class: String,
    #[props(default)] to: Option<String>,
    #[props(default = "Select".to_string())] action_label: String,
    #[props(default)] index: Option<usize>,
    #[props(default)] autofocus: bool,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    #[props(default)] onfocus: Option<EventHandler<FocusEvent>>,
    children: Element,
) -> Element {
    let class = tw_merge!(
        "group relative flex aspect-[3/4] w-full border-0 p-0 outline-none transition duration-200 ease-out hover:-translate-y-1 focus-visible:-translate-y-1 active:scale-95",
        class,
    );

    rsx! {
        Focusable {
            class,
            to,
            action_label,
            index,
            autofocus,
            onclick,
            onfocus,
            {children}
        }
    }
}
