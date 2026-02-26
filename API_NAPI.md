# Rust <-> Electron Contract (`napi-rs`)

## 1. Module
N-API module name: `linxust_native`.

## 2. Types

### 2.1 Request Types
```ts
export interface CompressRequest {
  inputPaths: string[];
  outputPath: string;
  password?: string;
  chunkSizeBytes?: number;
}

export interface ExtractRequest {
  archivePath: string;
  outputDir: string;
  password?: string;
}
```

### 2.2 Event/Progress Types
```ts
export interface ProgressEvent {
  taskId: string;
  phase: 'scan' | 'compress' | 'encrypt' | 'write' | 'extract' | 'done';
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
```

## 3. Exported Functions

### 3.1 Sanity / Bridge
- `hello_from_rust(name: string): string`
  - Returns greeting text.
  - Traceability: REQ-001, REQ-002, REQ-003.

### 3.2 Archive Operations
- `start_compress(req: CompressRequest): Promise<TaskHandle>`
- `start_extract(req: ExtractRequest): Promise<TaskHandle>`
- `cancel_task(taskId: string): Promise<boolean>`
- `query_task(taskId: string): Promise<ProgressEvent>`

## 4. Error Model
All thrown JS errors map to `NativeError` payload fields serialized in message JSON when applicable.

## 5. Data Transfer Rules
- Binary payloads use `Buffer`/typed arrays.
- Prefer zero-copy transfer where supported by `napi-rs`.
- Paths are UTF-8 strings and validated in Rust.

## 6. Threading Model
- CPU-heavy compression runs in Rust worker threads (`rayon`).
- Async file orchestration runs in `tokio` runtime.
- N-API boundary remains non-blocking for Electron renderer responsiveness.
