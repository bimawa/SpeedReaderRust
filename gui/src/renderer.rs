use ab_glyph::{FontArc, Font, PxScale, ScaleFont};
use speed_reader_core::config::ConfigModel;

fn hex_to_rgba(hex: &str) -> [u8; 4] {
    let h = hex.trim_start_matches('#');
    [
        u8::from_str_radix(&h[0..2], 16).unwrap_or(0),
        u8::from_str_radix(&h[2..4], 16).unwrap_or(0),
        u8::from_str_radix(&h[4..6], 16).unwrap_or(0),
        255,
    ]
}

fn blend_pixel(buf: &mut [u8], idx: usize, color: [u8; 4], alpha: u8) {
    let a = alpha as u32;
    let ia = 255 - a;
    buf[idx] = ((color[0] as u32 * a + buf[idx] as u32 * ia) / 255) as u8;
    buf[idx + 1] = ((color[1] as u32 * a + buf[idx + 1] as u32 * ia) / 255) as u8;
    buf[idx + 2] = ((color[2] as u32 * a + buf[idx + 2] as u32 * ia) / 255) as u8;
    buf[idx + 3] = 255;
}

fn fill_rect(buf: &mut [u8], pw: usize, ph: usize, color: [u8; 4], x: i32, y: i32, w: i32, h: i32) {
    for dy in y.max(0)..(y + h).min(ph as i32) {
        for dx in x.max(0)..(x + w).min(pw as i32) {
            let idx = (dy as usize * pw + dx as usize) * 4;
            blend_pixel(buf, idx, color, 255);
        }
    }
}

fn draw_text_raw(
    buf: &mut [u8], pw: usize, ph: usize,
    font: &FontArc, font_size: f32, color: [u8; 4],
    text: &str, start_x: f32, baseline_y: f32,
) {
    let scaled = font.as_scaled(PxScale::from(font_size));
    let mut cur_x = start_x;
    for c in text.chars() {
        let glyph = scaled.scaled_glyph(c);
        let advance = scaled.h_advance(glyph.id);
        if let Some(outline) = scaled.outline_glyph(glyph) {
            let bb = outline.px_bounds();
            outline.draw(|dx, dy, coverage| {
                let px = (cur_x + bb.min.x) as i32 + dx as i32;
                let py = (baseline_y + bb.min.y) as i32 + dy as i32;
                if px >= 0 && px < pw as i32 && py >= 0 && py < ph as i32 {
                    let idx = (py as usize * pw + px as usize) * 4;
                    blend_pixel(buf, idx, color, (coverage * 255.0) as u8);
                }
            });
        }
        cur_x += advance;
    }
}

pub struct RSVPRenderer {
    font_size: f32,
    colors: speed_reader_core::config::ThemeColors,
    wpm: u32,
    font: FontArc,
}

impl RSVPRenderer {
    pub fn new(config: &ConfigModel) -> Self {
        let fd: &[u8] = include_bytes!("../../assets/NotoSans-Regular.ttf");
        Self {
            font_size: config.font_size,
            colors: config.current_colors().clone(),
            wpm: config.wpm,
            font: FontArc::try_from_slice(fd).expect("font"),
        }
    }

    pub fn set_wpm(&mut self, wpm: u32) { self.wpm = wpm; }
    pub fn current_wpm(&self) -> u32 { self.wpm }

    pub fn clear(&self, buf: &mut [u8], pw: usize, ph: usize) {
        fill_rect(buf, pw, ph, hex_to_rgba(&self.colors.bg), 0, 0, pw as i32, ph as i32);
    }

