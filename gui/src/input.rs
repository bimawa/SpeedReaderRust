use speed_reader_core::config::ConfigModel;
use speed_reader_core::position::PositionTracker;
use speed_reader_core::state::{Event, ReadingState, State};
use speed_reader_core::tokenizer::Token;
use winit::keyboard::{Key, NamedKey};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActionResult {
    Continue,
    Exit,
    Render,
    PausedAndJumped,
    SpeedChanged(u32),
    Restarted,
}

pub struct InputHandler {
    file_path: Option<String>,
    tokens: Vec<Token>,
    position_tracker: PositionTracker,
    config: ConfigModel,
}

impl InputHandler {
    pub fn new(file_path: Option<String>, tokens: Vec<Token>, text: &str, config: ConfigModel) -> Self {
        let position_tracker = PositionTracker::new(text);
        Self {
            file_path,
            tokens,
            position_tracker,
            config,
        }
    }

    pub fn handle_key(
        &self,
        key: &Key,
        state: &mut ReadingState,
    ) -> ActionResult {
        match key {
            Key::Named(NamedKey::Space) => match state.current_state() {
                State::Playing { .. } => {
                    let _ = state.transition(Event::Pause);
                    self.jump_to_position(state);
                    ActionResult::PausedAndJumped
                }
                State::Paused { .. } => {
                    let _ = state.transition(Event::Resume);
                    ActionResult::Continue
                }
                _ => ActionResult::Continue,
            },
            Key::Named(NamedKey::Escape) => {
                let _ = state.transition(Event::Stop);
                ActionResult::Exit
            }
            Key::Named(NamedKey::ArrowLeft) => {
                if let State::Paused { .. } = state.current_state() {
                    let _ = state.transition(Event::SkipBackward(self.config.skip_amount as usize));
                    ActionResult::Render
                } else {
                    ActionResult::Continue
                }
            }
            Key::Named(NamedKey::ArrowRight) => {
                if let State::Paused { .. } = state.current_state() {
                    let _ = state.transition(Event::SkipForward(self.config.skip_amount as usize));
                    ActionResult::Render
                } else {
                    ActionResult::Continue
                }
            }
            Key::Named(NamedKey::ArrowUp) => {
                if let State::Paused { .. } = state.current_state() {
                    let _ = state.transition(Event::SpeedUp(self.config.speed_step));
                    ActionResult::SpeedChanged(state.wpm())
                } else {
                    ActionResult::Continue
                }
            }
            Key::Named(NamedKey::ArrowDown) => {
                if let State::Paused { .. } = state.current_state() {
                    let _ = state.transition(Event::SpeedDown(self.config.speed_step));
                    ActionResult::SpeedChanged(state.wpm())
                } else {
                    ActionResult::Continue
                }
            }
            Key::Character(c) if c.as_ref() == "r" || c.as_ref() == "R" => {
                let _ = state.transition(Event::Stop);
                let _ = state.transition(Event::Play);
                ActionResult::Restarted
            }
            _ => ActionResult::Continue,
        }
    }

    pub fn jump_to_position(&self, state: &ReadingState) {
        let file_path = match &self.file_path {
            Some(fp) => fp.clone(),
            None => return,
        };

        let token_index = match state.current_index() {
            Some(idx) => idx,
            None => return,
        };

        if token_index >= self.tokens.len() {
            return;
        }

        let byte_offset = self.tokens[token_index].byte_offset;
        let pos = self.position_tracker.position_for_offset(byte_offset);

        let _ = std::process::Command::new("zed")
            .arg(format!("{}:{}:{}", file_path, pos.line, pos.column + 1))
            .spawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> ConfigModel {
        ConfigModel::default()
    }

    fn sample_tokens() -> Vec<Token> {
        vec![
            Token { word: "hello".into(), orp_index: 1, byte_offset: 0, byte_len: 5, is_sentence_end: false },
            Token { word: "world".into(), orp_index: 1, byte_offset: 6, byte_len: 5, is_sentence_end: false },
            Token { word: "test".into(), orp_index: 1, byte_offset: 12, byte_len: 4, is_sentence_end: false },
        ]
    }

    fn sample_text() -> &'static str {
        "hello world\ntest"
    }

