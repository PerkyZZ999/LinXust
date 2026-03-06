# LinXust Archive Format (`.lxt`) Specification

## 1. Endianness and Conventions
- All fixed-size integers are **little-endian**.
- Archive layout is stream-friendly.
- Versioned format with forward-compatible flags.

## 2. Top-Level Layout
`Header -> Chunk Table -> Data Blocks -> Footer`

## 3. Header (fixed + variable)

### 3.1 Fixed Header (minimum 32 bytes)
| Offset | Size | Field | Description |
|---|---:|---|---|
| 0x00 | 4 | Magic | `0x4C 0x58 0x54 0x01` (`LXT` + format major 1 marker) |
| 0x04 | 1 | VersionMajor | Initial value `1` |
| 0x05 | 1 | VersionMinor | Initial value `0` |
| 0x06 | 2 | HeaderFlags | Bitfield (encryption/compression metadata presence) |
| 0x08 | 4 | HeaderSize | Total header bytes (fixed + variable section) |
| 0x0C | 8 | ChunkCount | Number of data blocks |
| 0x14 | 8 | OriginalSize | Total uncompressed bytes |
| 0x1C | 4 | HeaderCrc32 | CRC32 of header excluding this field |

### 3.2 Variable Header Section
TLV entries (`Type:u16`, `Len:u16`, `Value:[u8; Len]`), contiguous until `HeaderSize`.
Defined types:
- `0x0001`: Archive UTF-8 comment
- `0x0002`: Argon2 parameters (encoded struct)
- `0x0003`: Salt (16 bytes minimum)
- `0x0004`: Nonce seed/base
- `0x0005`: Dictionary hint/version

## 4. Chunk Table
Immediately after header.
Each entry (32 bytes):
| Byte Range | Field |
|---|---|
| 0..8 | BlockOffset (absolute file offset) |
| 8..16 | CompressedSize |
| 16..24 | OriginalSize |
| 24..28 | BlockCrc32 (post-compression, pre-encryption bytes) |
| 28..30 | BlockFlags (compressed/encrypted/final) |
| 30..32 | Reserved |

`BlockFlags` bit assignments:
- `0x0001`: payload is compressed with the configured codec pipeline
- `0x0002`: payload is encrypted
- `0x0004`: final block in the archive

## 5. Data Blocks
- Stream of chunk payloads in table order.
- Compression method ID `0x01` = LZ77+Huffman.
- If encrypted, each block payload is AES-256-GCM sealed and includes an auth tag.

## 6. Metadata Section
For LinXust-created archives, `MetadataOffset` MUST point to a metadata section located after the data blocks and before the footer.

### 6.1 Metadata Header
| Offset | Size | Field | Description |
|---|---:|---|---|
| 0x00 | 4 | MetadataMagic | `0x4C 0x58 0x54 0x4D` (`LXTM`) |
| 0x04 | 4 | MetadataSize | Total metadata bytes including header and all entries |
| 0x08 | 8 | EntryCount | Number of archive entries |
| 0x10 | 4 | MetadataCrc32 | CRC32 of metadata excluding this field |
| 0x14 | 4 | Reserved | Must be `0` for version 1 |

### 6.2 Metadata Entries
Each metadata entry starts with a fixed header, followed by UTF-8 path bytes and an optional UTF-8 symlink target.

Fixed entry header (56 bytes):
| Byte Range | Field |
|---|---|
| 0..8 | EntryId |
| 8..16 | ParentId (`0` for root entries) |
| 16..24 | FirstChunkIndex |
| 24..28 | ChunkCount |
| 28..32 | UnixMode |
| 32..40 | MtimeUnixSeconds |
| 40..48 | OriginalSize |
| 48..50 | PathLen |
| 50..52 | LinkTargetLen |
| 52..54 | EntryFlags |
| 54..56 | Reserved |

`EntryFlags` bit assignments:
- `0x0001`: regular file
- `0x0002`: directory
- `0x0004`: symbolic link
- `0x0008`: executable bit was set when archived

Rules:
- Exactly one of `file`, `directory`, or `symbolic link` flags must be set.
- `PathLen` bytes immediately follow the fixed header and store the normalized relative path using `/` separators.
- `LinkTargetLen` bytes immediately follow the path and are present only for symbolic link entries.
- Directory entries MUST use `ChunkCount = 0`.
- `FirstChunkIndex` and `ChunkCount` map entries to contiguous chunk-table rows.

## 7. Footer (minimum 32 bytes)
| Offset from footer start | Size | Field |
|---|---:|---|
| 0x00 | 4 | FooterMagic `0x4C 0x58 0x54 0x46` (`LXTF`) |
| 0x04 | 4 | FooterSize |
| 0x08 | 8 | ChunkTableOffset |
| 0x10 | 8 | MetadataOffset (optional, else 0) |
| 0x18 | 4 | ArchiveCrc32 |
| 0x1C | 4 | FooterCrc32 |

`ArchiveCrc32` covers every byte from the start of the header through the end of the data block region, excluding the footer.

## 8. Validation Rules
- Magic/version mismatch => unsupported format error.
- `HeaderSize` and offsets MUST be bounds-checked.
- Chunk table entries MUST not overlap and MUST be monotonic.
- `MetadataOffset` MUST be `0` or point to a complete metadata section within archive bounds.
- Metadata entry paths MUST be UTF-8, normalized, relative, and MUST NOT contain `..` traversal segments or absolute roots.
- Metadata entry chunk ranges MUST stay within `ChunkCount` and MUST NOT overlap another entry's assigned chunk range.
- Duplicate normalized paths MUST be rejected.
- Auth tag failure MUST abort extraction.

## 9. Compatibility
- Minor version increments may add new TLV types.
- Unknown TLV types MUST be skipped using length.
- Future metadata entry flags MUST be ignored when unknown and not required for safe extraction.
