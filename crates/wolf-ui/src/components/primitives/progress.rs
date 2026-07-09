use dioxus::prelude::*;
use tw_merge::tw_merge;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProgressTone {
    Primary,
    Warning,
}

impl ProgressTone {
    fn panel_class(self) -> &'static str {
        match self {
            ProgressTone::Primary => {
                "rounded-xl border border-primary/30 bg-primary/10 px-4 py-3 text-foreground"
            }
            ProgressTone::Warning => {
                "rounded-xl border border-yellow-300/30 bg-yellow-300/10 px-4 py-3"
            }
        }
    }

    fn label_class(self) -> &'static str {
        match self {
            ProgressTone::Primary => "truncate",
            ProgressTone::Warning => "truncate text-yellow-100",
        }
    }

    fn value_class(self) -> &'static str {
        match self {
            ProgressTone::Primary => "shrink-0 tabular-nums",
            ProgressTone::Warning => "shrink-0 tabular-nums text-yellow-200",
        }
    }

    fn fill_class(self) -> &'static str {
        match self {
            ProgressTone::Primary => "bg-primary",
            ProgressTone::Warning => "bg-yellow-300",
        }
    }
}

#[component]
pub fn Progress(
    value: u8,
    #[props(default = ProgressTone::Primary)] tone: ProgressTone,
    #[props(default)] class: String,
    #[props(default)] indicator_class: String,
) -> Element {
    let value = value.min(100);
    let class = tw_merge!("h-1.5 overflow-hidden rounded-full bg-background/80", class);
    let indicator_class = tw_merge!(
        "h-full rounded-full transition-[width] duration-300",
        tone.fill_class(),
        indicator_class,
    );

    rsx! {
        div {
            class,
            "data-slot": "progress",
            div {
                class: indicator_class,
                "data-slot": "progress-indicator",
                style: "width: {value}%;"
            }
        }
    }
}

#[component]
pub fn ProgressPanel(
    label: String,
    progress: u8,
    #[props(default = ProgressTone::Primary)] tone: ProgressTone,
) -> Element {
    let progress = progress.min(100);

    rsx! {
        div { class: tone.panel_class(),
            div { class: "flex items-center justify-between gap-4 text-sm font-bold",
                span { class: tone.label_class(), "{label}" }
                span { class: tone.value_class(), "{progress}%" }
            }
            Progress { value: progress, tone, class: "mt-2" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(component: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(component);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn progress_clamps_width() {
        let html = render(|| rsx! { Progress { value: 200 } });

        assert!(html.contains(r#"data-slot="progress""#));
        assert!(html.contains(r#"data-slot="progress-indicator""#));
        assert!(html.contains("width: 100%;"));
    }

    #[test]
    fn progress_panel_clamps_labelled_value() {
        let html = render(|| rsx! { ProgressPanel { label: "Installing", progress: 200 } });

        assert!(html.contains("Installing"));
        assert!(html.contains("100%"));
        assert!(html.contains("width: 100%;"));
    }
}
