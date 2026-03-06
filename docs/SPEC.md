# LinXust Specification (SPEC)

## 1. Scope
LinXust is a Linux-first archive application that creates, opens, tests, and extracts `.lxt` archives using an Electron GUI and a pure Rust processing engine.

### 1.1 Product Direction
- LinXust is intended to be a modern, Linux-first WinRAR-like archive manager built around the `.lxt` format rather than a generic file compressor.
- The current MVP scope is `.lxt` creation, archive browsing, archive integrity testing, and extraction. Support for third-party archive formats may be added later, but is out of scope unless explicitly specified.
- Final UI implementations may be inspired by the mockups under `assets/design/`, but the shipped interface must be an original design with a dark charcoal foundation and royal blue accents.

## 2. User Stories
- **US-001**: As a user, I can compress selected files/folders into a `.lxt` archive from the GUI.
- **US-002**: As a user, I can extract `.lxt` archives from the GUI.
- **US-003**: As a user, I can monitor progress and errors during archive operations.
- **US-004**: As a user, I can optionally encrypt archives with a password.
- **US-005**: As a KDE user, I can start compression from Dolphin context menu.
- **US-006**: As a user, I can inspect an existing `.lxt` archive and browse its file tree before extracting it.
- **US-007**: As a user, I can run an archive integrity test without extracting the archive contents.
- **US-008**: As a user, I can choose how extraction handles file conflicts and selectively extract only part of an archive.
- **US-009**: As a Linux user, I can open a `.lxt` archive directly from my desktop environment or file manager.
- **US-010**: As a user, I can manage archives from a persistent dashboard-style workspace instead of relying only on transient dialogs.

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

### 3.9 Archive Semantics And Archive Manager Workflows
- **REQ-023**: The native layer MUST persist an entry manifest for `.lxt` archives that describes relative paths, entry kind (`file`, `directory`, `symlink`), size, timestamps, Unix mode bits, and chunk ownership.
- **REQ-024**: The application MUST support archive inspection and integrity test workflows for existing `.lxt` archives without requiring full extraction.
- **REQ-025**: Extraction MUST support entry subset selection and explicit overwrite policies (`skip`, `replace`, `rename`).
- **REQ-026**: The UI MUST provide a primary archive manager/dashboard surface for recent archives, archive actions, and task progress, plus a dedicated create-archive flow.
- **REQ-027**: The project MUST preserve Linux-relevant filesystem semantics where possible, including executable bits, empty directories, and symlink targets.

### 3.10 Linux Experience And Product Design
- **REQ-028**: Linux desktop integration MUST include `.lxt` MIME registration and file association so archives can be opened directly in LinXust.
- **REQ-029**: UI MUST support drag-and-drop from Linux file managers into both compression and extraction flows.
- **REQ-030**: The visual design system MUST default to a dark theme with charcoal base colors, royal blue accents, high contrast, and original modern layouts rather than generic admin styling.
- **REQ-031**: The application MUST expose user-facing defaults for compression preset, output destination, and overwrite behavior.
- **REQ-032**: The application MUST surface recent archives and current or recent operations in the dashboard so archive work does not disappear behind modal flows.

## 4. Non-Functional Requirements
- **REQ-020**: Core archive operations SHOULD be multi-threaded where safe.
- **REQ-021**: APIs MUST validate inputs and reject path traversal patterns in archive entries.
- **REQ-022**: `.lxt` format MUST include enough metadata for compatibility checks and future versioning.

## 5. Traceability Notes
Implementation and tests SHOULD reference requirement IDs in commit messages, tests, or task annotations.
