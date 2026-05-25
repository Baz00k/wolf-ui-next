use dioxus::prelude::*;
use futures_util::StreamExt;

use crate::input::actions::{ActionRegistry, use_action_bridge};
use crate::input::{InputEvent, InputSource, UiAction, gamepad};

#[derive(Clone, Copy, Debug)]
struct InputState {
    pub source: Signal<InputSource>,
}

pub fn use_input_source() -> Signal<InputSource> {
    use_context::<InputState>().source
}

#[component]
pub fn InputProvider(children: Element) -> Element {
    let source = use_signal(|| InputSource::MouseKeyboard);
    let registry = use_hook(ActionRegistry::new);
    use_context_provider(|| InputState { source });
    use_context_provider(|| registry.clone());

    let dispatcher = use_coroutine(
        move |mut events: UnboundedReceiver<InputEvent>| async move {
            while let Some(event) = events.next().await {
                match event {
                    InputEvent::SourceChanged(next_source) => {
                        update_source(source, next_source);
                        if matches!(next_source, InputSource::Gamepad(_)) {
                            recover_focus();
                        }
                    }
                    InputEvent::Action {
                        source: next_source,
                        action,
                    } => {
                        update_source(source, next_source);
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
    use_action_bridge(registry);

    let onkeydown = move |event: KeyboardEvent| {
        if let Some(action) = keyboard_action(&event) {
            update_source(source, InputSource::MouseKeyboard);

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
        update_source(source, InputSource::MouseKeyboard);
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

fn update_source(mut source: Signal<InputSource>, next_source: InputSource) {
    if *source.peek() != next_source {
        source.set(next_source);
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
    let _ = document::eval(&format!(
        "window.__wolfUiDispatchAction?.({:?});",
        action.as_js_action()
    ));
}

fn recover_focus() {
    let _ = document::eval("window.__wolfUiEnsureFocusableActiveElement?.();");
}
