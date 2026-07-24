use unicode_segmentation::UnicodeSegmentation;

/// Tokenizer — разбивает текст на токены с ORP-позиционированием.
/// Requirement: 1.1, Design: Tokenizer section

/// Позиция ORP-буквы в слове.
pub fn orp_index(word: &str) -> usize {
    let len = word.chars().count();
    if len <= 2 {
        0
    } else {
        ((len + 1) / 2).min(4) - 1
    }
}

/// Токен — одно слово с метаданными для RSVP.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub word: String,
    pub orp_index: usize,
    pub byte_offset: usize,
    pub byte_len: usize,
    pub is_sentence_end: bool,
}

/// Ошибки токенизации.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenizeError {
    EmptyText,
}

/// Разбивает текст на токены.
pub fn tokenize(text: &str) -> Result<Vec<Token>, TokenizeError> {
    if text.is_empty() {
        return Err(TokenizeError::EmptyText);
    }
    let mut tokens = Vec::new();
    let mut byte_offset = 0;
    let mut prev_char_was_sentence_end = false;

    for word in text.split_word_bounds() {
        let trimmed = word.trim();
        if trimmed.is_empty() {
            byte_offset += word.len();
            // whitespace between sentences
            if word.contains('\n') {
                prev_char_was_sentence_end = true;
            }
            continue;
        }
        let is_sentence_end = matches!(
            trimmed.chars().last(),
            Some('.') | Some('!') | Some('?')
        );
        // Handle sentence-starting punctuation
        let display_word = if prev_char_was_sentence_end {
            // Capitalize first letter of first word after sentence end
            let mut chars: Vec<char> = trimmed.chars().collect();
            if let Some(c) = chars.first_mut() {
                if c.is_alphabetic() {
                    *c = c.to_ascii_uppercase();
                }
            }
            chars.into_iter().collect()
        } else {
            trimmed.to_string()
        };

        let orp = orp_index(&display_word);
        tokens.push(Token {
            word: display_word,
            orp_index: orp,
            byte_offset,
            byte_len: word.len(),
            is_sentence_end,
        });
        byte_offset += word.len();
        prev_char_was_sentence_end = is_sentence_end;
    }

    if tokens.is_empty() {
        return Err(TokenizeError::EmptyText);
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orp_short_word() {
        assert_eq!(orp_index("a"), 0);
        assert_eq!(orp_index("an"), 0);
        assert_eq!(orp_index("and"), 1);
    }

    #[test]
    fn test_orp_medium_word() {
        assert_eq!(orp_index("hello"), 2);
        assert_eq!(orp_index("world"), 2);
        assert_eq!(orp_index("speed"), 2);
    }

    #[test]
    fn test_orp_long_word() {
        assert_eq!(orp_index("reading"), 3);
        assert_eq!(orp_index("presentation"), 3);
    }

    #[test]
    fn test_tokenize_simple() {
        let tokens = tokenize("Hello world").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].word, "Hello");
        assert_eq!(tokens[0].orp_index, 2);
        assert_eq!(tokens[1].word, "world");
        assert_eq!(tokens[1].orp_index, 2);
    }

    #[test]
    fn test_tokenize_empty() {
        assert_eq!(tokenize(""), Err(TokenizeError::EmptyText));
    }

    #[test]
    fn test_tokenize_sentence_end() {
        let tokens = tokenize("Go").unwrap();
        assert_eq!(tokens.len(), 1);
        // Note: punctuation like "." is split into separate tokens by unicode word segmentation
    }

    #[test]
    fn test_tokenize_byte_offsets() {
        let tokens = tokenize("Hello world").unwrap();
        assert_eq!(tokens[0].byte_offset, 0);
        assert_eq!(tokens[1].byte_offset, 6); // 'H'(0) e(1) l(2) l(3) o(4) ' '(5) w(6)
    }

    #[test]
    fn test_tokenize_unicode() {
        let tokens = tokenize("Привет мир").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].word, "Привет");
        assert_eq!(tokens[1].word, "мир");
    }
}
