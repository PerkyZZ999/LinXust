# LinXust Specification (SPEC)

## 1. Scope
LinXust is a Linux-first archive application that creates and extracts `.lxt` archives using an Electron GUI and a pure Rust processing engine.

## 2. User Stories
- **US-001**: As a user, I can compress selected files/folders into a `.lxt` archive from the GUI.
- **US-002**: As a user, I can extract `.lxt` archives from the GUI.
- **US-003**: As a user, I can monitor progress and errors during archive operations.
- **US-004**: As a user, I can optionally encrypt archives with a password.
- **US-005**: As a KDE user, I can start compression from Dolphin context menu.

## 3. Functional Requirements

### 3.1 Architecture
- **REQ-001**: The frontend MUST be Electron + React + TypeScript + Vite.
- **REQ-002**: All archive I/O, binary stream processing, compression, and encryption MUST run in Rust.
- **REQ-003**: JavaScript/Rust communication MUST use `napi-rs` and expose a stable API documented in `API_NAPI.md`.

### 3.2 Spec-Driven Workflow
- **REQ-004**: Every implementation task MUST trace to one or more REQ IDs in this document.
- **REQ-005**: The project MUST maintain `SPEC.md`, `FORMAT_LXT.md`, `API_NAPI.md`, and `TASKS.md` as canonical artifacts.

### 3.3 Compression Engine
- **REQ-006**: Compression MUST be implemented manually in Rust using LZ77 + Huffman coding.
- **REQ-007**: The project MUST NOT depend on `flate2`, `zstd`, `tar`, or `zip` for compression/archiving.
- **REQ-008**: Compression and decompression MUST be symmetric and deterministic for valid inputs.

### 3.4 Encryption
- **REQ-009**: Archive encryption MUST use AES-256-GCM from RustCrypto.
- **REQ-010**: Password key derivation MUST use Argon2 with per-archive random salt.
- **REQ-011**: Authentication failures MUST produce explicit recoverable errors.

### 3.5 Streaming and Memory Safety
- **REQ-012**: File operations MUST be streaming and chunk-based; full file loading is forbidden.
- **REQ-013**: Rust processing MUST use bounded buffers with `BufReader`/`BufWriter` style semantics.

### 3.6 Concurrency and Performance
- **REQ-014**: Parallel block compression MUST use `rayon`.
- **REQ-015**: Async file coordination and long-running task orchestration MUST use `tokio`.

### 3.7 Linux Integration
- **REQ-016**: The project MUST include Linux desktop integration artifacts (`.desktop`, KDE service menu).
- **REQ-017**: The project MUST support an Arch Linux packaging path (`PKGBUILD`).

### 3.8 UX and Observability
- **REQ-018**: UI MUST provide drag-and-drop target, file list/tree, and operation progress.
- **REQ-019**: API responses MUST include progress and structured error messages suitable for display.

## 4. Non-Functional Requirements
- **REQ-020**: Core archive operations SHOULD be multi-threaded where safe.
- **REQ-021**: APIs MUST validate inputs and reject path traversal patterns in archive entries.
- **REQ-022**: `.lxt` format MUST include enough metadata for compatibility checks and future versioning.

## 5. Traceability Notes
Implementation and tests SHOULD reference requirement IDs in commit messages, tests, or task annotations.
