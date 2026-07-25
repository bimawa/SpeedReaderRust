# Tasks: SpeedReader для Zed

## Implementation Notes
- Rendering: `ab_glyph` for font rasterization, `pixels` (wgpu) for window framebuffer
- Window: `winit` for OS window, transparent + always-on-top + frameless
- Config: `~/.config/speed-reader/config.json`, editor_cmd defaults to "zed"
- Published on crates.io: `speed-reader` 0.2.0, `speed-reader-core` 0.2.0
- Zed extension (zed-ext/) deprecated — not needed, use tasks instead

---

## 1. Core Library (speed-reader-core) — ✅ ALL DONE

### 1.1 Core Tokenizer с ORP `[x]`
- Tokenizer split by unicode word boundaries, ORP = `min((len+1)/2, 4)-1`

### 1.2 Core TimingEngine WPM `[x]`
- Base: 60000/wpm * word_len/5, clamped [0.5, 3.0], sentence-end 1.5x pause

### 1.3 Core ReadingState machine `[x]`
- States: Idle → Playing → Paused → Finished. SpeedUp/Down + SkipFwd/Back work in all active states

### 1.4 Core PositionTracker `[x]`
- Maps byte_offset → { line, column, context } via binary search on line offsets

### 1.5 Core ConfigModel `[x]`
- Serde JSON, editor_cmd field, WPM/theme/font_size/skip/speed_step

## 2. GUI Application — ✅ ALL DONE

### 2.1 GUI scaffold `[x]`
- winit window, transparent, always-on-top, 600x200, centered, frameless

### 2.2 GUI RSVPRenderer `[x]`
- ab_glyph text rendering, ORP letter at center X, RoundedRect background

### 2.3 GUI InputHandler `[x]`
- Space=pause+editor jump, Esc/Q=exit, arrows=skip, Up/Down=speed, R=restart

### 2.4 GUI Config persistence `[x]`
- ~/.config/speed-reader/config.json, serde_json, CLI overrides

### 2.5 GUI Overlay controller `[x]`
- Instant-based timer, keyboard routing, drag window, settings panel (S key)

## 3. Zed Extension — ❌ DEPRECATED

### 3.1 Zed ext scaffold `[-]`
### 3.2 Zed ext slash command `[-]`
- Zed extension API не позволяет UI-оверлеи или манипуляцию курсором
- Вместо этого: Zed Tasks + глобальная таска в ~/.config/zed/tasks.json
