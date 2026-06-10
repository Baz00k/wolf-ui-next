use dioxus::prelude::*;
use tw_merge::tw_merge;

#[component]
pub fn Card(#[props(default)] class: String, children: Element) -> Element {
    let class = tw_merge!(
        "rounded-4xl border border-border bg-card text-card-foreground shadow-2xl",
        class,
    );

    rsx! {
        div { class,
            {children}
        }
    }
}

#[component]
pub fn CardHeader(#[props(default)] class: String, children: Element) -> Element {
    let class = tw_merge!("border-b border-border/70 px-6 py-6", class);

    rsx! {
        div { class,
            {children}
        }
    }
}

#[component]
pub fn CardContent(#[props(default)] class: String, children: Element) -> Element {
    let class = tw_merge!("px-4 py-4", class);

    rsx! {
        div { class,
            {children}
        }
    }
}

#[component]
pub fn CardFooter(#[props(default)] class: String, children: Element) -> Element {
    let class = tw_merge!("border-t border-border/70 px-4 py-4", class);

    rsx! {
        div { class,
            {children}
        }
    }
}
