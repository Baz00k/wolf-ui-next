use crate::input::{
    ActionHint as InputActionHint, GamepadFamily, InputSource, UiHint, use_input_source,
};
use dioxus::prelude::*;

#[component]
pub fn ActionFooter(#[props(default)] class: String) -> Element {
    let mut hints = use_signal(Vec::<InputActionHint>::new);

    use_effect(move || {
        spawn(async move {
            let mut eval = document::eval(
                r#"
                document.addEventListener("wolf-ui-action-hints-changed", (event) => {
                    dioxus.send(event.detail ?? []);
                });
                dioxus.send(window.__wolfUiActionHints?.() ?? []);
                "#,
            );

            while let Ok(next_hints) = eval.recv::<Vec<InputActionHint>>().await {
                hints.set(next_hints);
            }
        });
    });

    let input_source = use_input_source();
    let hints = hints();

    if hints.is_empty() {
        return rsx! {};
    }

    let source = input_source();

    rsx! {
        footer { class: "pointer-events-none relative z-10 flex w-full flex-wrap items-center justify-center gap-x-7 gap-y-3 text-sm font-medium text-muted-foreground sm:justify-end {class}",
            for hint in hints {
                ActionHint { source, hint }
            }
        }
    }
}

#[component]
fn ActionHint(source: InputSource, hint: InputActionHint) -> Element {
    rsx! {
        div { class: "flex items-center gap-2",
            ActionGlyph { source, action: hint.action }
            span { class: "leading-none", "{hint.label}" }
        }
    }
}

#[component]
fn ActionGlyph(source: InputSource, action: UiHint) -> Element {
    let label = button_label(source, action);

    rsx! {
        span { class: "inline-flex items-center gap-1.5",
            kbd { class: "inline-flex min-h-6 min-w-6 items-center justify-center rounded-md border border-border/80 bg-card/90 px-1.5 text-[0.68rem] font-bold uppercase leading-none text-card-foreground shadow-[inset_0_-1px_0_oklch(1_0_0/0.12)]",
                "{label}"
            }
        }
    }
}

fn button_label(source: InputSource, action: UiHint) -> &'static str {
    match source {
        InputSource::MouseKeyboard => keyboard_label(action),
        InputSource::Gamepad(GamepadFamily::Xbox | GamepadFamily::Generic) => xbox_label(action),
        InputSource::Gamepad(GamepadFamily::PlayStation) => playstation_label(action),
        InputSource::Gamepad(GamepadFamily::Switch) => switch_label(action),
    }
}

fn keyboard_label(action: UiHint) -> &'static str {
    match action {
        UiHint::Accept => "Enter",
        UiHint::Cancel => "Esc",
        UiHint::Menu => "Menu",
        UiHint::Navigate => "Arrows",
        UiHint::PageUp => "PgUp",
        UiHint::PageDown => "PgDn",
    }
}

fn xbox_label(action: UiHint) -> &'static str {
    match action {
        UiHint::Accept => "A",
        UiHint::Cancel => "B",
        UiHint::Menu => "Menu",
        UiHint::Navigate => "D-pad",
        UiHint::PageUp => "LB",
        UiHint::PageDown => "RB",
    }
}

fn playstation_label(action: UiHint) -> &'static str {
    match action {
        UiHint::Accept => "Cross",
        UiHint::Cancel => "Circle",
        UiHint::Menu => "Options",
        UiHint::Navigate => "D-pad",
        UiHint::PageUp => "L1",
        UiHint::PageDown => "R1",
    }
}

fn switch_label(action: UiHint) -> &'static str {
    match action {
        UiHint::Accept => "A",
        UiHint::Cancel => "B",
        UiHint::Menu => "Plus",
        UiHint::Navigate => "D-pad",
        UiHint::PageUp => "L",
        UiHint::PageDown => "R",
    }
}
