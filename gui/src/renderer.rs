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

fn draw_glyph(buf: &mut [u8], pw: usize, ph: usize, color: [u8; 4], outline: ab_glyph::OutlinedGlyph, offset_x: f32, offset_y: f32) {
    outline.draw(|dx, dy, coverage| {
        let px = offset_x as i32 + dx as i32;
        let py = offset_y as i32 + dy as i32;
        if px >= 0 && px < pw as i32 && py >= 0 && py < ph as i32 {
            let idx = (py as usize * pw + px as usize) * 4;
            blend_pixel(buf, idx, color, (coverage * 255.0) as u8);
        }
    });
}

fn draw_text(
    buf: &mut [u8], pw: usize, ph: usize,
    font: &FontArc, font_size: f32, color: [u8; 4],
    text: &str, start_x: f32, baseline_y: f32,
    orp_highlight: Option<(usize, [u8; 4])>,
) {
    let scaled = font.as_scaled(PxScale::from(font_size));
    let mut cur_x = start_x;
    for (i, c) in text.chars().enumerate() {
        let clr = match orp_highlight {
            Some((idx, accent)) if i == idx => accent,
            _ => color,
        };
        let glyph = scaled.scaled_glyph(c);
        let advance = scaled.h_advance(glyph.id);
        if let Some(outline) = scaled.outline_glyph(glyph) {
            let bb = outline.px_bounds();
            draw_glyph(buf, pw, ph, clr, outline, cur_x + bb.min.x, baseline_y + bb.min.y);
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
        let bg = hex_to_rgba(&self.colors.bg);
        fill_rect(buf, pw, ph, bg, 0, 0, pw as i32, ph as i32);
    }

    pub fn render_word(&self, buf: &mut [u8], pw: usize, ph: usize, word: &str, orp_index: usize) {
        let tc = hex_to_rgba(&self.colors.text);
        let ac = hex_to_rgba(&self.colors.accent);
        let scaled = self.font.as_scaled(PxScale::from(self.font_size));

        let before: f32 = word.chars().take(orp_index).map(|c| scaled.h_advance(scaled.scaled_glyph(c).id)).sum();
        let cx = pw as f32 / 2.0;
        let sx = cx - before;
        let by = ph as f32 / 2.0 + self.font_size / 3.0;
        draw_text(buf, pw, ph, &self.font, self.font_size, tc, word, sx, by, Some((orp_index, ac)));
    }

    pub fn render_progress(&self, buf: &mut [u8], pw: usize, ph: usize, cur: usize, total: usize) {
        let text = format!("{}/{}  {} WPM", cur, total, self.wpm);
        let tc = hex_to_rgba(&self.colors.text);
        draw_text(buf, pw, ph, &self.font, self.font_size * 0.35, tc, &text, 10.0, ph as f32 - 10.0, None);
    }
}
