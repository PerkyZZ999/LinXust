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

## 5. Data Blocks
- Stream of chunk payloads in table order.
- Compression method ID `0x01` = LZ77+Huffman.
- If encrypted, each block payload is AES-256-GCM sealed and includes an auth tag.

## 6. Footer (minimum 32 bytes)
| Offset from footer start | Size | Field |
|---|---:|---|
| 0x00 | 4 | FooterMagic `0x4C 0x58 0x54 0x46` (`LXTF`) |
| 0x04 | 4 | FooterSize |
| 0x08 | 8 | ChunkTableOffset |
| 0x10 | 8 | MetadataOffset (optional, else 0) |
| 0x18 | 4 | ArchiveCrc32 |
| 0x1C | 4 | FooterCrc32 |

## 7. Validation Rules
- Magic/version mismatch => unsupported format error.
- `HeaderSize` and offsets MUST be bounds-checked.
- Chunk table entries MUST not overlap and MUST be monotonic.
- Auth tag failure MUST abort extraction.

## 8. Compatibility
- Minor version increments may add new TLV types.
- Unknown TLV types MUST be skipped using length.
