use std::time::Instant;

use speed_reader_core::config::ConfigModel;
use speed_reader_core::state::{Event, ReadingState, State};
use speed_reader_core::timing::TimingEngine;
use speed_reader_core::tokenizer::Token;
use skia_safe::surfaces;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize, Position, Size},
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId, WindowLevel},
};

use crate::input::{ActionResult, InputHandler};
use crate::renderer::RSVPRenderer;

pub const DEFAULT_WIDTH: u32 = 600;
pub const DEFAULT_HEIGHT: u32 = 200;

pub fn build_window_attributes(config: &ConfigModel) -> WindowAttributes {
    let _ = config;
    let size = Size::Logical(LogicalSize::new(
        DEFAULT_WIDTH as f64,
        DEFAULT_HEIGHT as f64,
    ));

    WindowAttributes::default()
        .with_title("SpeedReader")
        .with_inner_size(size)
        .with_decorations(false)
        .with_transparent(true)
        .with_window_level(WindowLevel::AlwaysOnTop)
        .with_visible(true)
}

pub struct OverlayWindow {
    config: ConfigModel,
    window: Option<Window>,
    renderer: RSVPRenderer,
    input_handler: InputHandler,
    reading_state: ReadingState,
    tokens: Vec<Token>,
    timing_engine: TimingEngine,
    last_advance: Instant,
}

impl OverlayWindow {
    pub fn new(
        file_path: Option<String>,
        config: ConfigModel,
        text: String,
        tokens: Vec<Token>,
    ) -> Self {
        let renderer = RSVPRenderer::new(&config);
        let input_handler = InputHandler::new(
            file_path.clone(),
            tokens.clone(),
            &text,
            config.clone(),
        );
        let reading_state = ReadingState::new(tokens.len(), config.wpm);
        let timing_engine = TimingEngine::new(config.wpm);

        Self {
            config,
            window: None,
            renderer,
            input_handler,
            reading_state,
            tokens,
            timing_engine,
            last_advance: Instant::now(),
        }
    }
    pub fn run(mut self) -> Result<(), String> {
        let event_loop = EventLoop::new().map_err(|e| format!("Failed to create event loop: {e}"))?;
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        event_loop
            .run_app(&mut self)
            .map_err(|e| format!("Event loop error: {e}"))
    }


    fn advance_if_due(&mut self) {
        if !matches!(self.reading_state.current_state(), State::Playing { .. }) {
            return;
        }

        let token_index = match self.reading_state.current_index() {
            Some(idx) => idx,
            None => return,
        };

        if token_index >= self.tokens.len() {
            return;
        }

        let timed = self.timing_engine.calculate(&self.tokens[token_index]);
        let elapsed = self.last_advance.elapsed();

        let total_duration = timed.display_duration + timed.pause_after;
        if elapsed >= total_duration {
            let _ = self.reading_state.transition(Event::TokenAdvanced);
            self.last_advance = Instant::now();
        }
    }
}

impl ApplicationHandler for OverlayWindow {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {

            let mut attrs = build_window_attributes(&self.config);
            if let Some(monitor) = event_loop.primary_monitor() {
                let monitor_size = monitor.size();
                let logical_size: LogicalSize<f64> = monitor_size.to_logical(monitor.scale_factor());
                let x: f64 = (logical_size.width - DEFAULT_WIDTH as f64) / 2.0;
                let y: f64 = (logical_size.height - DEFAULT_HEIGHT as f64) / 2.0;
                attrs = attrs.with_position(Position::Logical(
                    LogicalPosition::new(x.max(0.0), y.max(0.0)),
                ));
            }

            let window = event_loop
                .create_window(attrs)
                .map_err(|e| format!("Failed to create window: {e}"))
                .unwrap();

            self.window = Some(window);

