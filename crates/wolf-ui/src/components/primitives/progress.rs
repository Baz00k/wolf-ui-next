use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProgressTone {
    Primary,
    Warning,
}

#[component]
pub fn ProgressPanel(
    label: String,
    progress: u8,
    #[props(default = ProgressTone::Primary)] tone: ProgressTone,
) -> Element {
    let (panel, label_class, value_class, fill) = match tone {
        ProgressTone::Primary => (
            "rounded-xl border border-primary/30 bg-primary/10 px-4 py-3 text-foreground",
            "truncate",
            "shrink-0 tabular-nums",
            "h-full rounded-full bg-primary transition-[width] duration-300",
        ),
        ProgressTone::Warning => (
            "rounded-xl border border-yellow-300/30 bg-yellow-300/10 px-4 py-3",
            "truncate text-yellow-100",
            "shrink-0 tabular-nums text-yellow-200",
            "h-full rounded-full bg-yellow-300 transition-[width] duration-300",
        ),
    };

    rsx! {
        div { class: panel,
            div { class: "flex items-center justify-between gap-4 text-sm font-bold",
                span { class: label_class, "{label}" }
                span { class: value_class, "{progress}%" }
            }
            div { class: "mt-2 h-1.5 overflow-hidden rounded-full bg-background/80",
                div { class: fill, style: "width: {progress}%;" }
            }
        }
    }
}
