# SpeedReaderRust

RSVP (Rapid Serial Visual Presentation) reading overlay for Zed editor. Displays text word by word with highlighted center letter (ORP). Press Space to pause and jump to the exact reading position in the editor.

## Install

```sh
cargo build --release -p speed-reader
cp target/release/speed-reader /usr/local/bin/
```

## Usage in Zed

Open any file → `Cmd+Shift+P` → `SpeedReader: Read current file`.

Or set a keybinding in `~/.config/zed/keybindings.json`:

```json
{
  "context": "Editor",
  "bindings": {
    "cmd-shift-r": ["task::Spawn", { "task_name": "SpeedReader: Read current file" }]
  }
}
```

Then `Cmd+Shift+R` from any buffer.

## Controls

| Key | Action |
|-----|--------|
| `Space` | Pause + jump to position in Zed |
| `Space` (again) | Resume |
| `Esc` | Close overlay |
| `←` `→` | Skip words |
| `↑` `↓` | Speed ±10 WPM |
| `S` | Settings panel |
| Mouse drag | Move overlay window |

## Project structure

```
core/     RSVP engine (Rust library — tokenizer, ORP, timing, state machine, config)
gui/      Desktop overlay (winit + ab_glyph + pixels)

zed-ext/  Optional Zed WASM extension (not needed for basic usage)
```
