use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use dioxus::prelude::*;
use futures_util::StreamExt;

use crate::input::actions::{ActionRegistry, use_action_bridge};
use crate::input::repeat::DirectionRepeat;
use crate::input::{InputEvent, InputSource, UiAction, gamepad};

const KEYBOARD_INPUT_TICK: Duration = Duration::from_millis(16);

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
    let keyboard_repeat_tx = use_hook(move || spawn_keyboard_repeat(dispatcher.tx()));
    use_action_bridge(registry);

    let onkeydown = {
        let keyboard_repeat_tx = keyboard_repeat_tx.clone();
        move |event: KeyboardEvent| {
            if let Some(action) = keyboard_action(&event) {
                update_source(source, InputSource::MouseKeyboard);

                if action == UiAction::Accept {
                    return;
                }

                event.prevent_default();

                if is_direction_action(action) {
                    if !event.is_auto_repeating() {
                        let _ = keyboard_repeat_tx.send(KeyboardRepeatEvent::Pressed(action));
                    }
                    return;
                }

                dispatcher.send(InputEvent::Action {
                    source: InputSource::MouseKeyboard,
                    action,
                });
            }
        }
    };

    let onkeyup = move |event: KeyboardEvent| {
        if let Some(action) = keyboard_action(&event)
            && is_direction_action(action)
        {
            event.prevent_default();
            let _ = keyboard_repeat_tx.send(KeyboardRepeatEvent::Released(action));
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
            onkeyup,
            onpointerdown,
            {children}
        }
    }
}

#[derive(Clone, Copy)]
enum KeyboardRepeatEvent {
    Pressed(UiAction),
    Released(UiAction),
}

#[derive(Default)]
struct KeyboardDirectionState {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

impl KeyboardDirectionState {
    fn set(&mut self, action: UiAction, pressed: bool) {
        match action {
            UiAction::Up => self.up = pressed,
            UiAction::Down => self.down = pressed,
            UiAction::Left => self.left = pressed,
            UiAction::Right => self.right = pressed,
            _ => {}
        }
    }

    fn direction(&self) -> Option<UiAction> {
        if self.left {
            Some(UiAction::Left)
        } else if self.right {
            Some(UiAction::Right)
        } else if self.up {
            Some(UiAction::Up)
        } else if self.down {
            Some(UiAction::Down)
        } else {
            None
        }
    }
}

fn spawn_keyboard_repeat(
    dispatcher: UnboundedSender<InputEvent>,
) -> mpsc::Sender<KeyboardRepeatEvent> {
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        let mut repeat = DirectionRepeat::default();
        let mut directions = KeyboardDirectionState::default();

        loop {
            if directions.direction().is_none() {
                let Ok(event) = receiver.recv() else {
                    return;
                };
                handle_keyboard_repeat_event(event, &mut directions, &mut repeat, &dispatcher);
            } else {
                match receiver.recv_timeout(KEYBOARD_INPUT_TICK) {
                    Ok(event) => handle_keyboard_repeat_event(
                        event,
                        &mut directions,
                        &mut repeat,
                        &dispatcher,
                    ),
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => return,
                }
            }

            while let Ok(event) = receiver.try_recv() {
                handle_keyboard_repeat_event(event, &mut directions, &mut repeat, &dispatcher);
            }

            if let Some(action) = repeat.tick() {
                send_keyboard_action(&dispatcher, action);
            }
        }
    });

    sender
}

fn handle_keyboard_repeat_event(
    event: KeyboardRepeatEvent,
    directions: &mut KeyboardDirectionState,
    repeat: &mut DirectionRepeat,
    dispatcher: &UnboundedSender<InputEvent>,
) {
    match event {
        KeyboardRepeatEvent::Pressed(action) => directions.set(action, true),
        KeyboardRepeatEvent::Released(action) => directions.set(action, false),
    }

    if let Some(action) = repeat.update(directions.direction()) {
        send_keyboard_action(dispatcher, action);
    }
}

fn send_keyboard_action(dispatcher: &UnboundedSender<InputEvent>, action: UiAction) {
    let _ = dispatcher.unbounded_send(InputEvent::Action {
        source: InputSource::MouseKeyboard,
        action,
    });
}

fn is_direction_action(action: UiAction) -> bool {
    matches!(
        action,
        UiAction::Up | UiAction::Down | UiAction::Left | UiAction::Right
    )
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
        Key::Character(value) if value.eq_ignore_ascii_case("f") => Some(UiAction::Menu),
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
