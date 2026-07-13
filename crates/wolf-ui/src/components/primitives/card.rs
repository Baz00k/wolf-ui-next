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
        div { class, "data-slot": "card", {children} }
    }
}

#[component]
pub fn CardHeader(#[props(default)] class: String, children: Element) -> Element {
    let class = tw_merge!("border-b-2 border-border/70 px-6 py-6", class);

    rsx! {
        div { class, "data-slot": "card-header", {children} }
    }
}

#[component]
pub fn CardTitle(#[props(default)] class: String, children: Element) -> Element {
    let class = tw_merge!("text-2xl font-bold tracking-tight", class);

    rsx! {
        h2 { class, "data-slot": "card-title", {children} }
    }
}

#[component]
pub fn CardDescription(#[props(default)] class: String, children: Element) -> Element {
    let class = tw_merge!("text-sm leading-6 text-muted-foreground", class);

    rsx! {
        p { class, "data-slot": "card-description", {children} }
    }
}

#[component]
pub fn CardAction(#[props(default)] class: String, children: Element) -> Element {
    let class = tw_merge!("ml-auto flex items-center gap-2", class);

    rsx! {
        div { class, "data-slot": "card-action", {children} }
    }
}

#[component]
pub fn CardContent(#[props(default)] class: String, children: Element) -> Element {
    let class = tw_merge!("px-4 py-4", class);

    rsx! {
        div { class, "data-slot": "card-content", {children} }
    }
}

#[component]
pub fn CardFooter(#[props(default)] class: String, children: Element) -> Element {
    let class = tw_merge!("border-t-2 border-border/70 p-3 sm:p-4", class);

    rsx! {
        div { class, "data-slot": "card-footer", {children} }
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
            data_slot: "card-trigger",
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

#[cfg(test)]
mod tests {
    use super::*;

    fn render(component: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(component);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn card_parts_render_stable_slots() {
        let html = render(|| {
            rsx! {
                Card {
                    CardHeader {
                        CardTitle { "Title" }
                        CardDescription { "Description" }
                        CardAction { "Action" }
                    }
                    CardContent { "Content" }
                    CardFooter { "Footer" }
                }
            }
        });

        assert!(html.contains(r#"data-slot="card""#));
        assert!(html.contains(r#"data-slot="card-header""#));
        assert!(html.contains(r#"data-slot="card-title""#));
        assert!(html.contains(r#"data-slot="card-description""#));
        assert!(html.contains(r#"data-slot="card-action""#));
        assert!(html.contains(r#"data-slot="card-content""#));
        assert!(html.contains(r#"data-slot="card-footer""#));
    }

    #[test]
    fn card_trigger_keeps_focus_contract() {
        let html = render(|| {
            rsx! {
                CardTrigger { index: 3, "Launch" }
            }
        });

        assert!(html.contains(r#"data-slot="card-trigger""#));
        assert!(html.contains(r#"data-focusable="true""#));
        assert!(html.contains(r#"data-grid-index="3""#));
    }
}
