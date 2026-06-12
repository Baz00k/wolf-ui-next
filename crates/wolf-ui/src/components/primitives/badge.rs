use dioxus::prelude::*;
use tw_merge::tw_merge;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BadgeVariant {
    Neutral,
    Success,
    Warning,
    Overlay,
}

#[component]
pub fn Badge(
    #[props(default = BadgeVariant::Neutral)] variant: BadgeVariant,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let tone = match variant {
        BadgeVariant::Neutral => "border-border bg-background/50 text-muted-foreground",
        BadgeVariant::Success => "border-emerald-300/30 bg-emerald-300/10 text-emerald-100",
        BadgeVariant::Warning => "border-yellow-400/30 bg-yellow-400/10 text-yellow-300",
        BadgeVariant::Overlay => {
            "border-transparent bg-black/55 text-muted-foreground backdrop-blur-sm"
        }
    };
    let class = tw_merge!(
        "inline-flex items-center gap-1.5 rounded-full border px-3 py-1 text-xs font-semibold uppercase tracking-widest",
        tone,
        class,
    );

    rsx! { span { class, {children} } }
}
