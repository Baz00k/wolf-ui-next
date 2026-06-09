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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_change_fires_once_and_waits_before_repeating() {
        let mut repeat = DirectionRepeat::default();

        assert_eq!(repeat.update(Some(UiAction::Right)), Some(UiAction::Right));
        assert_eq!(repeat.update(Some(UiAction::Right)), None);
        assert_eq!(repeat.tick(), None);
    }

    #[test]
    fn clearing_direction_stops_repeats_until_new_direction() {
        let mut repeat = DirectionRepeat::default();

        assert_eq!(repeat.update(Some(UiAction::Down)), Some(UiAction::Down));
        assert_eq!(repeat.update(None), None);
        assert_eq!(repeat.tick(), None);
        assert_eq!(repeat.update(Some(UiAction::Up)), Some(UiAction::Up));
    }
}
