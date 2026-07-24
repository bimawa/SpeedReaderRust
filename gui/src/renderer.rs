use speed_reader_core::config::ConfigModel;
use skia_safe::{
    Canvas, Color, Font, FontMgr, FontStyle, Paint, Point, Rect, RRect, TextBlob,
};

pub struct RSVPRenderer {
    font_size: f32,
    bg_color: Color,
    text_color: Color,
    accent_color: Color,
    wpm: u32,
}

fn parse_hex_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    let a = if hex.len() == 8 {
        u8::from_str_radix(&hex[6..8], 16).unwrap_or(255)
    } else {
        255
    };
    Color::from_argb(a, r, g, b)
}

impl RSVPRenderer {
    pub fn new(config: &ConfigModel) -> Self {
        let colors = config.current_colors();
        Self {
            font_size: config.font_size,
            bg_color: parse_hex_color(&colors.bg),
            text_color: parse_hex_color(&colors.text),
            accent_color: parse_hex_color(&colors.accent),
            wpm: config.wpm,
        }
    }
    pub fn set_wpm(&mut self, wpm: u32) {
        self.wpm = wpm;
    }

    pub fn clear(&self, canvas: &Canvas, width: f32, height: f32) {
        canvas.clear(Color::TRANSPARENT);
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(self.bg_color);
        let rect = Rect::new(0.0, 0.0, width, height);
        let rrect = RRect::new_rect_xy(rect, 12.0, 12.0);
        canvas.draw_rrect(rrect, &paint);
    }

    fn make_font(&self) -> Font {
        let font_mgr = FontMgr::default();
        let typeface = font_mgr
            .match_family_style("sans-serif", FontStyle::normal())
            .or_else(|| font_mgr.match_family_style("Arial", FontStyle::normal()));
        match typeface {
            Some(tf) => Font::new(tf, self.font_size),
            None => Font::default().with_size(self.font_size).unwrap_or_else(Font::default),
        }
    }

    pub fn render_word(&self, canvas: &Canvas, word: &str, orp_index: usize, width: f32, height: f32) {
        if word.is_empty() {
            return;
        }

        let font = self.make_font();
        let center_x = width / 2.0;
        let center_y = height / 2.0;

        let chars: Vec<char> = word.chars().collect();
        let orp_idx = orp_index.min(chars.len().saturating_sub(1));

        let char_widths: Vec<f32> = chars
            .iter()
            .map(|ch| {
                let (w, _) = font.measure_str(ch.to_string().as_str(), None);
                w
            })
            .collect();

        let orp_center_x: f32 = char_widths[..orp_idx].iter().sum::<f32>()
            + char_widths[orp_idx] / 2.0;

        let start_x = center_x - orp_center_x;
        let baseline_y = center_y;

        let mut x_offset = 0.0f32;
        for (i, ch) in chars.iter().enumerate() {
            let ch_x = start_x + x_offset;
            let ch_str = ch.to_string();

            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            if i == orp_idx {
                paint.set_color(self.accent_color);
            } else {
                paint.set_color(self.text_color);
            }

            if let Some(blob) = TextBlob::new(ch_str.as_str(), &font) {
                canvas.draw_text_blob(blob, Point::new(ch_x, baseline_y), &paint);
            }

            x_offset += char_widths[i];
        }
    }

