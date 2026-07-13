use crate::input::gamepad::InputSource;

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
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

impl UiAction {
    pub fn as_js_action(self) -> &'static str {
        match self {
            Self::Accept => "accept",
            Self::Cancel => "cancel",
            Self::Menu => "menu",
            Self::Up => "up",
            Self::Down => "down",
            Self::Left => "left",
            Self::Right => "right",
            Self::PageUp => "page-up",
            Self::PageDown => "page-down",
        }
    }
}

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum UiHint {
    Accept,
    Cancel,
    Menu,
    Navigate,
    PageUp,
    PageDown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UiCommand {
    Cancel,
    Menu,
}

impl From<UiCommand> for UiHint {
    fn from(command: UiCommand) -> Self {
        match command {
            UiCommand::Cancel => Self::Cancel,
            UiCommand::Menu => Self::Menu,
        }
    }
}

impl From<UiAction> for UiHint {
    fn from(action: UiAction) -> Self {
        match action {
            UiAction::Accept => Self::Accept,
            UiAction::Cancel => Self::Cancel,
            UiAction::Menu => Self::Menu,
            UiAction::Up | UiAction::Down | UiAction::Left | UiAction::Right => Self::Navigate,
            UiAction::PageUp => Self::PageUp,
            UiAction::PageDown => Self::PageDown,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum InputEvent {
    SourceChanged(InputSource),
    Action {
        source: InputSource,
        action: UiAction,
    },
}
