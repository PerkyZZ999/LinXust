# LinXust

Spec-driven initialization for LinXust.

## Phase 0 Artifacts
- `SPEC.md`
- `FORMAT_LXT.md`
- `API_NAPI.md`
- `TASKS.md`

## Phase 1 Scaffold
- `frontend/`: React + TypeScript + Vite UI
- `native/`: Rust 2024 `napi-rs` backend skeleton
- `electron/`: Electron main/preload bridge skeleton

## Quick validation
```bash
npm --prefix frontend install
npm --prefix frontend run build
cargo test --manifest-path native/Cargo.toml
```
