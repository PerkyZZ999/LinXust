# LinXust Implementation Tasks

## Phase 0: Canonical Artifacts
- [x] Maintain the canonical spec set in `docs/` (`SPEC.md`, `FORMAT_LXT.md`, `API_NAPI.md`, `TASKS.md`) (`REQ-004`, `REQ-005`)

## Phase 1: Foundation And Native Bridge
- [x] Scaffold the Electron main/preload shell and React + TypeScript renderer (`REQ-001`)
- [x] Scaffold the Rust `napi-rs` crate and root package wiring (`REQ-001`, `REQ-002`, `REQ-003`)
- [x] Verify the compiled N-API addon loads through the shared Electron bridge loader for `hello_from_rust` without relying on a checked-in shim (`REQ-001`, `REQ-002`, `REQ-003`)
- [x] Add a documented, repeatable local dev flow for renderer + Electron + native addon builds (`REQ-004`, `REQ-005`)
- [x] Adopt BiomeJS for renderer formatting/linting and Evlog for structured frontend diagnostics (`REQ-018`, `REQ-019`)

## Phase 2: Compression Codec
- [x] Implement bitstream primitives (`BitWriter`, `BitReader`) (`REQ-006`, `REQ-008`)
- [x] Implement LZ77 encode/decode with tests (`REQ-006`, `REQ-008`)
- [x] Implement Huffman encode/decode with tests (`REQ-006`, `REQ-008`)
- [x] Implement the Phase 2 LZ77 + Huffman pipeline with roundtrip coverage (`REQ-006`, `REQ-008`)

## Phase 3: `.lxt` Container And Streaming I/O
- [x] Implement `.lxt` header, chunk table, and footer serialization (`REQ-022`)
- [x] Implement `.lxt` reader validation for offsets, versions, and CRCs (`REQ-021`, `REQ-022`)
- [x] Integrate Phase 2 codec blocks into the `.lxt` writer/reader path (`REQ-006`, `REQ-008`, `REQ-022`)
- [x] Add streaming file I/O with bounded buffers (`REQ-012`, `REQ-013`)
- [ ] Integrate AES-256-GCM block encryption and Argon2 key derivation into the `.lxt` pipeline (`REQ-009`, `REQ-010`, `REQ-022`)
- [ ] Emit explicit authentication and wrong-password errors from native archive operations (`REQ-011`, `REQ-019`)
- [ ] Implement metadata section and entry manifest serialization for files, directories, and symlinks (`REQ-023`, `REQ-027`)
- [ ] Enforce metadata path normalization, duplicate-path rejection, and traversal-safe extraction rules (`REQ-021`, `REQ-023`, `REQ-027`)
- [ ] Implement selective extraction and overwrite-policy behavior in native extraction paths (`REQ-025`, `REQ-031`)
- [ ] Implement archive inspection and integrity-test APIs over metadata without full extraction (`REQ-024`, `REQ-025`)
- [ ] Add async orchestration for long-running archive operations (`REQ-015`, `REQ-019`)
- [ ] Add parallel block processing with `rayon` where safe (`REQ-014`, `REQ-020`)

## Phase 4: Native API Surface And IPC Wiring
- [ ] Implement stable native exports for compress, extract, inspect, test, cancel, and query operations (`REQ-003`, `REQ-019`, `REQ-024`, `REQ-025`)
- [ ] Implement TypeScript request/result models in the renderer and preload layers that mirror `API_NAPI.md` (`REQ-003`, `REQ-019`)
- [ ] Expose the full `window.linxustApi` surface from `electron/preload.cjs` with typed renderer declarations (`REQ-001`, `REQ-003`)
- [ ] Add Electron main-process IPC handlers for every archive action and task-control action (`REQ-001`, `REQ-003`, `REQ-019`)
- [ ] Wire main-process handlers to the Rust addon and map native failures into structured UI-safe errors (`REQ-003`, `REQ-019`)
- [ ] Add dialog/file-picker helpers in Electron for create, open, extract destination, and overwrite confirmation flows (`REQ-018`, `REQ-025`, `REQ-026`)

## Phase 5: Renderer Foundations And Component System
- [ ] Establish renderer design tokens, theme variables, typography, spacing, and motion primitives for the dark charcoal + royal blue visual system (`REQ-026`, `REQ-030`)
- [ ] Build the top-level application shell, window layout, header, navigation, and responsive content regions (`REQ-026`, `REQ-030`)
- [ ] Create reusable UI primitives for buttons, inputs, selects, checkboxes, segmented controls, badges, cards, and panels (`REQ-018`, `REQ-030`)
- [ ] Create reusable feedback primitives for dialogs, toasts, banners, loading states, empty states, and inline error states (`REQ-019`, `REQ-026`, `REQ-030`)
- [ ] Build archive-centric display components for archive rows, archive cards, entry tree/list views, metadata panels, and progress cards (`REQ-018`, `REQ-024`, `REQ-026`)
- [ ] Define icon usage for archive types, actions, and developer-oriented file states using project assets where appropriate (`REQ-026`, `REQ-030`)

