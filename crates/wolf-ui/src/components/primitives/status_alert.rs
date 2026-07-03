use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StatusAlertVariant {
    Error,
    Info,
}

#[component]
pub fn StatusAlert(
    #[props(default)] title: Option<String>,
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
        section { class: "mx-auto flex w-full max-w-2xl flex-col items-center gap-4 rounded-4xl border px-8 py-8 text-center shadow-2xl shadow-black/30 {tone}",
            if let Some(title) = title {
                h2 { class: "text-3xl font-semibold tracking-tight", "{title}" }
            }
            p { class: "max-w-xl text-base leading-7 text-muted-foreground", "{message}" }
            {children}
        }
    }
}