    pub fn render_progress(&self, canvas: &Canvas, current: usize, total: usize, width: f32, height: f32) {
        let font_mgr = FontMgr::default();
        let small_typeface = font_mgr
            .match_family_style("sans-serif", FontStyle::normal())
            .or_else(|| font_mgr.match_family_style("Arial", FontStyle::normal()));
        let small_font = match small_typeface {
            Some(tf) => Font::new(tf, 14.0),
            None => Font::default().with_size(14.0).unwrap_or_else(Font::default),
        };

        let progress_text = format!("{}/{}", current, total);
        let wpm_text = format!("{} WPM", self.wpm);

        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(self.text_color);

        if let Some(blob) = TextBlob::new(progress_text.as_str(), &small_font) {
            let bounds = blob.bounds();
            let pw = bounds.width();
            canvas.draw_text_blob(blob, Point::new(width - pw - 8.0, height - 8.0), &paint);
        }

        if let Some(blob) = TextBlob::new(wpm_text.as_str(), &small_font) {
            let bounds = blob.bounds();
            let ww = bounds.width();
            canvas.draw_text_blob(blob, Point::new(width - ww - 8.0, height - 24.0), &paint);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skia_safe::{surfaces, Surface};

    fn make_surface(w: i32, h: i32) -> Surface {
        surfaces::raster_n32_premul((w, h)).unwrap()
    }

    fn default_config() -> ConfigModel {
        ConfigModel::default()
    }

    #[test]
    fn test_create_renderer_default_config() {
        let config = default_config();
        let renderer = RSVPRenderer::new(&config);
        assert_eq!(renderer.font_size, 48.0);
        assert_eq!(renderer.wpm, 300);
    }

    #[test]
    fn test_render_word_does_not_panic() {
        let config = default_config();
        let renderer = RSVPRenderer::new(&config);
        let mut surface = make_surface(600, 200);
        let canvas = surface.canvas();
        renderer.clear(canvas, 600.0, 200.0);
        renderer.render_word(canvas, "hello", 2, 600.0, 200.0);
    }

    #[test]
    fn test_render_word_short_word() {
        let config = default_config();
        let renderer = RSVPRenderer::new(&config);
        let mut surface = make_surface(600, 200);
        let canvas = surface.canvas();
        renderer.clear(canvas, 600.0, 200.0);
        renderer.render_word(canvas, "a", 0, 600.0, 200.0);
        renderer.render_word(canvas, "ab", 0, 600.0, 200.0);
        renderer.render_word(canvas, "ab", 1, 600.0, 200.0);
    }

    #[test]
    fn test_render_word_long_word() {
        let config = default_config();
        let renderer = RSVPRenderer::new(&config);
        let mut surface = make_surface(600, 200);
        let canvas = surface.canvas();
        renderer.clear(canvas, 600.0, 200.0);
        renderer.render_word(canvas, "extraordinary", 5, 600.0, 200.0);
    }

    #[test]
    fn test_render_word_empty_string() {
        let config = default_config();
        let renderer = RSVPRenderer::new(&config);
        let mut surface = make_surface(600, 200);
        let canvas = surface.canvas();
        renderer.clear(canvas, 600.0, 200.0);
        renderer.render_word(canvas, "", 0, 600.0, 200.0);
    }

    #[test]
    fn test_render_progress_does_not_panic() {
        let config = default_config();
        let renderer = RSVPRenderer::new(&config);
        let mut surface = make_surface(600, 200);
        let canvas = surface.canvas();
        renderer.render_progress(canvas, 42, 100, 600.0, 200.0);
    }

    #[test]
    fn test_render_progress_formatting() {
        let config = default_config();
        let renderer = RSVPRenderer::new(&config);
        let mut surface = make_surface(600, 200);
        let canvas = surface.canvas();
        renderer.render_progress(canvas, 1, 1, 600.0, 200.0);
        renderer.render_progress(canvas, 999, 1000, 600.0, 200.0);
        renderer.render_progress(canvas, 0, 0, 600.0, 200.0);
    }

    #[test]
    fn test_orp_color_differs_from_text_color() {
        let config = default_config();
        let renderer = RSVPRenderer::new(&config);
        assert_ne!(renderer.accent_color, renderer.text_color);
    }

    #[test]
    fn test_config_changes_font_size() {
        let mut config = default_config();
        config.font_size = 72.0;
        let renderer = RSVPRenderer::new(&config);
        assert_eq!(renderer.font_size, 72.0);
    }

    #[test]
    fn test_config_different_theme_light() {
        let mut config = default_config();
        config.theme_mode = speed_reader_core::config::ThemeMode::Light;
        let renderer = RSVPRenderer::new(&config);
        let light = &config.theme.light;
        assert_eq!(renderer.bg_color, parse_hex_color(&light.bg));
        assert_eq!(renderer.text_color, parse_hex_color(&light.text));
        assert_eq!(renderer.accent_color, parse_hex_color(&light.accent));
    }

    #[test]
    fn test_config_different_theme_dark() {
        let mut config = default_config();
        config.theme_mode = speed_reader_core::config::ThemeMode::Dark;
        let renderer = RSVPRenderer::new(&config);
        let dark = &config.theme.dark;
        assert_eq!(renderer.bg_color, parse_hex_color(&dark.bg));
        assert_eq!(renderer.text_color, parse_hex_color(&dark.text));
        assert_eq!(renderer.accent_color, parse_hex_color(&dark.accent));
    }

    #[test]
    fn test_orp_index_at_boundary() {
        let config = default_config();
        let renderer = RSVPRenderer::new(&config);
        let mut surface = make_surface(600, 200);
        let canvas = surface.canvas();
        renderer.clear(canvas, 600.0, 200.0);
        renderer.render_word(canvas, "word", 0, 600.0, 200.0);
        renderer.render_word(canvas, "word", 3, 600.0, 200.0);
        renderer.render_word(canvas, "word", 5, 600.0, 200.0);
    }

    #[test]
    fn test_clear_sets_background() {
        let config = default_config();
        let renderer = RSVPRenderer::new(&config);
        let mut surface = make_surface(600, 200);
        let canvas = surface.canvas();
        renderer.clear(canvas, 600.0, 200.0);
        renderer.render_word(canvas, "test", 1, 600.0, 200.0);
    }

    #[test]
    fn test_full_pipeline() {
        let config = default_config();
        let renderer = RSVPRenderer::new(&config);
        let mut surface = make_surface(600, 200);
        let canvas = surface.canvas();
        renderer.clear(canvas, 600.0, 200.0);
        renderer.render_word(canvas, "hello", 2, 600.0, 200.0);
        renderer.render_progress(canvas, 1, 5, 600.0, 200.0);
    }
}
