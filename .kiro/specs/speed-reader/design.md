# Technical Design: SpeedReader

## Overview

Десктопное RSVP-приложение. Отображает текст слово за словом с ORP-подсветкой. Независимо от редактора — поддерживает любой редактор через `editor_cmd` (zed, code, nvim, hx...).

## Architecture

```
core/src/            Pure Rust RSVP engine
  tokenizer.rs       Токенизация текста + ORP-позиционирование
  timing.rs          WPM-тайминг с word-length scaling
  state.rs           Конечный автомат: Idle/Playing/Paused/Finished
  position.rs        Маппинг byte_offset → line:column
  config.rs          ConfigModel (serde)

gui/src/             Desktop overlay (winit + ab_glyph + pixels)
  main.rs            CLI entry (clap), ConfigPersistence::load
  overlay.rs         winit window, event loop, timer, drag
  renderer.rs        ab_glyph text rendering, ORP at center
  input.rs           Keyboard handling + zed CLI jump
  config.rs          ~/.config/speed-reader/config.json
```

## Decisions

| Decision | Rationale |
|----------|-----------|
| ab_glyph вместо skia-safe | Pure Rust, нет native deps |
| pixels (wgpu) для вывода | GPU-ускорение через Metal |
| editor_cmd в конфиге | Поддержка любого редактора |
| Zed Tasks вместо extension | Extension API не даёт UI/курсор |
| `[patch.crates-io]` | Локальная разработка + crates.io публикация |

## Controls

| Key | Action |
|-----|--------|
| Space | Pause + jump via `editor_cmd file:line:col` |
| Space (again) | Resume |
| Esc / Q | Close |
| ← → | Skip N words |
| ↑ ↓ | Speed ±10 WPM |
| S | Settings (WPM, theme) |
| R | Restart |
| Drag | Move window |

## Published Crates

- `speed-reader-core` 0.2.0 — библиотека RSVP-движка
- `speed-reader` 0.2.0 — бинарник оверлея