    #[test]
    fn action_result_continue_is_distinct() {
        let a = ActionResult::Continue;
        let b = ActionResult::Exit;
        let c = ActionResult::Render;
        let d = ActionResult::PausedAndJumped;
        let e = ActionResult::SpeedChanged(300);
        let f = ActionResult::Restarted;
        assert_ne!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
        assert_ne!(a, e);
        assert_ne!(a, f);
        assert_ne!(b, c);
        assert_ne!(b, d);
        assert_ne!(b, e);
        assert_ne!(b, f);
        assert_ne!(c, d);
        assert_ne!(c, e);
        assert_ne!(c, f);
        assert_ne!(d, e);
        assert_ne!(d, f);
        assert_ne!(e, f);
    }

    #[test]
    fn input_handler_construction() {
        let handler = InputHandler::new(
            Some("test.rs".into()),
            sample_tokens(),
            sample_text(),
            test_config(),
        );
        assert_eq!(handler.file_path.as_deref(), Some("test.rs"));
        assert_eq!(handler.tokens.len(), 3);
    }

    #[test]
    fn input_handler_no_file_path() {
        let handler = InputHandler::new(
            None,
            sample_tokens(),
            sample_text(),
            test_config(),
        );
        assert!(handler.file_path.is_none());
    }

    #[test]
    fn space_pauses_when_playing() {
        let handler = InputHandler::new(
            Some("test.rs".into()),
            sample_tokens(),
            sample_text(),
            test_config(),
        );
        let mut state = ReadingState::new(3, 300);
        let _ = state.transition(Event::Play);
        assert!(matches!(state.current_state(), State::Playing { .. }));

        let result = handler.handle_key(&Key::Named(NamedKey::Space), &mut state);

        assert_eq!(result, ActionResult::PausedAndJumped);
        assert!(matches!(state.current_state(), State::Paused { .. }));
    }

    #[test]
    fn space_resumes_when_paused() {
        let handler = InputHandler::new(
            Some("test.rs".into()),
            sample_tokens(),
            sample_text(),
            test_config(),
        );
        let mut state = ReadingState::new(3, 300);
        let _ = state.transition(Event::Play);
        let _ = state.transition(Event::Pause);
        assert!(matches!(state.current_state(), State::Paused { .. }));

        let result = handler.handle_key(&Key::Named(NamedKey::Space), &mut state);

        assert_eq!(result, ActionResult::Continue);
        assert!(matches!(state.current_state(), State::Playing { .. }));
    }

    #[test]
    fn esc_returns_exit() {
        let handler = InputHandler::new(
            Some("test.rs".into()),
            sample_tokens(),
            sample_text(),
            test_config(),
        );
        let mut state = ReadingState::new(3, 300);
        let _ = state.transition(Event::Play);

        let result = handler.handle_key(&Key::Named(NamedKey::Escape), &mut state);

        assert_eq!(result, ActionResult::Exit);
        assert!(matches!(state.current_state(), State::Idle));
    }

    #[test]
    fn left_skips_backward_when_paused() {
        let handler = InputHandler::new(
            Some("test.rs".into()),
            sample_tokens(),
            sample_text(),
            test_config(),
        );
        let mut state = ReadingState::new(3, 300);
        let _ = state.transition(Event::Play);
        let _ = state.transition(Event::Pause);

        let result = handler.handle_key(&Key::Named(NamedKey::ArrowLeft), &mut state);

        assert_eq!(result, ActionResult::Render);
        assert!(matches!(state.current_state(), State::Paused { token_index: 0 }));
    }

