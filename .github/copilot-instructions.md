# LinXust Copilot Instructions

## Canonical Documents
- Treat [docs/SPEC.md](../docs/SPEC.md), [docs/FORMAT_LXT.md](../docs/FORMAT_LXT.md), [docs/API_NAPI.md](../docs/API_NAPI.md), and [docs/TASKS.md](../docs/TASKS.md) as the source of truth.
- Keep implementation changes and documentation in sync. If behavior, architecture, or milestones change, update the matching document in the same change.
- Prefer task updates that include requirement references from `docs/SPEC.md`.

## Repository Layout
- `frontend/`: renderer-only React + TypeScript UI. Do not add direct filesystem, Node.js, or archive logic here.
- `electron/`: Electron main/preload bridge, window lifecycle, IPC surface, and renderer bootstrapping.
- `native/`: Rust processing engine and N-API exports. Compression, decompression, archive format parsing, encryption, streaming I/O, and path validation belong here.
- `docs/`: project specifications, API contract, architecture notes, and execution plan.

## Implementation Rules
- Follow the spec-driven workflow in `docs/SPEC.md` and keep work traceable to requirement IDs.
- Preserve the Linux-first direction of the project while keeping the Electron shell portable where it is cheap to do so.
- Do not add `flate2`, `zstd`, `tar`, or `zip`; REQ-006 and REQ-007 require a manual Rust implementation for compression and archiving.
- Keep JavaScript bridge code thin. Business logic and binary processing should remain in Rust.
- Prefer BiomeJS over ESLint/Prettier for the frontend package, and route renderer observability through Evlog helpers instead of ad-hoc console calls.
- Prefer small, feature-oriented Rust modules over growing `native/src/lib.rs` into a single file.

## Build And Validation
- Use Bun for JavaScript package management and scripts.
- Use `cd frontend && bun run lint` for renderer linting.
- Use `cd frontend && bun run build` for renderer builds.
- Use `cd native && bun run build` to produce the N-API addon.
- Use `cargo test --manifest-path native/Cargo.toml` for Rust validation.

## Review Expectations
- Flag task list drift immediately if implementation status no longer matches `docs/TASKS.md`.
- Remove stale starter-template files when they no longer reflect the real project.
- Avoid broad reorganizations unless they improve traceability, build flow, or module boundaries.
