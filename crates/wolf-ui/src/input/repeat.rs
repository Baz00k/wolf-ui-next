use std::time::{Duration, Instant};

use crate::input::UiAction;

pub(crate) const INPUT_INITIAL_REPEAT_DELAY: Duration = Duration::from_millis(500);
pub(crate) const INPUT_REPEAT_DELAY: Duration = Duration::from_millis(175);

#[derive(Default)]
pub(crate) struct DirectionRepeat {
    direction: Option<UiAction>,
    next_repeat: Option<Instant>,
}

impl DirectionRepeat {
    pub(crate) fn update(&mut self, direction: Option<UiAction>) -> Option<UiAction> {
        if self.direction == direction {
            return None;
        }

        self.direction = direction;
        self.next_repeat = direction.map(|_| Instant::now() + INPUT_INITIAL_REPEAT_DELAY);
        direction
    }

    pub(crate) fn tick(&mut self) -> Option<UiAction> {
        let direction = self.direction?;
        let next_repeat = self.next_repeat?;

        if Instant::now() < next_repeat {
            return None;
        }

        self.next_repeat = Some(Instant::now() + INPUT_REPEAT_DELAY);
        Some(direction)
    }
}
