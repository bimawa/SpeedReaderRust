# Technology: SpeedReader Zed Extension

## Core Language
- **Rust** — primary implementation language. Zed extensions use Rust for extension logic.
- **WebAssembly** — WASM compilation target for cross-platform compatibility within Zed's extension runtime.
- **TypeScript/JavaScript** — if needed for Zed extension manifest and UI interop (Zed's extension API surface).

## Zed Extension Platform
- Zed extensions run in a WASM-based sandbox
- Extension API provides: buffer access, editor hooks, UI panel creation, command registration
- Extensions are packaged as `.wasm` + metadata + optional assets

## UI Rendering Strategy
- Zed's native UI system (gpui) for extension surfaces — Rust-native, GPU-accelerated
- Alternative: custom HTML/CSS via webview if Zed's extension API supports it
- Design must work within Zed's panel/overlay system

## Build System
- `cargo build --target wasm32-wasi` for WASM extension target
- Zed extension development workflow: `zed: install extension from path`

## Dependencies
- Rust WASM toolchain (`wasm32-wasip1` target)
- Zed extension SDK (`https://github.com/zed-industries/extensions`)
- Minimal external crate dependencies preferred

## Coding Standards
- Strong type safety — no unsafe casts
- Prefer Rust enums for state modeling
- Structured error handling with custom error types
- Async where I/O or editor coordination is required
