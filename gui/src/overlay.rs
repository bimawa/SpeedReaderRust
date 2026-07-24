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

const W: f64 = 600.0;
const H: f64 = 200.0;

fn wa() -> WindowAttributes {
    WindowAttributes::default()
        .with_title("SpeedReader")
        .with_inner_size(winit::dpi::Size::Logical(LogicalSize::new(W, H)))
        .with_decorations(false).with_transparent(true)
        .with_window_level(WindowLevel::AlwaysOnTop).with_visible(true)
}

pub struct OverlayWindow {
    window: Option<Window>,
    pixels: Option<Pixels<'static>>,
    renderer: RSVPRenderer,
    input_handler: InputHandler,
    state: ReadingState,
    config: ConfigModel,
    tokens: Vec<Token>,
    timing: TimingEngine,
    last: Instant,
    drg: Option<(f64, f64, f64, f64)>,
    cursor: Option<(f64, f64)>,
    pub settings: bool,
}

impl OverlayWindow {
    pub fn new(fp: Option<String>, config: ConfigModel, text: String, tokens: Vec<Token>) -> Self {
        Self {
            renderer: RSVPRenderer::new(&config),
            input_handler: InputHandler::new(fp, tokens.clone(), &text, config.clone()),
            state: ReadingState::new(tokens.len(), config.wpm),
            timing: TimingEngine::new(config.wpm),
            config, window: None, pixels: None, tokens,
            last: Instant::now(), drg: None, cursor: None, settings: false,
        }
    }

    pub fn run(mut self) -> Result<(), String> {
        let el = EventLoop::new().map_err(|e| format!("EL: {e}"))?;
        el.run_app(&mut self).map_err(|e| format!("run: {e}"))
    }

    fn adv(&mut self) {
        if !matches!(self.state.current_state(), State::Playing { .. }) { return }
        let Some(i) = self.state.current_index() else { return };
        if i >= self.tokens.len() { return }
        let t = self.timing.calculate(&self.tokens[i]);
        if self.last.elapsed() >= t.display_duration + t.pause_after {
            let _ = self.state.transition(Event::TokenAdvanced);
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
                ((ls.width - W) / 2.0).max(0.0), ((ls.height - H) / 2.0).max(0.0),
            )));
        }
        let w = el.create_window(a).expect("win");
        let ws = w.inner_size();
        let st = pixels::SurfaceTexture::new(ws.width.max(1), ws.height.max(1), &w);
        let px = Pixels::new(ws.width.max(1), ws.height.max(1), st).expect("px");
        self.pixels = Some(unsafe { std::mem::transmute(px) });
        self.window = Some(w);
        let _ = self.state.transition(Event::Play);
        self.last = Instant::now();
        if let Some(w) = &self.window { w.request_redraw(); }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _: WindowId, ev: WindowEvent) {
        match ev {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                if state == ElementState::Pressed {
                    if let (Some(w), Some(cp)) = (self.window.as_ref(), self.cursor) {
                        if let Ok(wp) = w.outer_position() {
                            self.drg = Some((wp.x as f64, wp.y as f64, cp.0, cp.1));
                        }
                    }
                } else {
                    self.drg = None;
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = Some((position.x, position.y));
                if let Some((wx, wy, cx, cy)) = self.drg {
                    if let Some(w) = self.window.as_ref() {
                        w.set_outer_position(Position::Physical(winit::dpi::PhysicalPosition::new(
                            (wx + position.x - cx).max(0.0) as i32,
                            (wy + position.y - cy).max(0.0) as i32,
                        )));
                    }
                }
            }
            WindowEvent::KeyboardInput { event: KeyEvent { logical_key, state: ElementState::Pressed, .. }, .. } => {
                if let Key::Character(c) = &logical_key {
                    if c == "s" || c == "S" {
                        self.settings = !self.settings;
                        if let Some(w) = &self.window { w.request_redraw(); }
                        return;
                    }
                }
                let r = self.input_handler.handle_key(&logical_key, &mut self.state);
                use ActionResult::*;
                match r {
                    Continue => {}
                    Exit => el.exit(),
                    Render | PausedAndJumped => { if let Some(w) = &self.window { w.request_redraw(); } }
                    SpeedChanged(v) => { self.timing.set_wpm(v); self.renderer.set_wpm(v); if let Some(w) = &self.window { w.request_redraw(); } }
                    Restarted => { self.last = Instant::now(); if let Some(w) = &self.window { w.request_redraw(); } }
                }
            }
            WindowEvent::RedrawRequested => {
                self.adv();
                if let (Some(w), Some(px)) = (self.window.as_ref(), self.pixels.as_mut()) {
                    let ws = w.inner_size();
                    let pw = ws.width.max(1) as usize;
                    let ph = ws.height.max(1) as usize;
                    let fb = px.frame_mut();
                    fb.fill(0);
                    self.renderer.clear(fb, pw, ph);
                    if let Some(i) = self.state.current_index() {
                        if i < self.tokens.len() {
                            let t = &self.tokens[i];
                            let p = matches!(self.state.current_state(), State::Paused { .. });
                            self.renderer.word(fb, pw, ph, &t.word, t.orp_index);
                            self.renderer.progress(fb, pw, ph, i + 1, self.tokens.len(), p);
                        }
                    }
                    if self.settings {
                        self.renderer.settings(fb, pw, ph, &self.config, self.timing.wpm());
                    }
                    let _ = px.render();
                    if matches!(self.state.current_state(), State::Playing { .. }) {
                        w.request_redraw();
                    }
                }
            }
            _ => {}
        }
    }
}
