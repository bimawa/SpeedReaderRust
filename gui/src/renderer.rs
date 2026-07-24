use ab_glyph::{FontArc, Font, PxScale, ScaleFont};
use speed_reader_core::config::ConfigModel;

fn hex(c: &str) -> [u8; 4] {
    let h = c.trim_start_matches('#');
    [u8::from_str_radix(&h[0..2], 16).unwrap_or(0),
     u8::from_str_radix(&h[2..4], 16).unwrap_or(0),
     u8::from_str_radix(&h[4..6], 16).unwrap_or(0), 255]
}

fn blend(buf: &mut [u8], i: usize, c: [u8; 4], a: u8) {
    if i + 3 >= buf.len() { return; }
    let a = a as u32; let ia = 255 - a;
    buf[i] = ((c[0] as u32 * a + buf[i] as u32 * ia) / 255) as u8;
    buf[i+1] = ((c[1] as u32 * a + buf[i+1] as u32 * ia) / 255) as u8;
    buf[i+2] = ((c[2] as u32 * a + buf[i+2] as u32 * ia) / 255) as u8;
    buf[i+3] = 255;
}

fn rect(buf: &mut [u8], pw: usize, ph: usize, c: [u8; 4], x: i32, y: i32, w: i32, h: i32) {
    for dy in y.max(0)..(y+h).min(ph as i32) {
        for dx in x.max(0)..(x+w).min(pw as i32) {
            blend(buf, (dy as usize * pw + dx as usize) * 4, c, 255);
        }
    }
}

fn text(buf: &mut [u8], pw: usize, ph: usize, font: &FontArc, sz: f32, c: [u8; 4], s: &str, x: f32, y: f32) {
    let sc = font.as_scaled(PxScale::from(sz));
    let mut cx = x;
    for ch in s.chars() {
        let g = sc.scaled_glyph(ch);
        let adv = sc.h_advance(g.id);
        if let Some(o) = sc.outline_glyph(g) {
            let bb = o.px_bounds();
            o.draw(|dx, dy, cv| {
                let px = (cx + bb.min.x) as i32 + dx as i32;
                let py = (y + bb.min.y) as i32 + dy as i32;
                if px >= 0 && px < pw as i32 && py >= 0 && py < ph as i32 {
                    blend(buf, (py as usize * pw + px as usize) * 4, c, (cv * 255.0) as u8);
                }
            });
        }
        cx += adv;
    }
}

pub struct RSVPRenderer {
    fs: f32,
    co: speed_reader_core::config::ThemeColors,
    wpm: u32,
    font: FontArc,
}

impl RSVPRenderer {
    pub fn new(cfg: &ConfigModel) -> Self {
        let fd: &[u8] = include_bytes!("../../assets/NotoSans-Regular.ttf");
        Self { fs: cfg.font_size, co: cfg.current_colors().clone(), wpm: cfg.wpm, font: FontArc::try_from_slice(fd).expect("f") }
    }
    pub fn set_wpm(&mut self, v: u32) { self.wpm = v; }
    pub fn current_wpm(&self) -> u32 { self.wpm }

    pub fn clear(&self, buf: &mut [u8], pw: usize, ph: usize) {
        rect(buf, pw, ph, hex(&self.co.bg), 0, 0, pw as i32, ph as i32);
    }

    pub fn word(&self, buf: &mut [u8], pw: usize, ph: usize, w: &str, orp: usize) {
        let tc = hex(&self.co.text);
        let ac = hex(&self.co.accent);
        let sc = self.font.as_scaled(PxScale::from(self.fs));
        let before: f32 = w.chars().take(orp).map(|c| sc.h_advance(sc.scaled_glyph(c).id)).sum();
        let sx = pw as f32 / 2.0 - before;
        let by = ph as f32 / 2.0 + self.fs / 3.0;
        let mut cx = sx;
        for (i, ch) in w.chars().enumerate() {
            let cl = if i == orp { ac } else { tc };
            let g = sc.scaled_glyph(ch);
            let adv = sc.h_advance(g.id);
            if let Some(o) = sc.outline_glyph(g) {
                let bb = o.px_bounds();
                o.draw(|dx, dy, cv| {
                    let px = (cx + bb.min.x) as i32 + dx as i32;
                    let py = (by + bb.min.y) as i32 + dy as i32;
                    if px >= 0 && px < pw as i32 && py >= 0 && py < ph as i32 {
                        blend(buf, (py as usize * pw + px as usize) * 4, cl, (cv * 255.0) as u8);
                    }
                });
            }
            cx += adv;
        }
    }

    pub fn progress(&self, buf: &mut [u8], pw: usize, ph: usize, cur: usize, total: usize, paused: bool) {
        let tc = hex(&self.co.text);
        let ac = hex(&self.co.accent);
        let sz = self.fs * 0.3;

        // Status indicator - top right
        let status = if paused { "[PAUSED]" } else { "[PLAY]" };
        let sc = if paused { ac } else { tc };
        let sw = status.len() as f32 * sz * 0.55;
        text(buf, pw, ph, &self.font, sz, sc, status, pw as f32 - sw - 8.0, 18.0 + sz);

        // WPM - top left
        text(buf, pw, ph, &self.font, sz, tc, &format!("{} wpm", self.wpm), 8.0, 18.0 + sz);

        // Progress - bottom left
        text(buf, pw, ph, &self.font, sz, tc, &format!("{}/{}", cur, total), 8.0, ph as f32 - 8.0);

        // Controls - bottom, centered
        let hints = "Space=Pause  arrows=Skip  Up/Down=Speed  S=Settings  Esc=Exit";
        let hw = hints.len() as f32 * sz * 0.4;
        text(buf, pw, ph, &self.font, sz * 0.75, tc, hints,
            (pw as f32 - hw) / 2.0, ph as f32 - 8.0 - sz * 0.75);
    }

    pub fn settings(&self, buf: &mut [u8], pw: usize, ph: usize, cfg: &ConfigModel, wpm: u32) {
        let bg = [40, 40, 40, 230];
        let tc = hex(&self.co.text);
        let ac = hex(&self.co.accent);
        let sz = self.fs * 0.35;
        rect(buf, pw, ph, bg, 0, 0, pw as i32, ph as i32);
        let mut y = 40.0;
        let lh = sz * 1.8;
        text(buf, pw, ph, &self.font, sz * 1.3, ac, "SETTINGS", 20.0, y);
        y += lh * 1.5;
        text(buf, pw, ph, &self.font, sz, tc, &format!("Speed: {} wpm  (up/down to change)", wpm), 20.0, y);
        y += lh;
        let tm = match cfg.theme_mode { speed_reader_core::config::ThemeMode::Dark => "Dark", _ => "Light" };
        text(buf, pw, ph, &self.font, sz, tc, &format!("Theme: {}  (T to toggle)", tm), 20.0, y);
        y += lh;
        text(buf, pw, ph, &self.font, sz, tc, &format!("Skip: {} words", cfg.skip_amount), 20.0, y);
        y += lh * 1.5;
        text(buf, pw, ph, &self.font, sz * 0.8, ac, "Press S to close settings", 20.0, y);
    }
}
