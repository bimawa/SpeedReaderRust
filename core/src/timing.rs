use crate::tokenizer::Token;
use std::time::Duration;

/// Расчёт времени показа для каждого слова на основе WPM.
/// Requirement: 4.1, Design: TimingEngine section

pub struct TimingEngine {
    wpm: u32,
}

impl TimingEngine {
    pub fn new(wpm: u32) -> Self {
        Self { wpm: wpm.clamp(50, 1000) }
    }

    pub fn wpm(&self) -> u32 {
        self.wpm
    }

    /// Возвращает длительность показа слова и паузу после него.
    pub fn calculate(&self, token: &Token) -> TimedToken {
        let word_len = token.word.chars().count().max(1);
        let base_ms = 60_000.0 / self.wpm as f64;
        // Word-length scaling: a 5-char word gets 1x, shorter gets less, longer gets more
        let scale = (word_len as f64 / 5.0).clamp(0.5, 3.0);
        let display_ms = (base_ms * scale).max(30.0);

        let pause_after = if token.is_sentence_end {
            Duration::from_millis((display_ms * 1.5) as u64)
        } else {
            Duration::from_millis(0)
        };

        TimedToken {
            display_duration: Duration::from_millis(display_ms as u64),
            pause_after,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimedToken {
    pub display_duration: Duration,
    pub pause_after: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(word: &str) -> Token {
        Token {
            word: word.to_string(),
            orp_index: 0,
            byte_offset: 0,
            byte_len: word.len(),
            is_sentence_end: false,
        }
    }

    #[test]
    fn test_timing_base() {
        let engine = TimingEngine::new(300);
        let t = engine.calculate(&tok("hello"));
        // 60000/300 = 200ms base, * 1.0 (5 chars / 5)
        assert_eq!(t.display_duration.as_millis(), 200);
        assert_eq!(t.pause_after.as_millis(), 0);
    }

    #[test]
    fn test_timing_short_word() {
        let engine = TimingEngine::new(300);
        let t = engine.calculate(&tok("a"));
        // 60000/300 = 200ms * 0.5 (clamp 1/5)
        assert_eq!(t.display_duration.as_millis(), 100);
    }

    #[test]
    fn test_timing_long_word() {
        let engine = TimingEngine::new(300);
        let t = engine.calculate(&tok("presentation"));
        // 60000/300 = 200ms * 3.0 (clamp 12/5)
        assert_eq!(t.display_duration.as_millis(), 480);
    }

    #[test]
    fn test_timing_sentence_end_pause() {
        let engine = TimingEngine::new(300);
        let t = engine.calculate(&Token {
            word: "Go.".to_string(),
            orp_index: 0,
            byte_offset: 0,
            byte_len: 3,
            is_sentence_end: true,
        });
        assert!(t.pause_after.as_millis() > 0);
    }

    #[test]
    fn test_wpm_clamping() {
        let engine = TimingEngine::new(0);  // below min
        assert_eq!(engine.wpm(), 50);
        let engine = TimingEngine::new(2000);  // above max
        assert_eq!(engine.wpm(), 1000);
    }
}
