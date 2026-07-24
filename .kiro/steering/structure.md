# Structure: SpeedReader Zed Extension

## Repository Organization

```
SpeedReaderZedExt/
├── .kiro/                    # Project knowledge (steering, specs)
├── Cargo.toml                # Rust workspace root
├── extension.toml            # Zed extension manifest
├── src/
│   ├── lib.rs                # Extension entry point — register commands, panels
│   ├── speed_reader.rs       # Core RSVP engine — word timing, position tracking
│   └── ui/
│       └── overlay.rs        # Popup overlay UI — rendering, input handling
```

## Naming Conventions
- Source files: `snake_case.rs`
- Types, traits: `PascalCase`
- Functions, variables: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`

## Import Strategy
- `use crate::...` for internal modules
- Group: std → external crates → crate internals

## Module Boundaries
- `src/speed_reader.rs` — pure RSVP logic, no GUI
- `src/ui/overlay.rs` — all rendering and user interaction
- `src/lib.rs` — glue: extension lifecycle, command routing
