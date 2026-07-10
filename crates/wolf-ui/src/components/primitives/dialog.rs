use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;
use tw_merge::tw_merge;

static NEXT_DIALOG_ID: AtomicU64 = AtomicU64::new(1);

#[component]
pub fn Dialog(
    #[props(default)] class: String,
    #[props(default)] scope_actions: String,
    children: Element,
) -> Element {
    let dialog_id = use_hook(|| NEXT_DIALOG_ID.fetch_add(1, Ordering::Relaxed));
    use_hook(move || capture_dialog_opener(dialog_id));
    use_drop(move || restore_dialog_opener(dialog_id));
    let class = tw_merge!(
        "fixed inset-0 z-100 grid place-items-center bg-black/75 backdrop-blur-sm px-5",
        class,
    );

    rsx! {
        div {
            class,
            "data-focus-scope": "true",
            "data-focus-trap": "true",
            "data-scope-actions": scope_actions,
            tabindex: "-1",
            {children}
        }
    }
}

fn capture_dialog_opener(dialog_id: u64) {
    let _ = document::eval(&format!(
        "window.__wolfUiCaptureDialogOpener?.({dialog_id});"
    ));
}

fn restore_dialog_opener(dialog_id: u64) {
    let _ = document::eval(&format!(
        "window.__wolfUiRestoreDialogOpener?.({dialog_id});"
    ));
}

#[component]
pub fn DialogHeader(#[props(default)] class: String, children: Element) -> Element {
    let class = tw_merge!("min-w-0 flex-1", class);

    rsx! {
        div { class, {children} }
    }
}

#[component]
pub fn DialogTitle(#[props(default)] class: String, children: Element) -> Element {
    let class = tw_merge!("truncate text-2xl font-bold tracking-tight", class);

    rsx! {
        h2 { class, {children} }
    }
}

#[component]
pub fn DialogDescription(#[props(default)] class: String, children: Element) -> Element {
    let class = tw_merge!(
        "font-mono text-xs font-semibold uppercase tracking-widest text-muted-foreground",
        class,
    );

    rsx! {
        p { class, {children} }
    }
}