    #[test]
    fn right_skips_forward_when_paused() {
        let handler = InputHandler::new(
            Some("test.rs".into()),
            sample_tokens(),
            sample_text(),
            test_config(),
        );
        let mut state = ReadingState::new(3, 300);
        let _ = state.transition(Event::Play);
        let _ = state.transition(Event::Pause);

        let result = handler.handle_key(&Key::Named(NamedKey::ArrowRight), &mut state);
        assert_eq!(result, ActionResult::Render);
        assert!(matches!(state.current_state(), State::Finished));
    }

    #[test]
    fn up_speeds_up_when_paused() {
        let handler = InputHandler::new(
            Some("test.rs".into()),
            sample_tokens(),
            sample_text(),
            test_config(),
        );
        let mut state = ReadingState::new(3, 300);
        let _ = state.transition(Event::Play);
        let _ = state.transition(Event::Pause);

        let result = handler.handle_key(&Key::Named(NamedKey::ArrowUp), &mut state);

        assert_eq!(result, ActionResult::SpeedChanged(310));
        assert_eq!(state.wpm(), 310);
    }

    #[test]
    fn down_slows_down_when_paused() {
        let mut config = test_config();
        config.wpm = 300;
        let handler = InputHandler::new(
            Some("test.rs".into()),
            sample_tokens(),
            sample_text(),
            config,
        );
        let mut state = ReadingState::new(3, 300);
        let _ = state.transition(Event::Play);
        let _ = state.transition(Event::Pause);

        let result = handler.handle_key(&Key::Named(NamedKey::ArrowDown), &mut state);

        assert_eq!(result, ActionResult::SpeedChanged(290));
        assert_eq!(state.wpm(), 290);
    }

    #[test]
    fn left_ignored_when_playing() {
        let handler = InputHandler::new(
            Some("test.rs".into()),
            sample_tokens(),
            sample_text(),
            test_config(),
        );
        let mut state = ReadingState::new(3, 300);
        let _ = state.transition(Event::Play);

        let result = handler.handle_key(&Key::Named(NamedKey::ArrowLeft), &mut state);

        assert_eq!(result, ActionResult::Continue);
        assert!(matches!(state.current_state(), State::Playing { token_index: 0 }));
    }

    #[test]
    fn r_restarts() {
        let handler = InputHandler::new(
            Some("test.rs".into()),
            sample_tokens(),
            sample_text(),
            test_config(),
        );
        let mut state = ReadingState::new(3, 300);
        let _ = state.transition(Event::Play);
        let _ = state.transition(Event::Pause);

        let result = handler.handle_key(&Key::Character("r".into()), &mut state);

        assert_eq!(result, ActionResult::Restarted);
        assert!(matches!(state.current_state(), State::Playing { token_index: 0 }));
    }

    #[test]
    fn unknown_key_returns_continue() {
        let handler = InputHandler::new(
            Some("test.rs".into()),
            sample_tokens(),
            sample_text(),
            test_config(),
        );
        let mut state = ReadingState::new(3, 300);
        let _ = state.transition(Event::Play);

        let result = handler.handle_key(&Key::Named(NamedKey::Tab), &mut state);

        assert_eq!(result, ActionResult::Continue);
    }

    #[test]
    fn jump_to_position_skips_without_file_path() {
        let handler = InputHandler::new(
            None,
            sample_tokens(),
            sample_text(),
            test_config(),
        );
        let mut state = ReadingState::new(3, 300);
        let _ = state.transition(Event::Play);

        handler.jump_to_position(&state);
    }

    #[test]
    fn jump_to_position_skips_when_idle() {
        let handler = InputHandler::new(
            Some("test.rs".into()),
            sample_tokens(),
            sample_text(),
            test_config(),
        );
        let state = ReadingState::new(5, 300);

        handler.jump_to_position(&state);
    }

    #[test]
    fn jump_to_position_spawns_zed_for_playing_state() {
        let handler = InputHandler::new(
            Some("src/main.rs".into()),
            sample_tokens(),
            sample_text(),
            test_config(),
        );
        let mut state = ReadingState::new(5, 300);
        state.transition(Event::Play).unwrap();

        handler.jump_to_position(&state);
    }

}
