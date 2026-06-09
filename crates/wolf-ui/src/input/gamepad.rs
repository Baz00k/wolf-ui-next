use std::thread;
use std::time::Duration;

use dioxus::prelude::*;
use gilrs::{Axis, Button, EventType, Gamepad, GamepadId, Gilrs};

use crate::input::repeat::DirectionRepeat;
use crate::input::{InputEvent, UiAction};

const STICK_DEADZONE: f32 = 0.45;
const INPUT_TICK: Duration = Duration::from_millis(16);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GamepadFamily {
    Xbox,
    PlayStation,
    Switch,
    Generic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputSource {
    MouseKeyboard,
    Gamepad(GamepadFamily),
}

pub fn use_gamepad_input(dispatcher: UnboundedSender<InputEvent>) {
    use_effect(move || {
        let dispatcher = dispatcher.clone();
        thread::spawn(move || {
            let Ok(mut gilrs) = Gilrs::new() else {
                tracing::error!(target: "wolf-ui-input", "failed to initialize gilrs");
                return;
            };
            log_connected_gamepads(&gilrs);

            let mut repeat = DirectionRepeat::default();
            let mut dpad = DirectionState::default();
            let mut stick = StickState::default();
            let mut active_gamepad_id = None;

            loop {
                while let Some(event) = gilrs.next_event() {
                    active_gamepad_id = Some(event.id);
                    let gamepad = gilrs.gamepad(event.id);
                    let family = classify_gamepad(gamepad.name());

                    tracing::debug!(
                        target: "wolf-ui-input",
                        "event id={:?} name={:?} os_name={:?} family={:?} event={:?}",
                        event.id,
                        gamepad.name(),
                        gamepad.os_name(),
                        family,
                        event.event,
                    );

                    match event.event {
                        EventType::ButtonPressed(button, _) => {
                            if let Some(direction) = dpad_action(button) {
                                dpad.set_button(direction, true);
                                if let Some(action) = repeat.update(active_direction(&dpad, &stick))
                                {
                                    send_action(&dispatcher, family, action);
                                }
                            } else if let Some(action) = button_action(button) {
                                send_action(&dispatcher, family, action);
                            }
                        }
                        EventType::ButtonChanged(button, value, _) => {
                            if let Some(direction) = dpad_action(button) {
                                dpad.set_button(direction, value >= 0.5);
                                if let Some(action) = repeat.update(active_direction(&dpad, &stick))
                                {
                                    send_action(&dispatcher, family, action);
                                }
                            } else if let Some(action) = button_changed_action(button, value) {
                                send_action(&dispatcher, family, action);
                            }
                        }
                        EventType::ButtonReleased(button, _) => {
                            if let Some(direction) = dpad_action(button) {
                                dpad.set_button(direction, false);
                                if let Some(action) = repeat.update(active_direction(&dpad, &stick))
                                {
                                    send_action(&dispatcher, family, action);
                                }
                            }
                        }
                        EventType::AxisChanged(axis, value, _) => {
                            let previous_direction = active_direction(&dpad, &stick);
                            update_axis(&mut dpad, &mut stick, axis, value);
                            let next_direction = active_direction(&dpad, &stick);

                            if next_direction != previous_direction
                                && let Some(action) = repeat.update(next_direction)
                            {
                                send_action(&dispatcher, family, action);
                            }
                        }
                        EventType::Connected => {
                            log_gamepad("connected", gamepad);
                            let _ = dispatcher.unbounded_send(InputEvent::SourceChanged(
                                InputSource::Gamepad(family),
                            ));
                        }
                        EventType::Disconnected => {
                            tracing::debug!(target: "wolf-ui-input", "disconnected id={:?}", event.id);
                            active_gamepad_id = None;
                            repeat.update(None);
                            dpad = DirectionState::default();
                            stick = StickState::default();
                        }
                        _ => {}
                    }
                }

                if let Some(gamepad_id) = active_gamepad_id {
                    let family = {
                        let gamepad = gilrs.gamepad(gamepad_id);
                        reconcile_direction_state(&mut dpad, &mut stick, gamepad);
                        classify_gamepad(gamepad.name())
                    };

                    if let Some(action) = repeat.update(active_direction(&dpad, &stick)) {
                        send_action(&dispatcher, family, action);
                    }
                }

                if let Some(action) = repeat.tick() {
                    let family =
                        active_family(&gilrs, active_gamepad_id).unwrap_or(GamepadFamily::Generic);
                    send_action(&dispatcher, family, action);
                }

                thread::sleep(INPUT_TICK);
            }
        });
    });
}

#[derive(Default)]
struct DirectionState {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

impl DirectionState {
    fn set_button(&mut self, action: UiAction, pressed: bool) {
        match action {
            UiAction::Up => self.up = pressed,
            UiAction::Down => self.down = pressed,
            UiAction::Left => self.left = pressed,
            UiAction::Right => self.right = pressed,
            _ => {}
        }
    }

    fn set_axis(&mut self, axis: Axis, value: f32) {
        match axis {
            Axis::DPadX => {
                self.left = value <= -STICK_DEADZONE;
                self.right = value >= STICK_DEADZONE;
            }
            Axis::DPadY => {
                self.up = value >= STICK_DEADZONE;
                self.down = value <= -STICK_DEADZONE;
            }
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

#[derive(Default)]
struct StickState {
    x: f32,
    y: f32,
}

impl StickState {
    fn set_axis(&mut self, axis: Axis, value: f32) {
        match axis {
            Axis::LeftStickX => self.x = value,
            Axis::LeftStickY => self.y = value,
            _ => {}
        }
    }

    fn direction(&self) -> Option<UiAction> {
        let x_abs = self.x.abs();
        let y_abs = self.y.abs();

        if x_abs < STICK_DEADZONE && y_abs < STICK_DEADZONE {
            return None;
        }

        if x_abs >= y_abs {
            if self.x < 0.0 {
                Some(UiAction::Left)
            } else {
                Some(UiAction::Right)
            }
        } else if self.y > 0.0 {
            Some(UiAction::Up)
        } else {
            Some(UiAction::Down)
        }
    }
}

fn send_action(dispatcher: &UnboundedSender<InputEvent>, family: GamepadFamily, action: UiAction) {
    let _ = dispatcher.unbounded_send(InputEvent::Action {
        source: InputSource::Gamepad(family),
        action,
    });
}

fn active_direction(dpad: &DirectionState, stick: &StickState) -> Option<UiAction> {
    dpad.direction().or_else(|| stick.direction())
}

fn active_family(gilrs: &Gilrs, active_gamepad_id: Option<GamepadId>) -> Option<GamepadFamily> {
    if let Some(gamepad_id) = active_gamepad_id {
        return Some(classify_gamepad(gilrs.gamepad(gamepad_id).name()));
    }

    gilrs
        .gamepads()
        .next()
        .map(|(_, gamepad)| classify_gamepad(gamepad.name()))
}

fn dpad_action(button: Button) -> Option<UiAction> {
    match button {
        Button::DPadUp => Some(UiAction::Up),
        Button::DPadDown => Some(UiAction::Down),
        Button::DPadLeft => Some(UiAction::Left),
        Button::DPadRight => Some(UiAction::Right),
        _ => None,
    }
}

fn button_action(button: Button) -> Option<UiAction> {
    match button {
        Button::Start | Button::North => Some(UiAction::Menu),
        Button::South => Some(UiAction::Accept),
        Button::East => Some(UiAction::Cancel),
        _ => None,
    }
}
fn button_changed_action(button: Button, value: f32) -> Option<UiAction> {
    if value < 0.5 {
        return None;
    }

    button_action(button)
}

fn update_axis(dpad: &mut DirectionState, stick: &mut StickState, axis: Axis, value: f32) {
    dpad.set_axis(axis, value);
    stick.set_axis(axis, value);
}

fn reconcile_direction_state(dpad: &mut DirectionState, stick: &mut StickState, gamepad: Gamepad) {
    dpad.up = gamepad.is_pressed(Button::DPadUp) || gamepad.value(Axis::DPadY) >= STICK_DEADZONE;
    dpad.down =
        gamepad.is_pressed(Button::DPadDown) || gamepad.value(Axis::DPadY) <= -STICK_DEADZONE;
    dpad.left =
        gamepad.is_pressed(Button::DPadLeft) || gamepad.value(Axis::DPadX) <= -STICK_DEADZONE;
    dpad.right =
        gamepad.is_pressed(Button::DPadRight) || gamepad.value(Axis::DPadX) >= STICK_DEADZONE;

    stick.x = gamepad.value(Axis::LeftStickX);
    stick.y = gamepad.value(Axis::LeftStickY);
}

fn classify_gamepad(name: &str) -> GamepadFamily {
    let name = name.to_ascii_lowercase();

    if name.contains("dualsense")
        || name.contains("dualshock")
        || name.contains("playstation")
        || name.contains("ps4")
        || name.contains("ps5")
    {
        GamepadFamily::PlayStation
    } else if name.contains("nintendo") || name.contains("switch") || name.contains("joy-con") {
        GamepadFamily::Switch
    } else if name.contains("x-box") || name.contains("xbox") || name.contains("360") {
        GamepadFamily::Xbox
    } else {
        GamepadFamily::Generic
    }
}

fn log_connected_gamepads(gilrs: &Gilrs) {
    let mut count = 0;

    for (_, gamepad) in gilrs.gamepads() {
        count += 1;
        log_gamepad("startup", gamepad);
    }

    tracing::debug!(target: "wolf-ui-input", "startup connected_gamepads={count}");
}

fn log_gamepad(label: &str, gamepad: Gamepad) {
    tracing::debug!(
        target: "wolf-ui-input",
        "{label} name={:?} os_name={:?} map_name={:?} mapping_source={:?} vendor_id={:?} product_id={:?} uuid={:02x?}",
        gamepad.name(),
        gamepad.os_name(),
        gamepad.map_name(),
        gamepad.mapping_source(),
        gamepad.vendor_id(),
        gamepad.product_id(),
        gamepad.uuid(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_gamepad_families_are_classified() {
        assert_eq!(
            classify_gamepad("Xbox Wireless Controller"),
            GamepadFamily::Xbox,
        );
        assert_eq!(
            classify_gamepad("DualSense Wireless Controller"),
            GamepadFamily::PlayStation
        );
        assert_eq!(
            classify_gamepad("Nintendo Switch Pro Controller"),
            GamepadFamily::Switch,
        );
        assert_eq!(classify_gamepad("8BitDo Pro 2"), GamepadFamily::Generic);
    }

    #[test]
    fn wolf_virtual_gamepads_are_classified() {
        assert_eq!(
            classify_gamepad("Wolf X-Box One (virtual) pad"),
            GamepadFamily::Xbox
        );
        assert_eq!(
            classify_gamepad("Wolf DualSense (virtual) pad"),
            GamepadFamily::PlayStation,
        );
        assert_eq!(
            classify_gamepad("Wolf Nintendo (virtual) pad"),
            GamepadFamily::Switch,
        );
    }

    #[test]
    fn gamepad_action_buttons_use_gilrs_layout() {
        assert_eq!(button_action(Button::South), Some(UiAction::Accept));
        assert_eq!(button_action(Button::East), Some(UiAction::Cancel));
    }

    #[test]
    fn analog_button_changes_only_fire_on_press_threshold() {
        assert_eq!(button_changed_action(Button::South, 0.49), None);
        assert_eq!(
            button_changed_action(Button::South, 0.5),
            Some(UiAction::Accept)
        );
    }
}
