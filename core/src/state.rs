
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
    Play,
    Pause,
    Resume,
    SkipForward(usize),
    SkipBackward(usize),
    Stop,
    SpeedUp(u32),
    SpeedDown(u32),
    TokenAdvanced,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StateChange {
    Transition(State, State),
    Stay(State),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StateError {
    NoTokens,
    OutOfRange { index: usize, total: usize },
    InvalidTransition { from: State, event: Event },
}

pub struct ReadingState {
    state: State,
    token_count: usize,
    wpm: u32,
}

impl ReadingState {
    pub fn new(token_count: usize, wpm: u32) -> Self {
        Self {
            state: State::Idle,
            token_count,
            wpm,
        }
    }

    pub fn current_state(&self) -> State {
        self.state
    }

    pub fn current_index(&self) -> Option<usize> {
        match self.state {
            State::Playing { token_index } | State::Paused { token_index } => Some(token_index),
            _ => None,
        }
    }

    pub fn wpm(&self) -> u32 {
        self.wpm
    }

    pub fn transition(&mut self, event: Event) -> Result<StateChange, StateError> {
        if self.token_count == 0 {
            return Err(StateError::NoTokens);
        }
        match (self.state, event) {
            (State::Idle, Event::Play) => {
                self.state = State::Playing { token_index: 0 };
                Ok(StateChange::Transition(State::Idle, self.state))
            }
            (State::Playing { token_index }, Event::Pause) => {
                self.state = State::Paused { token_index };
                Ok(StateChange::Transition(
                    State::Playing { token_index },
                    self.state,
                ))
            }
            (State::Paused { token_index }, Event::Resume) => {
                self.state = State::Playing { token_index };
                Ok(StateChange::Transition(
                    State::Paused { token_index },
                    self.state,
                ))
            }
            (State::Paused { token_index }, Event::SkipForward(n)) => {
                let new_index = (token_index + n).min(self.token_count - 1);
                if new_index >= self.token_count - 1 {
                    self.state = State::Finished;
                } else {
                    self.state = State::Paused { token_index: new_index };
                }
                Ok(StateChange::Transition(
                    State::Paused { token_index },
                    self.state,
                ))
            }
            (State::Paused { token_index }, Event::SkipBackward(n)) => {
                let new_index = token_index.saturating_sub(n);
                self.state = State::Paused {
                    token_index: new_index,
                };
                Ok(StateChange::Transition(
                    State::Paused { token_index },
                    self.state,
                ))
            }
            (State::Playing { token_index }, Event::TokenAdvanced) => {
                let new_index = token_index + 1;
                if new_index >= self.token_count {
                    self.state = State::Finished;
                } else {
                    self.state = State::Playing {
                        token_index: new_index,
                    };
                }
                Ok(StateChange::Transition(
                    State::Playing { token_index },
                    self.state,
                ))
            }
            (State::Playing { .. } | State::Paused { .. }, Event::SpeedUp(delta)) => {
                self.wpm = (self.wpm + delta).min(1000);
                Ok(StateChange::Stay(self.state))
            }
            (State::Playing { .. } | State::Paused { .. }, Event::SpeedDown(delta)) => {
                self.wpm = self.wpm.saturating_sub(delta).max(50);
                Ok(StateChange::Stay(self.state))
            }
            (state, Event::Stop) => {
                self.state = State::Idle;
                Ok(StateChange::Transition(state, State::Idle))
            }
            (_, Event::Play) | (_, Event::Pause) | (_, Event::Resume) => {
                Err(StateError::InvalidTransition { from: self.state, event })
            }
            (_, Event::SkipForward(_)) | (_, Event::SkipBackward(_)) => {
                Err(StateError::InvalidTransition { from: self.state, event })
            }
            (_, Event::TokenAdvanced) => {
                Err(StateError::InvalidTransition { from: self.state, event })
            }
            _ => Err(StateError::InvalidTransition { from: self.state, event }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> ReadingState {
        ReadingState::new(10, 300)
    }

    #[test]
    fn test_initial_state() {
        let s = make_state();
        assert_eq!(s.current_state(), State::Idle);
    }

    #[test]
    fn test_play_from_idle() {
        let mut s = make_state();
        let result = s.transition(Event::Play);
        assert!(result.is_ok());
        assert_eq!(s.current_index(), Some(0));
    }

    #[test]
    fn test_pause_resume() {
        let mut s = make_state();
        s.transition(Event::Play).unwrap();
        s.transition(Event::Pause).unwrap();
        assert_eq!(s.current_state(), State::Paused { token_index: 0 });
        s.transition(Event::Resume).unwrap();
        assert_eq!(s.current_state(), State::Playing { token_index: 0 });
    }

    #[test]
    fn test_skip_forward() {
        let mut s = make_state();
        s.transition(Event::Play).unwrap();
        s.transition(Event::Pause).unwrap();
        s.transition(Event::SkipForward(3)).unwrap();
        assert_eq!(s.current_index(), Some(3));
    }

    #[test]
    fn test_skip_backward() {
        let mut s = make_state();
        s.transition(Event::Play).unwrap();
        s.transition(Event::Pause).unwrap();
        s.transition(Event::SkipForward(5)).unwrap();
        s.transition(Event::SkipBackward(2)).unwrap();
        assert_eq!(s.current_index(), Some(3));
    }

    #[test]
    fn test_token_advanced_to_finished() {
        let mut s = ReadingState::new(1, 300);
        s.transition(Event::Play).unwrap();
        let result = s.transition(Event::TokenAdvanced);
        assert!(result.is_ok());
        assert_eq!(s.current_state(), State::Finished);
    }

    #[test]
    fn test_speed_adjustment() {
        let mut s = make_state();
        s.transition(Event::Play).unwrap();
        s.transition(Event::Pause).unwrap();
        s.transition(Event::SpeedUp(50)).unwrap();
        assert_eq!(s.wpm(), 350);
        s.transition(Event::SpeedDown(100)).unwrap();
        assert_eq!(s.wpm(), 250);
    }

    #[test]
    fn test_invalid_transition() {
        let mut s = make_state();
        // Pause from Idle should fail
        let result = s.transition(Event::Pause);
        assert!(matches!(result, Err(StateError::InvalidTransition { .. })));
    }

    #[test]
    fn test_no_tokens() {
        let mut s = ReadingState::new(0, 300);
        let result = s.transition(Event::Play);
        assert_eq!(result, Err(StateError::NoTokens));
    }
}
