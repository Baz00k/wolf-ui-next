mod gamepad;

use dioxus::prelude::*;
use futures_util::StreamExt;

pub use gamepad::InputSource;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UiAction {
    Accept,
    Cancel,
    Menu,
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
}

#[derive(Clone, Copy, Debug)]
struct InputState {
    #[allow(dead_code)]
    pub source: Signal<InputSource>,
}

#[derive(Clone, Copy, Debug)]
pub enum InputEvent {
    SourceChanged(InputSource),
    Action {
        source: InputSource,
        action: UiAction,
    },
}

#[component]
pub fn InputProvider(children: Element) -> Element {
    let mut source = use_signal(|| InputSource::MouseKeyboard);
    use_context_provider(|| InputState { source });

    let dispatcher = use_coroutine(
        move |mut events: UnboundedReceiver<InputEvent>| async move {
            while let Some(event) = events.next().await {
                match event {
                    InputEvent::SourceChanged(next_source) => {
                        source.set(next_source);
                        if matches!(next_source, InputSource::Gamepad(_)) {
                            recover_focus();
                        }
                    }
                    InputEvent::Action {
                        source: next_source,
                        action,
                    } => {
                        source.set(next_source);
                        if matches!(next_source, InputSource::Gamepad(_)) {
                            recover_focus();
                        }
                        dispatch_action(action);
                    }
                }
            }
        },
    );

    gamepad::use_gamepad_input(dispatcher.tx());

    let onkeydown = move |event: KeyboardEvent| {
        if let Some(action) = keyboard_action(&event) {
            source.set(InputSource::MouseKeyboard);

            if action == UiAction::Accept {
                return;
            }

            event.prevent_default();
            dispatcher.send(InputEvent::Action {
                source: InputSource::MouseKeyboard,
                action,
            });
        }
    };

    let onpointerdown = move |_| {
        source.set(InputSource::MouseKeyboard);
    };

    rsx! {
        div {
            class: "min-h-screen",
            tabindex: "-1",
            onkeydown,
            onpointerdown,
            {children}
        }
    }
}

fn keyboard_action(event: &KeyboardEvent) -> Option<UiAction> {
    match event.key() {
        Key::Enter => Some(UiAction::Accept),
        Key::Character(value) if value == " " => Some(UiAction::Accept),
        Key::Escape | Key::Backspace => Some(UiAction::Cancel),
        Key::ArrowUp => Some(UiAction::Up),
        Key::ArrowDown => Some(UiAction::Down),
        Key::ArrowLeft => Some(UiAction::Left),
        Key::ArrowRight => Some(UiAction::Right),
        Key::PageUp => Some(UiAction::PageUp),
        Key::PageDown => Some(UiAction::PageDown),
        _ => None,
    }
}

fn dispatch_action(action: UiAction) {
    let action = match action {
        UiAction::Accept => "accept",
        UiAction::Cancel => "cancel",
        UiAction::Menu => "menu",
        UiAction::Up => "up",
        UiAction::Down => "down",
        UiAction::Left => "left",
        UiAction::Right => "right",
        UiAction::PageUp => "page-up",
        UiAction::PageDown => "page-down",
    };

    let _ = document::eval(&format!("window.__wolfUiDispatchAction?.({action:?});"));
}

fn recover_focus() {
    let _ = document::eval("window.__wolfUiEnsureFocusableActiveElement?.();");
}
