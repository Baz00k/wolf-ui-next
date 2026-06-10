use dioxus::prelude::*;

#[component]
pub fn Dialog(
    #[props(default)] class: String,
    #[props(default)] scope_actions: String,
    children: Element,
) -> Element {
    use_hook(capture_dialog_opener);
    use_drop(restore_dialog_opener);

    rsx! {
        div {
            class: "fixed inset-0 z-100 flex items-center justify-center bg-black/50 backdrop-blur-sm px-5 {class}",
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