            let _ = self.reading_state.transition(Event::Play);
            self.last_advance = Instant::now();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: key,
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                let action = self.input_handler.handle_key(&key, &mut self.reading_state);
                match action {
                    ActionResult::Continue => {}
                    ActionResult::Exit => {
                        event_loop.exit();
                    }
                    ActionResult::Render => {
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                    }
                    ActionResult::PausedAndJumped => {
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                    }
                    ActionResult::SpeedChanged(wpm) => {
                        self.timing_engine.set_wpm(wpm);
                        self.renderer.set_wpm(wpm);
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                    }
                    ActionResult::Restarted => {
                        self.last_advance = Instant::now();
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                self.advance_if_due();

                if let Some(w) = &self.window {
                    let size = w.inner_size();
                    let scale = w.scale_factor();
                    let width = size.width as f32 / scale as f32;
                    let height = size.height as f32 / scale as f32;

                    let mut surface = surfaces::raster_n32_premul((size.width as i32, size.height as i32))
                        .unwrap();
                    let canvas = surface.canvas();

                    self.renderer.clear(canvas, width, height);

                    if let Some(token_index) = self.reading_state.current_index() {
                        if token_index < self.tokens.len() {
                            let token = &self.tokens[token_index];
                            self.renderer.render_word(
                                canvas,
                                &token.word,
                                token.orp_index,
                                width,
                                height,
                            );

                            self.renderer.render_progress(
                                canvas,
                                token_index + 1,
                                self.tokens.len(),
                                width,
                                height,
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speed_reader_core::config::{Theme, ThemeColors};
    use winit::keyboard::Key;

    fn test_config() -> ConfigModel {
        ConfigModel {
            wpm: 300,
            theme_mode: speed_reader_core::config::ThemeMode::Dark,
            font_size: 48.0,
            skip_amount: 5,
            speed_step: 10,
            theme: Theme {
                light: ThemeColors {
                    bg: "#FFFFFF".into(),
                    text: "#1A1A1A".into(),
                    accent: "#E53935".into(),
                },
                dark: ThemeColors {
                    bg: "#1A1A1A".into(),
                    text: "#F5F5F5".into(),
                    accent: "#FF5252".into(),
                },
            },
        }
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
    fn overlay_new_with_all_components() {
        let config = test_config();
        let tokens = sample_tokens();
        let overlay = OverlayWindow::new(
            Some("test.txt".into()),
            config,
            sample_text().to_string(),
            tokens,
        );
        assert_eq!(overlay.tokens.len(), 3);
        assert_eq!(overlay.config.wpm, 300);
    }

    #[test]
    fn overlay_new_without_file_path() {
        let config = test_config();
        let tokens = sample_tokens();
        let overlay = OverlayWindow::new(None, config, sample_text().to_string(), tokens);
        assert_eq!(overlay.tokens.len(), 3);
    }

    #[test]
    fn overlay_stores_config() {
        let config = test_config();
        let tokens = sample_tokens();
        let overlay = OverlayWindow::new(None, config, sample_text().to_string(), tokens);
        assert_eq!(overlay.config.wpm, 300);
        assert_eq!(
            overlay.config.theme_mode,
            speed_reader_core::config::ThemeMode::Dark
        );
    }

    #[test]
    fn overlay_default_window_size() {
        let attrs = build_window_attributes(&test_config());
        let size = attrs.inner_size.unwrap();
        match size {
            Size::Logical(logical) => {
                assert_eq!(logical.width, DEFAULT_WIDTH as f64);
                assert_eq!(logical.height, DEFAULT_HEIGHT as f64);
            }
            other => panic!("Expected logical size, got: {other:?}"),
        }
    }

    #[test]
    fn overlay_window_is_frameless() {
        let attrs = build_window_attributes(&test_config());
        assert!(!attrs.decorations, "window should be frameless");
    }

    #[test]
    fn overlay_window_is_transparent() {
        let attrs = build_window_attributes(&test_config());
        assert!(attrs.transparent, "window should be transparent");
    }

    #[test]
    fn overlay_window_is_always_on_top() {
        let attrs = build_window_attributes(&test_config());
        assert_eq!(
            attrs.window_level,
            WindowLevel::AlwaysOnTop,
            "window should be always on top"
        );
    }

    #[test]
    fn overlay_window_title() {
        let attrs = build_window_attributes(&test_config());
        assert_eq!(
            attrs.title, "SpeedReader",
            "window title should be SpeedReader"
        );
    }

    #[test]
    fn overlay_window_visible() {
        let attrs = build_window_attributes(&test_config());
        assert!(attrs.visible, "window should be visible");
    }

    #[test]
    fn keyboard_space_pauses_playing_state() {
        let config = test_config();
        let tokens = sample_tokens();
        let mut overlay = OverlayWindow::new(
            Some("test.rs".into()),
            config,
            sample_text().to_string(),
            tokens,
        );
        let _ = overlay.reading_state.transition(Event::Play);

        let result = overlay.input_handler.handle_key(
            &Key::Named(winit::keyboard::NamedKey::Space),
            &mut overlay.reading_state,
        );

        assert_eq!(result, ActionResult::PausedAndJumped);
        assert!(matches!(overlay.reading_state.current_state(), State::Paused { .. }));
    }

    #[test]
    fn keyboard_space_resumes_paused_state() {
        let config = test_config();
        let tokens = sample_tokens();
        let mut overlay = OverlayWindow::new(
            Some("test.rs".into()),
            config,
            sample_text().to_string(),
            tokens,
        );
        let _ = overlay.reading_state.transition(Event::Play);
        let _ = overlay.reading_state.transition(Event::Pause);

        let result = overlay.input_handler.handle_key(
            &Key::Named(winit::keyboard::NamedKey::Space),
            &mut overlay.reading_state,
        );

        assert_eq!(result, ActionResult::Continue);
        assert!(matches!(overlay.reading_state.current_state(), State::Playing { .. }));
    }

    #[test]
    fn keyboard_escape_returns_exit() {
        let config = test_config();
        let tokens = sample_tokens();
        let mut overlay = OverlayWindow::new(
            Some("test.rs".into()),
            config,
            sample_text().to_string(),
            tokens,
        );
        let _ = overlay.reading_state.transition(Event::Play);

        let result = overlay.input_handler.handle_key(
            &Key::Named(winit::keyboard::NamedKey::Escape),
            &mut overlay.reading_state,
        );

        assert_eq!(result, ActionResult::Exit);
        assert!(matches!(overlay.reading_state.current_state(), State::Idle));
    }

    #[test]
    fn keyboard_arrows_skip_when_paused() {
        let config = test_config();
        let tokens = sample_tokens();
        let mut overlay = OverlayWindow::new(
            Some("test.rs".into()),
            config,
            sample_text().to_string(),
            tokens,
        );
        let _ = overlay.reading_state.transition(Event::Play);
        let _ = overlay.reading_state.transition(Event::Pause);

        let result = overlay.input_handler.handle_key(
            &Key::Named(winit::keyboard::NamedKey::ArrowRight),
            &mut overlay.reading_state,
        );

        assert_eq!(result, ActionResult::Render);
    }

    #[test]
    fn timer_advances_to_next_token_when_playing() {
        let config = test_config();
        let tokens = sample_tokens();
        let mut overlay = OverlayWindow::new(
            Some("test.rs".into()),
            config,
            sample_text().to_string(),
            tokens,
        );
        let _ = overlay.reading_state.transition(Event::Play);
        overlay.last_advance = Instant::now() - std::time::Duration::from_secs(10);

        overlay.advance_if_due();

        assert!(matches!(
            overlay.reading_state.current_state(),
            State::Playing { token_index: 1 }
        ));
    }

    #[test]
    fn timer_does_not_advance_when_paused() {
        let config = test_config();
        let tokens = sample_tokens();
        let mut overlay = OverlayWindow::new(
            Some("test.rs".into()),
            config,
            sample_text().to_string(),
            tokens,
        );
        let _ = overlay.reading_state.transition(Event::Play);
        let _ = overlay.reading_state.transition(Event::Pause);
        overlay.last_advance = Instant::now() - std::time::Duration::from_secs(10);

        overlay.advance_if_due();

        assert!(matches!(overlay.reading_state.current_state(), State::Paused { token_index: 0 }));
    }

    #[test]
    fn timer_does_not_advance_before_duration_elapsed() {
        let config = test_config();
        let tokens = sample_tokens();
        let mut overlay = OverlayWindow::new(
            Some("test.rs".into()),
            config,
            sample_text().to_string(),
            tokens,
        );
        let _ = overlay.reading_state.transition(Event::Play);
        overlay.last_advance = Instant::now();

        overlay.advance_if_due();

        assert!(matches!(
            overlay.reading_state.current_state(),
            State::Playing { token_index: 0 }
        ));
    }

    #[test]
    fn timer_stops_at_finished_state() {
        let config = test_config();
        let tokens = vec![
            Token { word: "only".into(), orp_index: 1, byte_offset: 0, byte_len: 4, is_sentence_end: false },
        ];
        let mut overlay = OverlayWindow::new(
            Some("test.rs".into()),
            config,
            "only".to_string(),
            tokens,
        );
        let _ = overlay.reading_state.transition(Event::Play);
        overlay.last_advance = Instant::now() - std::time::Duration::from_secs(10);

        overlay.advance_if_due();

        assert!(matches!(overlay.reading_state.current_state(), State::Finished));
    }
}
