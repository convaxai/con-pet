use crate::config::{CommandKey, TriggerChord, TriggerCommand};
use std::{collections::VecDeque, time::Duration};

#[derive(Default)]
pub struct SequenceMatcher {
    buffer: VecDeque<TriggerChord>,
    elapsed_since_input: Option<Duration>,
}

impl SequenceMatcher {
    pub fn feed(
        &mut self,
        chord: TriggerChord,
        command: &TriggerCommand,
        elapsed: Duration,
        timeout: Duration,
    ) -> bool {
        if self
            .elapsed_since_input
            .is_some_and(|previous| elapsed.saturating_sub(previous) > timeout)
        {
            self.buffer.clear();
        }
        self.elapsed_since_input = Some(elapsed);

        if command.steps.is_empty() {
            self.clear();
            return false;
        }

        self.buffer.push_back(chord);
        while self.buffer.len() > command.steps.len() {
            self.buffer.pop_front();
        }

        let matched = self.buffer.len() == command.steps.len()
            && self
                .buffer
                .iter()
                .zip(&command.steps)
                .all(|(actual, expected)| chord_matches(actual, expected));
        if matched {
            self.clear();
        }
        matched
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.elapsed_since_input = None;
    }

    #[cfg(test)]
    fn current(&self) -> Vec<TriggerChord> {
        self.buffer.iter().cloned().collect()
    }
}

fn chord_matches(actual: &TriggerChord, expected: &TriggerChord) -> bool {
    actual.modifiers == expected.modifiers
        && (actual.key == expected.key
            || (expected.key == CommandKey::Enter && actual.key == CommandKey::NumpadEnter))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TriggerModifier;

    fn command(steps: Vec<TriggerChord>) -> TriggerCommand {
        TriggerCommand { version: 1, steps }
    }

    #[test]
    fn matches_only_the_recent_suffix_and_clears_after_match() {
        let mut matcher = SequenceMatcher::default();
        let command = command(vec![
            TriggerChord::plain(CommandKey::KeyC),
            TriggerChord::plain(CommandKey::KeyO),
            TriggerChord::plain(CommandKey::KeyD),
        ]);
        let timeout = Duration::from_secs(3);

        assert!(!matcher.feed(
            TriggerChord::plain(CommandKey::KeyX),
            &command,
            Duration::ZERO,
            timeout
        ));
        assert!(!matcher.feed(
            TriggerChord::plain(CommandKey::KeyC),
            &command,
            Duration::from_millis(10),
            timeout
        ));
        assert!(!matcher.feed(
            TriggerChord::plain(CommandKey::KeyO),
            &command,
            Duration::from_millis(20),
            timeout
        ));
        assert!(matcher.feed(
            TriggerChord::plain(CommandKey::KeyD),
            &command,
            Duration::from_millis(30),
            timeout
        ));
        assert_eq!(matcher.current(), Vec::<TriggerChord>::new());
    }

    #[test]
    fn wrong_input_can_become_the_start_of_an_overlapping_match() {
        let mut matcher = SequenceMatcher::default();
        let command = command(vec![
            TriggerChord::plain(CommandKey::KeyA),
            TriggerChord::plain(CommandKey::KeyA),
            TriggerChord::plain(CommandKey::KeyB),
        ]);
        let timeout = Duration::from_secs(3);

        for (index, key) in [CommandKey::KeyA, CommandKey::KeyA, CommandKey::KeyA]
            .into_iter()
            .enumerate()
        {
            assert!(!matcher.feed(
                TriggerChord::plain(key),
                &command,
                Duration::from_millis(index as u64),
                timeout
            ));
        }
        assert!(matcher.feed(
            TriggerChord::plain(CommandKey::KeyB),
            &command,
            Duration::from_millis(3),
            timeout
        ));
    }

    #[test]
    fn double_space_requires_two_feed_events() {
        let mut matcher = SequenceMatcher::default();
        let command = command(vec![
            TriggerChord::plain(CommandKey::Space),
            TriggerChord::plain(CommandKey::Space),
        ]);
        let timeout = Duration::from_secs(3);

        assert!(!matcher.feed(
            TriggerChord::plain(CommandKey::Space),
            &command,
            Duration::ZERO,
            timeout
        ));
        assert!(matcher.feed(
            TriggerChord::plain(CommandKey::Space),
            &command,
            Duration::from_millis(100),
            timeout
        ));
    }

    #[test]
    fn modifiers_must_match_exactly() {
        let mut matcher = SequenceMatcher::default();
        let primary_c = command(vec![TriggerChord::new(
            CommandKey::KeyC,
            vec![TriggerModifier::Primary],
        )]);
        let timeout = Duration::from_secs(3);

        assert!(!matcher.feed(
            TriggerChord::plain(CommandKey::KeyC),
            &primary_c,
            Duration::ZERO,
            timeout
        ));
        assert!(!matcher.feed(
            TriggerChord::new(
                CommandKey::KeyC,
                vec![TriggerModifier::Primary, TriggerModifier::Shift]
            ),
            &primary_c,
            Duration::from_millis(1),
            timeout
        ));
        assert!(matcher.feed(
            TriggerChord::new(CommandKey::KeyC, vec![TriggerModifier::Primary]),
            &primary_c,
            Duration::from_millis(2),
            timeout
        ));
    }

    #[test]
    fn discards_stale_partial_input() {
        let mut matcher = SequenceMatcher::default();
        let command = command(vec![
            TriggerChord::plain(CommandKey::KeyC),
            TriggerChord::plain(CommandKey::KeyO),
        ]);
        let timeout = Duration::from_secs(1);

        assert!(!matcher.feed(
            TriggerChord::plain(CommandKey::KeyC),
            &command,
            Duration::ZERO,
            timeout
        ));
        assert!(!matcher.feed(
            TriggerChord::plain(CommandKey::KeyO),
            &command,
            Duration::from_secs(2),
            timeout
        ));
        assert_eq!(
            matcher.current(),
            vec![TriggerChord::plain(CommandKey::KeyO)]
        );
    }

    #[test]
    fn generic_enter_accepts_both_enter_keys_but_numpad_enter_is_exact() {
        let timeout = Duration::from_secs(3);
        for actual in [CommandKey::Enter, CommandKey::NumpadEnter] {
            let mut matcher = SequenceMatcher::default();
            assert!(matcher.feed(
                TriggerChord::plain(actual),
                &command(vec![TriggerChord::plain(CommandKey::Enter)]),
                Duration::ZERO,
                timeout
            ));
        }

        let mut matcher = SequenceMatcher::default();
        assert!(!matcher.feed(
            TriggerChord::plain(CommandKey::Enter),
            &command(vec![TriggerChord::plain(CommandKey::NumpadEnter)]),
            Duration::ZERO,
            timeout
        ));
    }
}
