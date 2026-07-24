use std::time::Instant;

use pixels::Pixels;
use speed_reader_core::config::ConfigModel;
use speed_reader_core::state::{Event, ReadingState, State};
use speed_reader_core::timing::TimingEngine;
use speed_reader_core::tokenizer::Token;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize, Position, Size},
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId, WindowLevel},
};

use crate::input::{ActionResult, InputHandler};
use crate::renderer::RSVPRenderer;

pub const W: u32 = 600;
pub const H: u32 = 200;

fn window_attrs() -> WindowAttributes {
    WindowAttributes::default()
        .with_title("SpeedReader")
        .with_inner_size(Size::Logical(LogicalSize::new(W as f64, H as f64)))
        .with_decorations(false)
        .with_transparent(true)
        .with_window_level(WindowLevel::AlwaysOnTop)
        .with_visible(true)
}

pub struct OverlayWindow {
    config: ConfigModel,
    window: Option<Window>,
    pixels: Option<Pixels<'static>>,
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
        Self {
            renderer: RSVPRenderer::new(&config),
            input_handler: InputHandler::new(file_path, tokens.clone(), &text, config.clone()),
            reading_state: ReadingState::new(tokens.len(), config.wpm),
            timing_engine: TimingEngine::new(config.wpm),
            config,
            window: None,
            pixels: None,
            tokens,
            last_advance: Instant::now(),
        }
    }

    pub fn run(mut self) -> Result<(), String> {
        let el = EventLoop::new().map_err(|e| format!("EL: {e}"))?;
        el.run_app(&mut self).map_err(|e| format!("run: {e}"))
    }

    fn advance(&mut self) {
        use State::*;
        if !matches!(self.reading_state.current_state(), Playing { .. }) { return }
        let Some(idx) = self.reading_state.current_index() else { return };
        if idx >= self.tokens.len() { return }
        let t = self.timing_engine.calculate(&self.tokens[idx]);
        if self.last_advance.elapsed() >= t.display_duration + t.pause_after {
            let _ = self.reading_state.transition(Event::TokenAdvanced);
            self.last_advance = Instant::now();
        }
    }
}

impl ApplicationHandler for OverlayWindow {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() { return }
        let mut attr = window_attrs();
        if let Some(m) = el.primary_monitor() {
            let ls = m.size().to_logical::<f64>(m.scale_factor());
            let x = (ls.width - W as f64) / 2.0;
            let y = (ls.height - H as f64) / 2.0;
            attr = attr.with_position(Position::Logical(LogicalPosition::new(x.max(0.0), y.max(0.0))));
        }
        let window = el.create_window(attr).expect("win");
        let ws = window.inner_size();
        let pw = ws.width.max(1);
        let ph = ws.height.max(1);
        let st = pixels::SurfaceTexture::new(pw, ph, &window);
        let px = Pixels::new(pw, ph, st).expect("pixels");
        let px: Pixels<'static> = unsafe { std::mem::transmute(px) };
        self.pixels = Some(px);
        self.window = Some(window);
        let _ = self.reading_state.transition(Event::Play);
        self.last_advance = Instant::now();
        if let Some(w) = &self.window { w.request_redraw(); }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _: WindowId, ev: WindowEvent) {
        match ev {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::KeyboardInput {
                event: KeyEvent { logical_key: key, state: ElementState::Pressed, .. }, ..
            } => {
                let a = self.input_handler.handle_key(&key, &mut self.reading_state);
                use ActionResult::*;
                match a {
                    Continue => {}
                    Exit => el.exit(),
                    Render | PausedAndJumped => { if let Some(w) = &self.window { w.request_redraw(); } }
                    SpeedChanged(wpm) => { self.timing_engine.set_wpm(wpm); self.renderer.set_wpm(wpm); if let Some(w) = &self.window { w.request_redraw(); } }
                    Restarted => { self.last_advance = Instant::now(); if let Some(w) = &self.window { w.request_redraw(); } }
                }
            }
            WindowEvent::RedrawRequested => {
                self.advance();
                if let (Some(w), Some(px)) = (self.window.as_ref(), self.pixels.as_mut()) {
                    let ws = w.inner_size();
                    let pw = ws.width.max(1) as usize;
                    let ph = ws.height.max(1) as usize;

                    let frame = px.frame_mut();
                    // Clear to transparent
                    frame.fill(0);

                    self.renderer.clear(frame, pw, ph);

                    if let Some(idx) = self.reading_state.current_index() {
                        if idx < self.tokens.len() {
                            let t = &self.tokens[idx];
                            self.renderer.render_word(frame, pw, ph, &t.word, t.orp_index);
                            self.renderer.render_progress(frame, pw, ph, idx + 1, self.tokens.len());
                        }
                    }

                    let _ = px.render();

                    if matches!(self.reading_state.current_state(), State::Playing { .. }) {
                        w.request_redraw();
                    }
                }
            }
            _ => {}
        }
    }
}