    pub fn render_word(&self, buf: &mut [u8], pw: usize, ph: usize, word: &str, orp_index: usize) {
        let tc = hex_to_rgba(&self.colors.text);
        let ac = hex_to_rgba(&self.colors.accent);
        let scaled = self.font.as_scaled(PxScale::from(self.font_size));
        let before: f32 = word.chars().take(orp_index).map(|c| scaled.h_advance(scaled.scaled_glyph(c).id)).sum();
        let cx = pw as f32 / 2.0;
        let sx = cx - before;
        let by = ph as f32 / 2.0 + self.font_size / 3.0;

        let mut cur_x = sx;
        for (i, c) in word.chars().enumerate() {
            let clr = if i == orp_index { ac } else { tc };
            let glyph = scaled.scaled_glyph(c);
            let advance = scaled.h_advance(glyph.id);
            if let Some(outline) = scaled.outline_glyph(glyph) {
                let bb = outline.px_bounds();
                outline.draw(|dx, dy, coverage| {
                    let px = (cur_x + bb.min.x) as i32 + dx as i32;
                    let py = (by + bb.min.y) as i32 + dy as i32;
                    if px >= 0 && px < pw as i32 && py >= 0 && py < ph as i32 {
                        let idx = (py as usize * pw + px as usize) * 4;
                        blend_pixel(buf, idx, clr, (coverage * 255.0) as u8);
                    }
                });
            }
            cur_x += advance;
        }
    }

    pub fn render_progress(&self, buf: &mut [u8], pw: usize, ph: usize, cur: usize, total: usize, paused: bool) {
        let tc = hex_to_rgba(&self.colors.text);
        let ac = hex_to_rgba(&self.colors.accent);
        let sz = self.font_size * 0.3;

        // Play/pause indicator — top right
        let indicator = if paused { "⏸ PAUSED" } else { "▶" };
        let indicator_color = if paused { ac } else { tc };
        let ind_w = indicator.len() as f32 * sz * 0.6;
        draw_text_raw(buf, pw, ph, &self.font, sz, indicator_color, indicator,
            pw as f32 - ind_w - 10.0, 20.0 + sz);

        // Speed — top left
        let speed_text = format!("{} WPM", self.wpm);
        draw_text_raw(buf, pw, ph, &self.font, sz, tc, &speed_text, 10.0, 20.0 + sz);

        // Progress — bottom left
        let prog_text = format!("{}/{}", cur, total);
        draw_text_raw(buf, pw, ph, &self.font, sz, tc, &prog_text, 10.0, ph as f32 - 10.0);

        // Controls hint — bottom right
        let hint = "⏎Pause  ↑↓Speed  ←→Skip  SSettings  EscExit";
        let hint_sz = sz * 0.8;
        draw_text_raw(buf, pw, ph, &self.font, hint_sz, tc, hint,
            10.0, ph as f32 - 10.0 - hint_sz - 4.0);
    }

    pub fn render_settings(&self, buf: &mut [u8], pw: usize, ph: usize, config: &ConfigModel, current_wpm: u32) {
        let bg = [40, 40, 40, 230]; // semi-transparent dark
        let tc = hex_to_rgba(&self.colors.text);
        let ac = hex_to_rgba(&self.colors.accent);
        let sz = self.font_size * 0.35;

        // Semi-transparent overlay
        fill_rect(buf, pw, ph, bg, 0, 0, pw as i32, ph as i32);

        let mut y = 40.0f32;
        let line_h = sz * 1.8;

        // Title
        draw_text_raw(buf, pw, ph, &self.font, sz * 1.3, ac, "Settings", 20.0, y);
        y += line_h * 1.5;

        // WPM
        draw_text_raw(buf, pw, ph, &self.font, sz, tc, &format!("Speed: {} WPM  (↑↓ to change)", current_wpm), 20.0, y);
        y += line_h;

        // Theme
        let theme_str = format!("Theme: {}  (T to toggle)", match config.theme_mode {
            speed_reader_core::config::ThemeMode::Dark => "Dark",
            speed_reader_core::config::ThemeMode::Light => "Light",
        });
        draw_text_raw(buf, pw, ph, &self.font, sz, tc, &theme_str, 20.0, y);
        y += line_h;

        // Skip amount
        draw_text_raw(buf, pw, ph, &self.font, sz, tc, &format!("Skip: {} words", config.skip_amount), 20.0, y);
        y += line_h;

        // Font size
        draw_text_raw(buf, pw, ph, &self.font, sz, tc, &format!("Font: {:.0}px", config.font_size), 20.0, y);
        y += line_h * 1.5;

        // Close hint
        draw_text_raw(buf, pw, ph, &self.font, sz * 0.8, ac, "Press S to close settings", 20.0, y);
    }
}
