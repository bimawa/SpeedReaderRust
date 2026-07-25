/// Конечный автомат чтения.
/// Requirement: 3.1, 3.3, Design: ReadingState section

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Idle,
    Playing { token_index: usize },
    Paused { token_index: usize },
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    Play, Pause, Resume,
    SkipForward(usize), SkipBackward(usize),
    Stop, SpeedUp(u32), SpeedDown(u32), TokenAdvanced,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StateChange { Transition(State, State), Stay(State) }

#[derive(Debug, Clone, PartialEq)]
pub enum StateError { NoTokens, OutOfRange { index: usize, total: usize }, InvalidTransition { from: State, event: Event } }

pub struct ReadingState {
    state: State,
    token_count: usize,
    wpm: u32,
}

fn skip_forward(state: &mut State, idx: usize, n: usize, total: usize) {
    let new = (idx + n).min(total - 1);
    *state = if new >= total - 1 { State::Finished } else { State::Playing { token_index: new } };
}

fn skip_backward(state: &mut State, idx: usize, n: usize) {
    *state = State::Playing { token_index: idx.saturating_sub(n) };
}

impl ReadingState {
    pub fn new(token_count: usize, wpm: u32) -> Self {
        Self { state: State::Idle, token_count, wpm }
    }

    pub fn current_state(&self) -> State { self.state }

    pub fn current_index(&self) -> Option<usize> {
        match self.state {
            State::Playing { token_index } | State::Paused { token_index } => Some(token_index),
            _ => None,
        }
    }

    pub fn wpm(&self) -> u32 { self.wpm }

    pub fn transition(&mut self, event: Event) -> Result<StateChange, StateError> {
        if self.token_count == 0 { return Err(StateError::NoTokens) }
        match (self.state, event) {
            (State::Idle, Event::Play) => {
                self.state = State::Playing { token_index: 0 };
                Ok(StateChange::Transition(State::Idle, self.state))
            }
            (State::Playing { token_index }, Event::Pause) => {
                self.state = State::Paused { token_index };
                Ok(StateChange::Transition(State::Playing { token_index }, self.state))
            }
            (State::Paused { token_index }, Event::Resume) => {
                self.state = State::Playing { token_index };
                Ok(StateChange::Transition(State::Paused { token_index }, self.state))
            }
            (State::Playing { token_index } | State::Paused { token_index }, Event::SkipForward(n)) => {
                let old = self.state;
                skip_forward(&mut self.state, token_index, n, self.token_count);
                Ok(StateChange::Transition(old, self.state))
            }
            (State::Playing { token_index } | State::Paused { token_index }, Event::SkipBackward(n)) => {
                let old = self.state;
                skip_backward(&mut self.state, token_index, n);
                Ok(StateChange::Transition(old, self.state))
            }
            (State::Playing { token_index }, Event::TokenAdvanced) => {
                let new = token_index + 1;
                let old = self.state;
                self.state = if new >= self.token_count { State::Finished } else { State::Playing { token_index: new } };
                Ok(StateChange::Transition(old, self.state))
            }
            (State::Playing { .. } | State::Paused { .. }, Event::SpeedUp(delta)) => {
                self.wpm = (self.wpm + delta).min(1000);
                Ok(StateChange::Stay(self.state))
            }
            (State::Playing { .. } | State::Paused { .. }, Event::SpeedDown(delta)) => {
                self.wpm = self.wpm.saturating_sub(delta).max(50);
                Ok(StateChange::Stay(self.state))
            }
            (st, Event::Stop) => {
                self.state = State::Idle;
                Ok(StateChange::Transition(st, State::Idle))
            }
            _ => Err(StateError::InvalidTransition { from: self.state, event }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ms() -> ReadingState { ReadingState::new(10, 300) }

    #[test] fn init() { assert_eq!(ms().current_state(), State::Idle); }
    #[test] fn play() { let mut s = ms(); s.transition(Event::Play).unwrap(); assert_eq!(s.current_index(), Some(0)); }
    #[test] fn pause_resume() {
        let mut s = ms(); s.transition(Event::Play).unwrap(); s.transition(Event::Pause).unwrap();
        assert_eq!(s.current_state(), State::Paused { token_index: 0 });
        s.transition(Event::Resume).unwrap();
        assert_eq!(s.current_state(), State::Playing { token_index: 0 });
    }
    #[test] fn skip_fwd() {
        let mut s = ms(); s.transition(Event::Play).unwrap(); s.transition(Event::Pause).unwrap();
        s.transition(Event::SkipForward(3)).unwrap(); assert_eq!(s.current_index(), Some(3));
    }
    #[test] fn skip_bwd() {
        let mut s = ms(); s.transition(Event::Play).unwrap(); s.transition(Event::Pause).unwrap();
        s.transition(Event::SkipForward(5)).unwrap(); s.transition(Event::SkipBackward(2)).unwrap();
        assert_eq!(s.current_index(), Some(3));
    }
    #[test] fn skip_while_playing() {
        let mut s = ms(); s.transition(Event::Play).unwrap();
        s.transition(Event::SkipForward(3)).unwrap();
        assert_eq!(s.current_index(), Some(3));
        assert!(matches!(s.current_state(), State::Playing { .. }));
    }
    #[test] fn speed_while_playing() {
        let mut s = ms(); s.transition(Event::Play).unwrap();
        s.transition(Event::SpeedUp(50)).unwrap();
        assert_eq!(s.wpm(), 350);
    }
    #[test] fn finish() {
        let mut s = ReadingState::new(1, 300);
        s.transition(Event::Play).unwrap();
        s.transition(Event::TokenAdvanced).unwrap();
        assert_eq!(s.current_state(), State::Finished);
    }
    #[test] fn invalid() {
        let mut s = ms();
        assert!(matches!(s.transition(Event::Pause), Err(StateError::InvalidTransition { .. })));
    }
    #[test] fn no_tokens() {
        assert!(matches!(ReadingState::new(0, 300).transition(Event::Play), Err(StateError::NoTokens)));
    }
}
