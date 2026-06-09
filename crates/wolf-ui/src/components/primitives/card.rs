use dioxus::prelude::*;

#[component]
pub fn Card(#[props(default)] class: String, children: Element) -> Element {
    rsx! {
        div { class: "rounded-4xl border border-border bg-card text-card-foreground shadow-2xl {class}",
            {children}
        }
    }
}

#[component]
pub fn CardHeader(#[props(default)] class: String, children: Element) -> Element {
    rsx! {
        div { class: "border-b border-border/70 px-6 py-6 {class}",
            {children}
        }
    }
}

#[component]
pub fn CardContent(#[props(default)] class: String, children: Element) -> Element {
    rsx! {
        div { class: "px-4 py-4 {class}",
            {children}
        }
    }
}

#[component]
pub fn CardFooter(#[props(default)] class: String, children: Element) -> Element {
    rsx! {
        div { class: "border-t border-border/70 px-4 py-4 {class}",
            {children}
        }
    }
}
