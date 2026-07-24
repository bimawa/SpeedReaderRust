use speed_reader_core::config::ConfigModel;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize, Position, Size},
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes, WindowId, WindowLevel},
};

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
    file_path: Option<String>,
    config: ConfigModel,
    window: Option<Window>,
}

impl OverlayWindow {
    pub fn new(file_path: Option<String>, config: ConfigModel) -> Self {
        Self {
            file_path,
            config,
            window: None,
        }
    }

    pub fn file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }

    pub fn config(&self) -> &ConfigModel {
        &self.config
    }

    pub fn run(mut self) -> Result<(), String> {
        let event_loop = EventLoop::new().map_err(|e| format!("Failed to create event loop: {e}"))?;

        event_loop
            .run_app(&mut self)
            .map_err(|e| format!("Event loop error: {e}"))
    }
}

impl ApplicationHandler for OverlayWindow {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let mut attrs = build_window_attributes(&self.config);
            // Center window on primary monitor
            if let Some(monitor) = event_loop.primary_monitor() {
                let monitor_size = monitor.size();
                let logical_size: LogicalSize<f64> = monitor_size.to_logical(monitor.scale_factor());
                let x: f64 = (logical_size.width - DEFAULT_WIDTH as f64) / 2.0;
                let y: f64 = (logical_size.height - DEFAULT_HEIGHT as f64) / 2.0;
                attrs = attrs.with_position(Position::Logical(
                    winit::dpi::LogicalPosition::new(x.max(0.0), y.max(0.0)),
                ));
            }
            let window = event_loop
                .create_window(attrs)
                .map_err(|e| format!("Failed to create window: {e}"))
                .unwrap();
            self.window = Some(window);
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
                handle_key(&key, event_loop);
            }
            WindowEvent::RedrawRequested => {
                // Rendering handled by RSVPRenderer (task 2.2)
            }
            _ => {}
        }
    }
}

fn handle_key(key: &Key, event_loop: &ActiveEventLoop) {
    match key {
        Key::Named(NamedKey::Escape) => {
            event_loop.exit();
        }
        Key::Named(NamedKey::Space)
        | Key::Named(NamedKey::ArrowLeft)
        | Key::Named(NamedKey::ArrowRight)
        | Key::Named(NamedKey::ArrowUp)
        | Key::Named(NamedKey::ArrowDown) => {
            // Input handling delegated to InputHandler in task 2.3
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speed_reader_core::config::{Theme, ThemeColors};

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

    #[test]
    fn overlay_new_with_file_path() {
        let config = test_config();
        let overlay = OverlayWindow::new(Some("test.txt".into()), config);
        assert_eq!(overlay.file_path(), Some("test.txt"));
    }

    #[test]
    fn overlay_new_without_file_path() {
        let config = test_config();
        let overlay = OverlayWindow::new(None, config);
        assert!(overlay.file_path().is_none());
    }

    #[test]
    fn overlay_stores_config() {
        let config = test_config();
        let overlay = OverlayWindow::new(None, config);
        assert_eq!(overlay.config().wpm, 300);
        assert_eq!(
            overlay.config().theme_mode,
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
}
