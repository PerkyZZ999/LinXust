# LinXust

LinXust is a Linux-first desktop archive manager built around a custom `.lxt` format, with an Electron renderer shell and a Rust processing engine.

## Product Direction
- LinXust aims to be a modern, Linux-first WinRAR-like experience centered on `.lxt` rather than a generic multi-format archiver.
- The current product scope is `.lxt` creation, inspection, integrity testing, and extraction.
- UI work should take inspiration from `assets/design/`, but the shipped application should be an original design with dark charcoal surfaces and royal blue accents.

## Repository Layout
- `docs/`: canonical specs, API contract, architecture notes, and implementation plan
- `electron/`: Electron main/preload bridge and window lifecycle
- `frontend/`: React + TypeScript renderer
- `native/`: Rust `napi-rs` addon and archive engine

## Current Status
- Specification and format documents are in place.
- Phase 2 bitstream and LZ77 + Huffman codec work is implemented in Rust and covered by unit tests.
- The Electron shell and renderer scaffold exist, and the bridge now reports whether it is using the compiled native addon or a fallback diagnostic path.
- The frontend TypeScript toolchain now uses BiomeJS for linting and formatting, and Evlog provides structured renderer-side observability.
- `.lxt` container serialization and streaming I/O exist, while metadata-driven archive inspection, archive manager workflows, and Linux shell integration are still in progress.

## Canonical Documents
- [docs/SPEC.md](docs/SPEC.md)
- [docs/FORMAT_LXT.md](docs/FORMAT_LXT.md)
- [docs/API_NAPI.md](docs/API_NAPI.md)
- [docs/TASKS.md](docs/TASKS.md)
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)

## Development

### Prerequisites
- Bun
- Rust toolchain with Cargo

### Install Dependencies
```bash
bun install
cd frontend && bun install
cd ../native && bun install
```

### Common Commands
```bash
bun run build:frontend
bun run build:native
 bun run verify:native-bridge
bun run lint:frontend
bun run format:frontend
cargo test --manifest-path native/Cargo.toml
```

### Electron Development
Run the renderer dev server in one terminal:

```bash
cd frontend && bun run dev
```

Start Electron in another terminal:

```bash
LINXUST_DEV_URL=http://localhost:5173 bun run electron
```

For production-style local runs, build the renderer and native addon first, then launch Electron without `LINXUST_DEV_URL`.

Use `bun run verify:native-bridge` after `bun run build:native` to confirm the compiled addon loads through the same bridge loader Electron uses in the main process.

Set `VITE_EVLOG_ENDPOINT` before starting the renderer if you want Evlog browser drains to ship batched frontend logs to an ingest endpoint; otherwise Evlog stays local to the renderer console.
