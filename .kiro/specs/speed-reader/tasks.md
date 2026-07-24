# Tasks: SpeedReader для Zed

## Implementation Notes
- Use `cargo test` for all verification
- Core library (`speed-reader-core`) is pure Rust, no platform deps
- Unicode handling via `unicode-segmentation` crate
- All config serialization via `serde_json`

---

## 1. Core Library (speed-reader-core)

### 1.1 Core Tokenizer с ORP
- **Requirement**: 1.1, 1.2
- **Design**: Tokenizer section
- **Boundary**: `core/src/tokenizer.rs`
- **Depends**: —
- **Status**: `[x]`

### 1.2 Core TimingEngine WPM
- **Requirement**: 4.1
- **Design**: TimingEngine section
- **Boundary**: `core/src/timing.rs`
- **Depends**: 1.1
- **Status**: `[x]`

### 1.3 Core ReadingState machine
- **Requirement**: 3.1, 3.3
- **Design**: ReadingState section
- **Boundary**: `core/src/state.rs`
- **Depends**: 1.2
- **Status**: `[x]`

### 1.4 Core PositionTracker
- **Requirement**: 2.2
- **Design**: PositionTracker section
- **Boundary**: `core/src/position.rs`
- **Depends**: —
- **Status**: `[x]`

### 1.5 Core ConfigModel
- **Requirement**: 4.1, 4.2, 4.3
- **Design**: ConfigModel section
- **Boundary**: `core/src/config.rs`
- **Depends**: —
- **Status**: `[x]`

## 2. GUI Application (speed-reader-gui)

### 2.1 GUI scaffold
- **Requirement**: 2.1
- **Design**: OverlayWindow section
- **Boundary**: `gui/`
- **Depends**: 1.1-1.5
- **Status**: `[x]`

### 2.2 GUI RSVPRenderer
- **Requirement**: 1.1
- **Design**: RSVPRenderer section
- **Boundary**: `gui/src/renderer.rs`
- **Depends**: 2.1
- **Status**: `[x]`

### 2.3 GUI InputHandler
- **Requirement**: 3.1, 3.2, 3.3, 3.2
- **Design**: InputHandler section
- **Boundary**: `gui/src/input.rs`
- **Depends**: 2.1
- **Status**: `[x]`

### 2.4 GUI Config persistence
- **Requirement**: 4.1, 4.2, 4.3
- **Design**: ConfigPersistence section
- **Boundary**: `gui/src/config.rs`
- **Depends**: 2.1
- **Status**: `[x]`

### 2.5 GUI Overlay controller + integration
- **Requirement**: 2.1, 2.2, 3.2
- **Design**: OverlayWindow, IPC Protocol sections
- **Boundary**: `gui/src/overlay.rs`, `gui/src/ipc.rs`
- **Depends**: 2.2, 2.3, 2.4
- **Status**: `[x]`

## 3. Zed Extension (speed-reader-zed)

### 3.1 Zed ext scaffold
- **Requirement**: 5.2
- **Design**: Extension trait section
- **Boundary**: `zed-ext/`
- **Status**: `[x]`

### 3.2 Zed ext slash command
- **Requirement**: 1.2, 5.2
- **Design**: ExtSlashCommand section
- **Boundary**: `zed-ext/src/lib.rs`
- **Depends**: 3.1
- **Status**: `[x]`
