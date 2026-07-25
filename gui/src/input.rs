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
        Self { file_path, position_tracker: PositionTracker::new(text), tokens, config }
    }

    pub fn handle_key(&self, key: &Key, state: &mut ReadingState) -> ActionResult {
        match key {
            Key::Named(NamedKey::Space) => self.handle_space(state),
            Key::Character(c) if c.as_ref() == " " => self.handle_space(state),
            Key::Named(NamedKey::Escape) => { let _ = state.transition(Event::Stop); ActionResult::Exit }
            Key::Named(NamedKey::ArrowLeft) => self.handle_skip(state, false),
            Key::Named(NamedKey::ArrowRight) => self.handle_skip(state, true),
            Key::Named(NamedKey::ArrowUp) => self.handle_speed(state, true),
            Key::Named(NamedKey::ArrowDown) => self.handle_speed(state, false),
            Key::Character(c) if c.as_ref() == "r" || c.as_ref() == "R" => {
                let _ = state.transition(Event::Stop);
                let _ = state.transition(Event::Play);
                ActionResult::Restarted
            }
            _ => ActionResult::Continue,
        }
    }

    fn handle_space(&self, state: &mut ReadingState) -> ActionResult {
        match state.current_state() {
            State::Playing { .. } => { let _ = state.transition(Event::Pause); self.jump_to_position(state); ActionResult::PausedAndJumped }
            State::Paused { .. } => { let _ = state.transition(Event::Resume); ActionResult::Render }
            _ => ActionResult::Continue,
        }
    }

    fn handle_skip(&self, state: &mut ReadingState, forward: bool) -> ActionResult {
        if !matches!(state.current_state(), State::Paused { .. }) { return ActionResult::Continue }
        let n = self.config.skip_amount as usize;
        let ev = if forward { Event::SkipForward(n) } else { Event::SkipBackward(n) };
        let _ = state.transition(ev);
        ActionResult::Render
    }

    fn handle_speed(&self, state: &mut ReadingState, up: bool) -> ActionResult {
        let step = self.config.speed_step;
        if matches!(state.current_state(), State::Playing { .. } | State::Paused { .. }) {
            let ev = if up { Event::SpeedUp(step) } else { Event::SpeedDown(step) };
            let _ = state.transition(ev);
            ActionResult::SpeedChanged(state.wpm())
        } else {
            ActionResult::Continue
        }
    }

    pub fn jump_to_position(&self, state: &ReadingState) {
        let fp = match &self.file_path { Some(f) => f.clone(), None => return };
        let ti = match state.current_index() { Some(i) => i, None => return };
        if ti >= self.tokens.len() { return }
        let pos = self.position_tracker.position_for_offset(self.tokens[ti].byte_offset);
        let _ = std::process::Command::new(&self.config.editor_cmd)
            .arg(format!("\"{}\":{}:{}", fp, pos.line, pos.column + 1))
            .spawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cfg() -> ConfigModel { ConfigModel::default() }
    fn tok() -> Vec<Token> { vec![Token { word: "hello".into(), orp_index: 1, byte_offset: 0, byte_len: 5, is_sentence_end: false }] }
    fn txt() -> &'static str { "hello" }

    #[test] fn space_pauses() {
        let h = InputHandler::new(Some("x.rs".into()), tok(), txt(), cfg());
        let mut s = ReadingState::new(1, 300); let _ = s.transition(Event::Play);
        assert_eq!(h.handle_key(&Key::Named(NamedKey::Space), &mut s), ActionResult::PausedAndJumped);
    }
    #[test] fn space_char_pauses() {
        let h = InputHandler::new(Some("x.rs".into()), tok(), txt(), cfg());
        let mut s = ReadingState::new(1, 300); let _ = s.transition(Event::Play);
        assert_eq!(h.handle_key(&Key::Character(" ".into()), &mut s), ActionResult::PausedAndJumped);
    }
    #[test] fn space_resumes() {
        let h = InputHandler::new(Some("x.rs".into()), tok(), txt(), cfg());
        let mut s = ReadingState::new(1, 300); let _ = s.transition(Event::Play); let _ = s.transition(Event::Pause);
        assert_eq!(h.handle_key(&Key::Named(NamedKey::Space), &mut s), ActionResult::Render);
    }
    #[test] fn space_char_resumes() {
        let h = InputHandler::new(Some("x.rs".into()), tok(), txt(), cfg());
        let mut s = ReadingState::new(1, 300); let _ = s.transition(Event::Play); let _ = s.transition(Event::Pause);
        assert_eq!(h.handle_key(&Key::Character(" ".into()), &mut s), ActionResult::Render);
    }
    #[test] fn esc_exits() {
        let h = InputHandler::new(Some("x.rs".into()), tok(), txt(), cfg());
        let mut s = ReadingState::new(1, 300); let _ = s.transition(Event::Play);
        assert_eq!(h.handle_key(&Key::Named(NamedKey::Escape), &mut s), ActionResult::Exit);
    }
    #[test] fn r_restarts() {
        let h = InputHandler::new(Some("x.rs".into()), tok(), txt(), cfg());
        let mut s = ReadingState::new(1, 300); let _ = s.transition(Event::Play); let _ = s.transition(Event::Pause);
        assert_eq!(h.handle_key(&Key::Character("r".into()), &mut s), ActionResult::Restarted);
    }
    #[test] fn speed_up_works_playing() {
        let h = InputHandler::new(Some("x.rs".into()), tok(), txt(), cfg());
        let mut s = ReadingState::new(1, 300); let _ = s.transition(Event::Play);
        assert!(matches!(h.handle_key(&Key::Named(NamedKey::ArrowUp), &mut s), ActionResult::SpeedChanged(_)));
    }
    #[test] fn speed_down_works_paused() {
        let h = InputHandler::new(Some("x.rs".into()), tok(), txt(), cfg());
        let mut s = ReadingState::new(1, 300); let _ = s.transition(Event::Play); let _ = s.transition(Event::Pause);
        assert!(matches!(h.handle_key(&Key::Named(NamedKey::ArrowDown), &mut s), ActionResult::SpeedChanged(_)));
    }
}
