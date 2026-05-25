use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StatusAlertVariant {
    Error,
    Info,
}

#[component]
pub fn StatusAlert(
    title: String,
    message: String,
    #[props(default = StatusAlertVariant::Info)] variant: StatusAlertVariant,
    children: Element,
) -> Element {
    let tone = match variant {
        StatusAlertVariant::Error => {
            "border-destructive/40 bg-destructive/10 text-destructive-foreground"
        }
        StatusAlertVariant::Info => "border-border bg-card text-card-foreground",
    };

    rsx! {
        section {
            class: "mx-auto flex w-full max-w-2xl flex-col items-center gap-5 rounded-4xl border px-8 py-10 text-center shadow-2xl shadow-black/30 {tone}",
            role: if variant == StatusAlertVariant::Error { "alert" } else { "status" },
            h2 { class: "text-3xl font-semibold tracking-tight", "{title}" }
            p { class: "max-w-xl text-base leading-7 text-muted-foreground", "{message}" }
            {children}
        }
    }
}
