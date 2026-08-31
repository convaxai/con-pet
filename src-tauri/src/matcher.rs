use std::{collections::VecDeque, time::Duration};

#[derive(Default)]
pub struct SequenceMatcher {
    buffer: VecDeque<char>,
    elapsed_since_input: Option<Duration>,
}

impl SequenceMatcher {
    pub fn feed_text(
        &mut self,
        text: &str,
        keyword: &str,
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

        let needle: Vec<char> = keyword.to_lowercase().chars().collect();
        if needle.is_empty() {
            self.buffer.clear();
            return false;
        }

        for character in text.to_lowercase().chars() {
            if character.is_control() {
                self.buffer.clear();
                continue;
            }
            self.buffer.push_back(character);
            while self.buffer.len() > needle.len() {
                self.buffer.pop_front();
            }
        }

        if self.buffer.iter().copied().eq(needle.iter().copied()) {
            self.clear();
            return true;
        }
        false
    }

    pub fn backspace(&mut self) {
        self.buffer.pop_back();
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.elapsed_since_input = None;
    }

    #[cfg(test)]
    fn current(&self) -> String {
        self.buffer.iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_only_recent_suffix_and_clears_after_match() {
        let mut matcher = SequenceMatcher::default();
        let timeout = Duration::from_secs(3);
        assert!(!matcher.feed_text("xco", "codex", Duration::ZERO, timeout));
        assert!(matcher.feed_text("dex", "codex", Duration::from_millis(20), timeout));
        assert_eq!(matcher.current(), "");
    }

    #[test]
    fn ignores_case_and_honors_backspace() {
        let mut matcher = SequenceMatcher::default();
        let timeout = Duration::from_secs(3);
        assert!(!matcher.feed_text("COX", "codex", Duration::ZERO, timeout));
        matcher.backspace();
        assert!(matcher.feed_text("DEX", "codex", Duration::from_millis(10), timeout));
    }

    #[test]
    fn discards_stale_partial_input() {
        let mut matcher = SequenceMatcher::default();
        let timeout = Duration::from_secs(1);
        assert!(!matcher.feed_text("co", "codex", Duration::ZERO, timeout));
        assert!(!matcher.feed_text("dex", "codex", Duration::from_secs(2), timeout));
        assert_eq!(matcher.current(), "dex");
    }

    #[test]
    fn accepts_unicode_text_when_platform_hook_supplies_it() {
        let mut matcher = SequenceMatcher::default();
        let timeout = Duration::from_secs(3);
        assert!(matcher.feed_text("你好", "你好", Duration::ZERO, timeout));
    }
}