## Phase 6: Frontend State, Data Flow, And Backend Wiring
- [ ] Add a renderer-side service layer that wraps `window.linxustApi` and normalizes archive/task requests and responses (`REQ-003`, `REQ-019`)
- [ ] Add application state management for current archive context, recent archives, task progress, selections, dialogs, and user settings (`REQ-026`, `REQ-031`, `REQ-032`)
- [ ] Implement task polling, cancellation, and optimistic UI updates for long-running archive operations (`REQ-019`, `REQ-032`)
- [ ] Persist recent archives, task history, and user defaults for the dashboard experience (`REQ-031`, `REQ-032`)
- [ ] Wire route/state initialization for app launch, archive-open events, and empty-dashboard startup (`REQ-026`, `REQ-028`, `REQ-032`)

## Phase 7: User Workflows And Screens
- [ ] Implement archive manager dashboard shell for recent archives, actions, and task history (`REQ-026`, `REQ-032`)
- [ ] Implement create-archive modal or workspace flow aligned with the design references in `assets/design/` (`REQ-026`, `REQ-030`)
- [ ] Implement compress workflow inputs, validation, and submit flow (`REQ-018`, `REQ-019`, `REQ-031`)
- [ ] Implement archive inspection screen with entry tree, metadata summary, and archive test action (`REQ-024`, `REQ-026`)
- [ ] Implement extract workflow inputs, selective extraction UI, and overwrite-policy UX (`REQ-018`, `REQ-019`, `REQ-025`, `REQ-031`)
- [ ] Implement password-based encryption UX, reveal/hide controls, and wrong-password error display (`REQ-009`, `REQ-010`, `REQ-011`, `REQ-019`)
- [ ] Add drag-and-drop archive targets and file selection UX for compression and extraction flows (`REQ-018`, `REQ-029`)
- [ ] Add progress dashboard, task polling, and cancellation surface (`REQ-018`, `REQ-019`, `REQ-032`)
- [ ] Add settings for default compression preset, output destination, and overwrite behavior (`REQ-031`)

## Phase 8: Integration, Testing, And Quality
- [ ] Add Rust unit and integration coverage for encryption, metadata manifests, inspection, overwrite policy, and traversal safety (`REQ-009`, `REQ-010`, `REQ-011`, `REQ-021`, `REQ-023`, `REQ-025`)
- [ ] Add Electron/preload integration coverage for IPC request routing and error translation (`REQ-003`, `REQ-019`)
- [ ] Add renderer component tests for core archive manager UI states and forms (`REQ-018`, `REQ-026`, `REQ-030`)
- [ ] Add end-to-end desktop smoke tests for create, inspect, test, and extract flows (`REQ-018`, `REQ-024`, `REQ-025`, `REQ-026`)
- [ ] Validate accessibility, keyboard navigation, and focus handling across dashboard, dialogs, and forms (`REQ-018`, `REQ-030`)
- [ ] Perform performance and responsiveness pass for large archives, long task lists, and large entry trees (`REQ-014`, `REQ-015`, `REQ-020`, `REQ-032`)

## Phase 9: Linux Integration And Packaging
- [ ] Route desktop open-file and file-manager launch events into archive inspection/extract flows (`REQ-026`, `REQ-028`)
- [ ] Add `.lxt` MIME type, file association, and open-with integration (`REQ-016`, `REQ-028`)
- [ ] Add `.desktop` launcher and icon assets (`REQ-016`)
- [ ] Add Dolphin/KDE service menu integration (`REQ-016`)
- [ ] Add Arch Linux packaging path (`PKGBUILD`) (`REQ-017`)

## Phase 10: Release Readiness
- [ ] Audit the end-to-end implementation against `SPEC.md`, `FORMAT_LXT.md`, `API_NAPI.md`, and `TASKS.md` for drift before first usable release (`REQ-004`, `REQ-005`)
- [ ] Document user-facing archive workflows, Linux integration steps, and known limitations in the README and product docs (`REQ-004`, `REQ-005`)
- [ ] Verify production build, packaged app startup, native loading, and critical archive workflows on Linux (`REQ-001`, `REQ-003`, `REQ-016`, `REQ-028`)
