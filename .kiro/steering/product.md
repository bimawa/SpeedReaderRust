# Product: SpeedReader Zed Extension

## Purpose
A Zed editor extension that overlays a SpeedReader popup for rapid reading of text buffers — word-by-word with highlighted center letter (RSVP — Rapid Serial Visual Presentation).

## Value
- Accelerate reading of large documents (specs, RFCs, code comments) directly inside Zed without context-switching
- Preserve cursor context: on pause/cancel, the editor focuses on the exact reading position so the user can edit and resume from the same spot
- Configurable WPM speed and color scheme

## Core Capabilities
- RSVP word-by-word playback with center-letter highlighting
- Configurable speed (words per minute) and color theme
- Pause/resume with editor cursor synchronization
- On cancel: focus editor at reading position
- Popup overlay within Zed's existing UI surface

## Non-Goals (current scope)
- No standalone window — operates inside Zed editor
- No document import or file management — reads from open buffer
- No text-to-speech
- No cloud sync or sharing
