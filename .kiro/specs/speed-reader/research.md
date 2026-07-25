# Research Log: SpeedReader

## Summary

Проект реализован. RSVP-оверлей на Rust (winit + ab_glyph + pixels). 
Zed Extension API оказался бесполезен — нет UI/курсора. Решение: Tasks + `editor_cmd`.

## Key Findings

1. **Zed Extension API** не даёт UI, курсора или буфера — только LSP/themes/slash-команды
2. **RSVP core** реализован на Rust, опубликован на crates.io
3. **GUI** через winit + wgpu (pixels) + ab_glyph
4. **Configurable editor** через `~/.config/speed-reader/config.json` → `editor_cmd`

## Final Decisions

| Decision | Selected | Why |
|----------|----------|-----|
| Rendering | ab_glyph + pixels | Pure Rust, GPU via Metal |
| Editor integration | Task + editor_cmd | Extension API недостаточен |
| Window | winit | Кроссплатформенность |
| Config | JSON в ~/.config | Простота, человекочитаемость |
