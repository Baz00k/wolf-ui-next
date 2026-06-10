use dioxus::prelude::*;

#[component]
pub fn Dialog(
    #[props(default)] class: String,
    #[props(default)] label: String,
    #[props(default)] scope_actions: String,
    children: Element,
) -> Element {
    rsx! {
        div {
            class: "fixed inset-0 z-100 flex items-center justify-center bg-black/50 backdrop-blur-sm px-5 {class}",
            role: "dialog",
            aria_modal: "true",
            aria_label: "{label}",
            "data-focus-scope": "true",
            "data-focus-trap": "true",
            "data-scope-actions": scope_actions,
            tabindex: "-1",
            {children}
        }
    }
}

#[component]
pub fn DialogHeader(#[props(default)] class: String, children: Element) -> Element {
    rsx! {
        div { class: "min-w-0 flex-1 {class}",
            {children}
        }
    }
}

#[component]
pub fn DialogTitle(#[props(default)] class: String, children: Element) -> Element {
    rsx! {
        h2 { class: "truncate text-2xl font-bold tracking-tight {class}",
            {children}
        }
    }
}

#[component]
pub fn DialogDescription(#[props(default)] class: String, children: Element) -> Element {
    rsx! {
        p { class: "font-mono text-xs font-semibold uppercase tracking-widest text-muted-foreground {class}",
            {children}
        }
    }
}
