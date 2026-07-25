# SpeedReaderRust

RSVP (Rapid Serial Visual Presentation) reading overlay. Displays text word by word with highlighted center letter (ORP). Press Space to pause and jump to the exact reading position in the editor.

## Install

```sh
cargo install speed-reader
```

Or from source:
```sh
git clone https://github.com/bimawa/SpeedReaderRust
cd SpeedReaderRust
cargo install --path gui
```

## Usage in Zed

### 1. Add task (global)

Add to `~/.config/zed/tasks.json`:

```json
[
  {
    "label": "SpeedReader: Read current file",
    "command": "speed-reader",
    "args": ["$ZED_FILE"],
    "tags": ["speed-reader"],
    "hide": "on_success"
  }
]
```

### 2. Run

`Cmd+Shift+P` → `SpeedReader: Read current file`

### 3. Keybinding (optional)

Add to `~/.config/zed/keybindings.json`:

```json
{
  "context": "Editor",
  "bindings": {
    "cmd-shift-r": ["task::Spawn", { "task_name": "SpeedReader: Read current file" }]
  }
}
```

Then `Cmd+Shift+R` from any buffer.

## Configure editor (not just Zed)

Edit `~/.config/speed-reader/config.json`:

```json
{
  "editor_cmd": "code",
  "wpm": 300
}
```

Supports any editor with `editor file:line:col` CLI syntax (Zed, VS Code, Vim, Helix, etc.).

## Controls

| Key | Action |
|-----|--------|
| `Space` | Pause + jump to editor |
| `Space` (again) | Resume |
| `Esc` | Close overlay |
| `←` `→` | Skip words |
| `↑` `↓` | Speed ±10 WPM |
| `S` | Settings panel |
| Mouse drag | Move overlay |

## Project structure

```
core/     RSVP engine (Rust library)
gui/      Desktop overlay (winit + ab_glyph + pixels)
```
