use std::time::Instant;

use pixels::Pixels;
use speed_reader_core::config::ConfigModel;
use speed_reader_core::state::{Event, ReadingState, State};
use speed_reader_core::timing::TimingEngine;
use speed_reader_core::tokenizer::Token;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize, Position},
    event::{ElementState, KeyEvent, WindowEvent, MouseButton},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::Key,
    window::{Window, WindowAttributes, WindowId, WindowLevel},
};

use crate::input::{ActionResult, InputHandler};
use crate::renderer::RSVPRenderer;

pub const W: u32 = 600;
pub const H: u32 = 200;

fn wa() -> WindowAttributes {
    WindowAttributes::default()
        .with_title("SpeedReader")
        .with_inner_size(winit::dpi::Size::Logical(LogicalSize::new(W as f64, H as f64)))
        .with_decorations(false).with_transparent(true)
        .with_window_level(WindowLevel::AlwaysOnTop).with_visible(true)
}

pub struct OverlayWindow {
    window: Option<Window>,
    pixels: Option<Pixels<'static>>,
    renderer: RSVPRenderer,
    input_handler: InputHandler,
    reading_state: ReadingState,
    config: ConfigModel,
    tokens: Vec<Token>,
    timing_engine: TimingEngine,
    last: Instant,
    drag_origin: Option<(f64, f64, f64, f64)>,
    pub show_settings: bool,
}

impl OverlayWindow {
    pub fn new(fp: Option<String>, config: ConfigModel, text: String, tokens: Vec<Token>) -> Self {
        Self {
            renderer: RSVPRenderer::new(&config),
            input_handler: InputHandler::new(fp, tokens.clone(), &text, config.clone()),
            reading_state: ReadingState::new(tokens.len(), config.wpm),
            timing_engine: TimingEngine::new(config.wpm),
            config, window: None, pixels: None, tokens,
            last: Instant::now(), drag_origin: None, show_settings: false,
        }
    }

    pub fn run(mut self) -> Result<(), String> {
        let el = EventLoop::new().map_err(|e| format!("EL: {e}"))?;
        el.run_app(&mut self).map_err(|e| format!("run: {e}"))
    }

    fn adv(&mut self) {
        if !matches!(self.reading_state.current_state(), State::Playing { .. }) { return }
        let Some(idx) = self.reading_state.current_index() else { return };
        if idx >= self.tokens.len() { return }
        let t = self.timing_engine.calculate(&self.tokens[idx]);
        if self.last.elapsed() >= t.display_duration + t.pause_after {
            let _ = self.reading_state.transition(Event::TokenAdvanced);
            self.last = Instant::now();
        }
    }
}

impl ApplicationHandler for OverlayWindow {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() { return }
        let mut a = wa();
        if let Some(m) = el.primary_monitor() {
            let ls = m.size().to_logical::<f64>(m.scale_factor());
            a = a.with_position(Position::Logical(LogicalPosition::new(
                ((ls.width - W as f64) / 2.0).max(0.0),
                ((ls.height - H as f64) / 2.0).max(0.0),
            )));
        }
        let w = el.create_window(a).expect("win");
        let ws = w.inner_size();
        let st = pixels::SurfaceTexture::new(ws.width.max(1), ws.height.max(1), &w);
        let px = Pixels::new(ws.width.max(1), ws.height.max(1), st).expect("px");
        self.pixels = Some(unsafe { std::mem::transmute(px) });
        self.window = Some(w);
        let _ = self.reading_state.transition(Event::Play);
        self.last = Instant::now();
        if let Some(w) = &self.window { w.request_redraw(); }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _: WindowId, ev: WindowEvent) {
        match ev {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                match state {
                    ElementState::Pressed => {
                        if let Some(w) = self.window.as_ref() {
                            self.drag_origin = w.outer_position().ok().map(|p| (p.x as f64, p.y as f64, 0.0, 0.0));
                        }
                    }
                    ElementState::Released => self.drag_origin = None,
                    _ => {}
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some((wx, wy, _, _)) = self.drag_origin {
                    if let Some(w) = self.window.as_ref() {
                        let scale = w.scale_factor();
                        w.set_outer_position(Position::Logical(LogicalPosition::new(
                            (wx / scale + position.x / scale).max(0.0),
                            (wy / scale + position.y / scale).max(0.0),
                        )));
                        self.drag_origin = None;
                    }
                }
            }
            WindowEvent::KeyboardInput { event: KeyEvent { logical_key, state: ElementState::Pressed, .. }, .. } => {
                if let Key::Character(c) = &logical_key {
                    if c == "s" || c == "S" {
                        self.show_settings = !self.show_settings;
                        if let Some(w) = &self.window { w.request_redraw(); }
                        return;
                    }
                }
                let a = self.input_handler.handle_key(&logical_key, &mut self.reading_state);
                use ActionResult::*;
                match a {
                    Continue => {}
                    Exit => el.exit(),
                    Render | PausedAndJumped => { if let Some(w) = &self.window { w.request_redraw(); } }
                    SpeedChanged(wpm) => { self.timing_engine.set_wpm(wpm); self.renderer.set_wpm(wpm); if let Some(w) = &self.window { w.request_redraw(); } }
                    Restarted => { self.last = Instant::now(); if let Some(w) = &self.window { w.request_redraw(); } }
                }
            }
            WindowEvent::RedrawRequested => {
                self.adv();
                if let (Some(w), Some(px)) = (self.window.as_ref(), self.pixels.as_mut()) {
                    let ws = w.inner_size();
                    let pw = ws.width.max(1) as usize;
                    let ph = ws.height.max(1) as usize;
                    let frame = px.frame_mut();
                    frame.fill(0);
                    self.renderer.clear(frame, pw, ph);
                    if let Some(idx) = self.reading_state.current_index() {
                        if idx < self.tokens.len() {
                            let t = &self.tokens[idx];
                            let paused = matches!(self.reading_state.current_state(), State::Paused { .. });
                            self.renderer.render_word(frame, pw, ph, &t.word, t.orp_index);
                            self.renderer.render_progress(frame, pw, ph, idx + 1, self.tokens.len(), paused);
                        }
                    }
                    if self.show_settings {
                        self.renderer.render_settings(frame, pw, ph, &self.config, self.timing_engine.wpm());
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
