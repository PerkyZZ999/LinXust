# Rust <-> Electron Contract (`napi-rs`)

## 1. Module
N-API module name: `linxust_native`.

## 2. Types

### 2.1 Request Types
```ts
export type CompressionPreset = 'fast' | 'normal' | 'ultra';
export type OverwritePolicy = 'skip' | 'replace' | 'rename';

export interface CompressRequest {
  inputPaths: string[];
  outputPath: string;
  password?: string;
  compressionPreset?: CompressionPreset;
  chunkSizeBytes?: number;
}

export interface ExtractRequest {
  archivePath: string;
  outputDir: string;
  password?: string;
  selectedEntries?: string[];
  overwritePolicy?: OverwritePolicy;
}

export interface InspectRequest {
  archivePath: string;
  password?: string;
}
```

### 2.2 Event/Progress Types
```ts
export interface ProgressEvent {
  taskId: string;
  phase:
    | 'scan'
    | 'inspect'
    | 'verify'
    | 'compress'
    | 'encrypt'
    | 'write'
    | 'extract'
    | 'done';
  percent: number;
  processedBytes: number;
  totalBytes?: number;
  message?: string;
}
```

### 2.3 Result Types
```ts
export interface TaskHandle {
  taskId: string;
}

export interface NativeError {
  code: string;
  message: string;
  retriable: boolean;
}

export interface ArchiveEntry {
  path: string;
  kind: 'file' | 'directory' | 'symlink';
  originalSize: number;
  compressedSize?: number;
  modifiedAtUnixSeconds?: number;
  unixMode?: number;
  encrypted: boolean;
  chunkCount: number;
  linkTarget?: string;
}

export interface ArchiveSummary {
  archivePath: string;
  format: 'lxt';
  entryCount: number;
  originalSize: number;
  compressedSize: number;
  encrypted: boolean;
  comment?: string;
  entries: ArchiveEntry[];
}
```

## 3. Exported Functions

### 3.1 Sanity / Bridge
- `hello_from_rust(name: string): string`
  - Returns greeting text.
  - Traceability: REQ-001, REQ-002, REQ-003.
  - The JavaScript entrypoint in `native/` MUST resolve the compiled `.node` artifact produced by `napi build --platform`; a static shim is acceptable only as a temporary diagnostic fallback outside the canonical bridge path.
  - Use `bun run verify:native-bridge` to validate compiled binding resolution through the shared Electron bridge loader.

### 3.2 Archive Operations
- `start_compress(req: CompressRequest): Promise<TaskHandle>`
- `start_extract(req: ExtractRequest): Promise<TaskHandle>`
- `inspect_archive(req: InspectRequest): Promise<ArchiveSummary>`
- `test_archive(req: InspectRequest): Promise<TaskHandle>`
- `cancel_task(taskId: string): Promise<boolean>`
- `query_task(taskId: string): Promise<ProgressEvent>`

## 4. Error Model
All thrown JS errors map to `NativeError` payload fields serialized in message JSON when applicable.

## 5. Data Transfer Rules
- Binary payloads use `Buffer`/typed arrays.
- Prefer zero-copy transfer where supported by `napi-rs`.
- Paths are UTF-8 strings and validated in Rust.
- Archive entry paths returned to the renderer are normalized relative paths using `/` separators.

## 6. Threading Model
- CPU-heavy compression runs in Rust worker threads (`rayon`).
- Async file orchestration runs in `tokio` runtime.
- N-API boundary remains non-blocking for Electron renderer responsiveness.
