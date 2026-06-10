use dioxus::prelude::*;
use tw_merge::tw_merge;

#[component]
pub fn Dialog(
    #[props(default)] class: String,
    #[props(default)] scope_actions: String,
    children: Element,
) -> Element {
    use_hook(capture_dialog_opener);
    use_drop(restore_dialog_opener);
    let class = tw_merge!(
        "fixed inset-0 z-100 flex items-center justify-center bg-black/50 backdrop-blur-sm px-5",
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

fn capture_dialog_opener() {
    let _ = document::eval(
        r#"
        window.__wolfUiDialogRestoreElement = document.activeElement?.matches?.('[data-focusable="true"]')
            ? document.activeElement
            : null;
        "#,
    );
}

fn restore_dialog_opener() {
    let _ = document::eval(
        r#"
        requestAnimationFrame(() => requestAnimationFrame(() => {
            const element = window.__wolfUiDialogRestoreElement;
            window.__wolfUiDialogRestoreElement = null;

            if (element?.isConnected) {
                const style = window.getComputedStyle(element);
                const rect = element.getBoundingClientRect();
                const visible = style.display !== "none"
                    && style.visibility !== "hidden"
                    && !element.matches(':disabled,[aria-disabled="true"],[inert]')
                    && rect.width > 0
                    && rect.height > 0;

                if (visible && window.__wolfUiFocusElement?.(element, { inline: "nearest" })) {
                    return;
                }
            }

            window.__wolfUiEnsureFocusableActiveElement?.();
        }));
        "#,
    );
}

#[component]
pub fn DialogHeader(#[props(default)] class: String, children: Element) -> Element {
    let class = tw_merge!("min-w-0 flex-1", class);

    rsx! {
        div { class,
            {children}
        }
    }
}

#[component]
pub fn DialogTitle(#[props(default)] class: String, children: Element) -> Element {
    let class = tw_merge!("truncate text-2xl font-bold tracking-tight", class);

    rsx! {
        h2 { class,
            {children}
        }
    }
}

#[component]
pub fn DialogDescription(#[props(default)] class: String, children: Element) -> Element {
    let class = tw_merge!(
        "font-mono text-xs font-semibold uppercase tracking-widest text-muted-foreground",
        class,
    );

    rsx! {
        p { class,
            {children}
        }
    }
}
