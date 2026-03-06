# LinXust Architecture

## Overview
LinXust is a Linux-first desktop archiver built around three layers:

- `frontend/`: React renderer for the archive manager dashboard, create-archive flow, archive inspection, and progress reporting.
- `electron/`: main/preload bridge that hosts the renderer, opens files from the desktop shell, and exposes a narrow IPC API.
- `native/`: Rust `napi-rs` module that owns archive processing, codecs, `.lxt` metadata, inspection, and format logic.

## Execution Flow
1. The renderer calls `window.linxustApi` methods exposed by `electron/preload.cjs`.
2. Electron routes IPC requests and desktop open-file events to the main process.
3. The main process loads the compiled N-API addon from `native/` and delegates archive work to Rust.
4. Rust returns archive summaries, task handles, results, or structured errors back across the N-API boundary.

## Product Surfaces
- The primary desktop surface is an archive manager dashboard that lists recent archives, archive actions, and current or recent tasks.
- Archive creation is a dedicated flow, likely modal or workspace-driven, rather than a single inline form.
- Archive inspection is a first-class workflow and must not depend on full extraction.

## Current Implementation Status
- The React and Electron shells exist and can display bridge diagnostics.
- The renderer TypeScript package uses BiomeJS as its lint/format toolchain and Evlog for structured frontend logging.
- The Rust crate already contains the Phase 2 bitstream and LZ77 + Huffman codec implementation with tests.
- The native addon loader now uses a shared Electron bridge loader that prefers the compiled `.node` artifact produced by `napi build --platform`; Electron falls back to a diagnostic shim only when the addon has not been built yet.
- `.lxt` container serialization and streaming I/O exist, but metadata-driven archive inspection, task orchestration, and end-user archive workflows remain to be completed.

## Ownership Boundaries
- Renderer code should stay presentation-focused and own dashboard state, archive browsing UX, settings UX, and task presentation.
- Electron code should stay limited to window creation, shell integration, environment selection, dialogs, and IPC.
- Rust should own compression, decompression, archive serialization, metadata parsing, encryption, filesystem safety, and long-running task orchestration.

## Design Direction
- Use the mockups in `assets/design/` as reference material, not as a source for one-to-one reproduction.
- The shipped UI should default to a dark theme with charcoal base tones and royal blue accents.
- The visual language should feel modern and original rather than like a generic admin panel.

## Development Notes
- During UI development, run the renderer dev server separately and start Electron with `LINXUST_DEV_URL=http://localhost:5173`.
- Run `bun run verify:native-bridge` after `bun run build:native` to validate the compiled addon through the same loader used by the Electron main process.
- The renderer can optionally forward Evlog batches to a remote ingest endpoint when `VITE_EVLOG_ENDPOINT` is set.
- For packaged or local production-style runs, Electron should load `frontend/dist/index.html`.
- Keep new milestones reflected in [docs/TASKS.md](TASKS.md) and detailed behavior changes reflected in the spec docs.
