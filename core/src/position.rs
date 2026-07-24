/// Маппинг токена в исходную позицию текста.
/// Requirement: 2.2, Design: PositionTracker section

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePosition {
    pub byte_offset: usize,
    pub line: usize,
    pub column: usize,
    pub context: String,
}

pub struct PositionTracker {
    line_offsets: Vec<usize>,
    text: String,
}

impl PositionTracker {
    pub fn new(text: &str) -> Self {
        let mut line_offsets = vec![0usize];
        for (i, c) in text.char_indices() {
            if c == '\n' {
                line_offsets.push(i + 1);
            }
        }
        Self {
            line_offsets,
            text: text.to_string(),
        }
    }

    /// Возвращает позицию в исходном тексте по байтовому смещению.
    pub fn position_for_offset(&self, byte_offset: usize) -> SourcePosition {
        let byte_offset = byte_offset.min(self.text.len());
        // binary search for the line
        let line = match self.line_offsets.binary_search(&byte_offset) {
            Ok(line_idx) => line_idx + 1,
            Err(line_idx) => line_idx, // line_idx is where it would be inserted = current line
        };
        let line = line.max(1);
        let line_start = self.line_offsets[line - 1];
        let column = byte_offset - line_start;

        // Grab context line
        let line_end = if line < self.line_offsets.len() {
            self.line_offsets[line]
        } else {
            self.text.len()
        };
        let context = self.text[line_start..line_end]
            .trim_end_matches('\n')
            .trim_end_matches('\r')
            .to_string();

        SourcePosition {
            byte_offset,
            line,
            column,
            context,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_line() {
        let tracker = PositionTracker::new("Hello world");
        let pos = tracker.position_for_offset(6);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 6);
        assert_eq!(pos.context, "Hello world");
    }

    #[test]
    fn test_multi_line() {
        let tracker = PositionTracker::new("Line one\nLine two\nLine three");
        let pos = tracker.position_for_offset(14); // "Line two" starts at 9, we want 't' in "two"
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 5);
        assert_eq!(pos.context, "Line two");
    }

    #[test]
    fn test_first_line() {
        let tracker = PositionTracker::new("Hello\nWorld");
        let pos = tracker.position_for_offset(0);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 0);
    }

    #[test]
    fn test_beyond_text() {
        let tracker = PositionTracker::new("Hello");
        let pos = tracker.position_for_offset(100);
        assert_eq!(pos.byte_offset, 5); // clamped
    }
}
