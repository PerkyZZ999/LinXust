#![allow(dead_code)]

use anyhow::{Result, bail, ensure};

const HEADER_MAGIC: [u8; 4] = [0x4C, 0x58, 0x54, 0x01];
const FOOTER_MAGIC: [u8; 4] = [0x4C, 0x58, 0x54, 0x46];
pub(crate) const FIXED_HEADER_SIZE: usize = 32;
pub(crate) const CHUNK_TABLE_ENTRY_SIZE: usize = 32;
pub(crate) const FIXED_FOOTER_SIZE: usize = 32;
const HEADER_CRC32_OFFSET: usize = 28;
const FOOTER_CRC32_OFFSET: usize = 28;
const CRC32_SIZE: usize = 4;
const CRC32_POLYNOMIAL: u32 = 0xEDB8_8320;
pub(crate) const BLOCK_FLAG_COMPRESSED: u16 = 0x0001;
pub(crate) const BLOCK_FLAG_ENCRYPTED: u16 = 0x0002;
pub(crate) const BLOCK_FLAG_FINAL: u16 = 0x0004;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HeaderTlv {
	pub(crate) kind: u16,
	pub(crate) value: Vec<u8>,
}

impl HeaderTlv {
	fn encoded_len(&self) -> usize {
		4 + self.value.len()
	}

	fn write_to(&self, output: &mut Vec<u8>) -> Result<()> {
		let value_len = self.value.len();
		ensure!(
			value_len <= u16::MAX as usize,
			"header tlv value exceeds u16 length field"
		);

		output.extend_from_slice(&self.kind.to_le_bytes());
		output.extend_from_slice(&(value_len as u16).to_le_bytes());
		output.extend_from_slice(&self.value);
		Ok(())
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LxtHeader {
	pub(crate) version_major: u8,
	pub(crate) version_minor: u8,
	pub(crate) header_flags: u16,
	pub(crate) chunk_count: u64,
	pub(crate) original_size: u64,
	pub(crate) tlvs: Vec<HeaderTlv>,
}

impl LxtHeader {
	pub(crate) fn serialized_len(&self) -> usize {
		FIXED_HEADER_SIZE
			+ self
				.tlvs
				.iter()
				.map(HeaderTlv::encoded_len)
				.sum::<usize>()
	}

	pub(crate) fn to_bytes(&self) -> Result<Vec<u8>> {
		let header_len = self.serialized_len();

		if header_len > u32::MAX as usize {
			bail!("header size exceeds u32 field capacity");
		}

		let mut bytes = Vec::with_capacity(header_len);
		bytes.extend_from_slice(&HEADER_MAGIC);
		bytes.push(self.version_major);
		bytes.push(self.version_minor);
		bytes.extend_from_slice(&self.header_flags.to_le_bytes());
		bytes.extend_from_slice(&(header_len as u32).to_le_bytes());
		bytes.extend_from_slice(&self.chunk_count.to_le_bytes());
		bytes.extend_from_slice(&self.original_size.to_le_bytes());
		bytes.extend_from_slice(&0_u32.to_le_bytes());

		for tlv in &self.tlvs {
			tlv.write_to(&mut bytes)?;
		}

		let crc32 = crc32_skipping_range(&bytes, HEADER_CRC32_OFFSET, CRC32_SIZE);
		bytes[HEADER_CRC32_OFFSET..HEADER_CRC32_OFFSET + CRC32_SIZE]
			.copy_from_slice(&crc32.to_le_bytes());

		Ok(bytes)
	}

	pub(crate) fn from_bytes(data: &[u8]) -> Result<Self> {
		ensure!(data.len() >= FIXED_HEADER_SIZE, "header is shorter than 32 bytes");
		ensure!(&data[..4] == HEADER_MAGIC, "header magic mismatch");

		let version_major = data[4];
		let version_minor = data[5];
		ensure!(version_major == 1, "unsupported header major version {}", version_major);

		let header_size = read_u32(data, 8)? as usize;
		ensure!(
			header_size >= FIXED_HEADER_SIZE,
			"header size is smaller than the fixed header"
		);
		ensure!(header_size <= data.len(), "header size exceeds available bytes");

		let header_bytes = &data[..header_size];
		let stored_crc32 = read_u32(header_bytes, HEADER_CRC32_OFFSET)?;
		let computed_crc32 =
			crc32_skipping_range(header_bytes, HEADER_CRC32_OFFSET, CRC32_SIZE);
		ensure!(stored_crc32 == computed_crc32, "header crc32 mismatch");

		let mut tlvs = Vec::new();
		let mut offset = FIXED_HEADER_SIZE;
		while offset < header_size {
			ensure!(header_size - offset >= 4, "header tlv is truncated");

			let kind = read_u16(header_bytes, offset)?;
			let value_len = usize::from(read_u16(header_bytes, offset + 2)?);
			offset += 4;

			let value_end = offset
				.checked_add(value_len)
				.ok_or_else(|| anyhow::anyhow!("header tlv length overflowed usize"))?;
			ensure!(value_end <= header_size, "header tlv exceeds header size");

			tlvs.push(HeaderTlv {
				kind,
				value: header_bytes[offset..value_end].to_vec(),
			});
			offset = value_end;
		}

		Ok(Self {
			version_major,
			version_minor,
			header_flags: read_u16(header_bytes, 6)?,
			chunk_count: read_u64(header_bytes, 12)?,
			original_size: read_u64(header_bytes, 20)?,
			tlvs,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChunkTableEntry {
	pub(crate) block_offset: u64,
	pub(crate) compressed_size: u64,
	pub(crate) original_size: u64,
	pub(crate) block_crc32: u32,
	pub(crate) block_flags: u16,
}

impl ChunkTableEntry {
	pub(crate) fn from_bytes(data: &[u8]) -> Result<Self> {
		ensure!(
			data.len() == CHUNK_TABLE_ENTRY_SIZE,
			"chunk table entry size mismatch"
		);
		ensure!(
			read_u16(data, 30)? == 0,
			"chunk table reserved field must be zero"
		);

		Ok(Self {
			block_offset: read_u64(data, 0)?,
			compressed_size: read_u64(data, 8)?,
			original_size: read_u64(data, 16)?,
			block_crc32: read_u32(data, 24)?,
			block_flags: read_u16(data, 28)?,
		})
	}
}

pub(crate) fn serialize_chunk_table(entries: &[ChunkTableEntry]) -> Vec<u8> {
	let mut bytes = Vec::with_capacity(entries.len() * CHUNK_TABLE_ENTRY_SIZE);

	for entry in entries {
		bytes.extend_from_slice(&entry.block_offset.to_le_bytes());
		bytes.extend_from_slice(&entry.compressed_size.to_le_bytes());
		bytes.extend_from_slice(&entry.original_size.to_le_bytes());
		bytes.extend_from_slice(&entry.block_crc32.to_le_bytes());
		bytes.extend_from_slice(&entry.block_flags.to_le_bytes());
		bytes.extend_from_slice(&0_u16.to_le_bytes());
	}

	bytes
}

pub(crate) fn parse_chunk_table(data: &[u8], chunk_count: u64) -> Result<Vec<ChunkTableEntry>> {
	let chunk_count =
		usize::try_from(chunk_count).map_err(|_| anyhow::anyhow!("chunk count exceeds usize"))?;
	let expected_len = chunk_count
		.checked_mul(CHUNK_TABLE_ENTRY_SIZE)
		.ok_or_else(|| anyhow::anyhow!("chunk table length overflowed usize"))?;
	ensure!(data.len() == expected_len, "chunk table size mismatch");

	data.chunks_exact(CHUNK_TABLE_ENTRY_SIZE)
		.map(ChunkTableEntry::from_bytes)
		.collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LxtFooter {
	pub(crate) chunk_table_offset: u64,
	pub(crate) metadata_offset: u64,
	pub(crate) archive_crc32: u32,
}

impl LxtFooter {
	pub(crate) fn to_bytes(&self) -> Vec<u8> {
		let mut bytes = Vec::with_capacity(FIXED_FOOTER_SIZE);
		bytes.extend_from_slice(&FOOTER_MAGIC);
		bytes.extend_from_slice(&(FIXED_FOOTER_SIZE as u32).to_le_bytes());
		bytes.extend_from_slice(&self.chunk_table_offset.to_le_bytes());
		bytes.extend_from_slice(&self.metadata_offset.to_le_bytes());
		bytes.extend_from_slice(&self.archive_crc32.to_le_bytes());
		bytes.extend_from_slice(&0_u32.to_le_bytes());

		let crc32 = crc32_skipping_range(&bytes, FOOTER_CRC32_OFFSET, CRC32_SIZE);
		bytes[FOOTER_CRC32_OFFSET..FOOTER_CRC32_OFFSET + CRC32_SIZE]
			.copy_from_slice(&crc32.to_le_bytes());

		bytes
	}

	pub(crate) fn from_bytes(data: &[u8]) -> Result<Self> {
		ensure!(data.len() == FIXED_FOOTER_SIZE, "footer size mismatch");
		ensure!(&data[..4] == FOOTER_MAGIC, "footer magic mismatch");
		ensure!(
			read_u32(data, 4)? == FIXED_FOOTER_SIZE as u32,
			"unsupported footer size"
		);

		let stored_crc32 = read_u32(data, FOOTER_CRC32_OFFSET)?;
		let computed_crc32 =
			crc32_skipping_range(data, FOOTER_CRC32_OFFSET, CRC32_SIZE);
		ensure!(stored_crc32 == computed_crc32, "footer crc32 mismatch");

		Ok(Self {
			chunk_table_offset: read_u64(data, 8)?,
			metadata_offset: read_u64(data, 16)?,
			archive_crc32: read_u32(data, 24)?,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedArchiveLayout {
	header: LxtHeader,
	chunk_table: Vec<ChunkTableEntry>,
	footer: LxtFooter,
}

impl ParsedArchiveLayout {
	fn from_bytes(data: &[u8]) -> Result<Self> {
		ensure!(
			data.len() >= FIXED_HEADER_SIZE + FIXED_FOOTER_SIZE,
			"archive is too short to contain header and footer"
		);

		let header = LxtHeader::from_bytes(data)?;
		let footer_start = data.len() - FIXED_FOOTER_SIZE;
		let footer = LxtFooter::from_bytes(&data[footer_start..])?;

		let chunk_table_offset = usize::try_from(footer.chunk_table_offset)
			.map_err(|_| anyhow::anyhow!("chunk table offset exceeds usize"))?;
		ensure!(
			chunk_table_offset == header.serialized_len(),
			"chunk table offset does not immediately follow the header"
		);

		if footer.metadata_offset != 0 {
			let metadata_offset = usize::try_from(footer.metadata_offset)
				.map_err(|_| anyhow::anyhow!("metadata offset exceeds usize"))?;
			ensure!(
				metadata_offset < footer_start,
				"metadata offset must point before the footer"
			);
		}

		let chunk_table_len = usize::try_from(header.chunk_count)
			.map_err(|_| anyhow::anyhow!("chunk count exceeds usize"))?
			.checked_mul(CHUNK_TABLE_ENTRY_SIZE)
			.ok_or_else(|| anyhow::anyhow!("chunk table length overflowed usize"))?;
		let chunk_table_end = chunk_table_offset
			.checked_add(chunk_table_len)
			.ok_or_else(|| anyhow::anyhow!("chunk table end overflowed usize"))?;
		ensure!(chunk_table_end <= footer_start, "chunk table overlaps the footer");

		let chunk_table =
			parse_chunk_table(&data[chunk_table_offset..chunk_table_end], header.chunk_count)?;
		validate_chunk_offsets(&chunk_table, chunk_table_end, footer_start)?;
		ensure!(
			footer.archive_crc32 == crc32(&data[..footer_start]),
			"archive crc32 mismatch"
		);

		Ok(Self {
			header,
			chunk_table,
			footer,
		})
	}
}

pub(crate) fn encode_archive(data: &[u8]) -> Result<Vec<u8>> {
	if data.is_empty() {
		return encode_archive_blocks(&[]);
	}

	encode_archive_blocks(&[data.to_vec()])
}

pub(crate) fn encode_archive_blocks(blocks: &[Vec<u8>]) -> Result<Vec<u8>> {
	let compressed_blocks = blocks
		.iter()
		.map(|block| crate::compress_phase2(block))
		.collect::<Vec<_>>();
	let original_size = blocks.iter().try_fold(0_u64, |total, block| {
		total
			.checked_add(block.len() as u64)
			.ok_or_else(|| anyhow::anyhow!("archive original size overflowed u64"))
	})?;

	let header = LxtHeader {
		version_major: 1,
		version_minor: 0,
		header_flags: 0,
		chunk_count: compressed_blocks.len() as u64,
		original_size,
		tlvs: Vec::new(),
	};
	let header_bytes = header.to_bytes()?;
	let chunk_table_len = compressed_blocks
		.len()
		.checked_mul(CHUNK_TABLE_ENTRY_SIZE)
		.ok_or_else(|| anyhow::anyhow!("chunk table length overflowed usize"))?;
	let mut next_block_offset = header_bytes
		.len()
		.checked_add(chunk_table_len)
		.ok_or_else(|| anyhow::anyhow!("block offset overflowed usize"))?;
	let mut chunk_table = Vec::with_capacity(compressed_blocks.len());

	for (index, compressed_block) in compressed_blocks.iter().enumerate() {
		let compressed_len = compressed_block.len();
		let block_offset = next_block_offset;
		next_block_offset = next_block_offset
			.checked_add(compressed_len)
			.ok_or_else(|| anyhow::anyhow!("block offset overflowed usize"))?;

		let mut block_flags = BLOCK_FLAG_COMPRESSED;
		if index + 1 == compressed_blocks.len() {
			block_flags |= BLOCK_FLAG_FINAL;
		}

		chunk_table.push(ChunkTableEntry {
			block_offset: block_offset as u64,
			compressed_size: compressed_len as u64,
			original_size: blocks[index].len() as u64,
			block_crc32: crc32(compressed_block),
			block_flags,
		});
	}

	let chunk_table_bytes = serialize_chunk_table(&chunk_table);
	let mut archive_prefix = Vec::with_capacity(next_block_offset);
	archive_prefix.extend_from_slice(&header_bytes);
	archive_prefix.extend_from_slice(&chunk_table_bytes);
	for compressed_block in &compressed_blocks {
		archive_prefix.extend_from_slice(compressed_block);
	}

	let footer = LxtFooter {
		chunk_table_offset: header_bytes.len() as u64,
		metadata_offset: 0,
		archive_crc32: crc32(&archive_prefix),
	}
	.to_bytes();

	archive_prefix.extend_from_slice(&footer);
	Ok(archive_prefix)
}

pub(crate) fn decode_archive(archive: &[u8]) -> Result<Vec<u8>> {
	let layout = ParsedArchiveLayout::from_bytes(archive)?;
	let expected_len = usize::try_from(layout.header.original_size)
		.map_err(|_| anyhow::anyhow!("archive original size exceeds usize"))?;
	let mut output = Vec::with_capacity(expected_len);

	for (index, entry) in layout.chunk_table.iter().enumerate() {
		ensure!(
			entry.block_flags & BLOCK_FLAG_COMPRESSED != 0,
			"chunk {} is not marked as compressed",
			index
		);
		ensure!(
			entry.block_flags & BLOCK_FLAG_ENCRYPTED == 0,
			"encrypted chunks are not supported yet"
		);

		let block_offset = usize::try_from(entry.block_offset)
			.map_err(|_| anyhow::anyhow!("chunk offset exceeds usize"))?;
		let block_len = usize::try_from(entry.compressed_size)
			.map_err(|_| anyhow::anyhow!("compressed size exceeds usize"))?;
		let block_end = block_offset
			.checked_add(block_len)
			.ok_or_else(|| anyhow::anyhow!("chunk end overflowed usize"))?;
		let block = &archive[block_offset..block_end];

		ensure!(
			crc32(block) == entry.block_crc32,
			"block crc32 mismatch for chunk {}",
			index
		);

		let decompressed = crate::decompress_phase2(block)
			.ok_or_else(|| anyhow::anyhow!("phase2 decode failed for chunk {}", index))?;
		let original_size = usize::try_from(entry.original_size)
			.map_err(|_| anyhow::anyhow!("original size exceeds usize"))?;
		ensure!(
			decompressed.len() == original_size,
			"chunk {} original size mismatch",
			index
		);

		output.extend_from_slice(&decompressed);
	}

	ensure!(output.len() == expected_len, "decoded archive size mismatch");
	Ok(output)
}

pub(crate) fn validate_chunk_offsets(
	entries: &[ChunkTableEntry],
	data_start: usize,
	footer_start: usize,
) -> Result<()> {
	let mut previous_end = data_start;

	for (index, entry) in entries.iter().enumerate() {
		let block_offset = usize::try_from(entry.block_offset)
			.map_err(|_| anyhow::anyhow!("chunk offset exceeds usize"))?;
		let block_size = usize::try_from(entry.compressed_size)
			.map_err(|_| anyhow::anyhow!("compressed size exceeds usize"))?;
		let block_end = block_offset
			.checked_add(block_size)
			.ok_or_else(|| anyhow::anyhow!("chunk end overflowed usize"))?;

		ensure!(
			block_offset >= data_start,
			"chunk entry {} starts before the data section",
			index
		);
		ensure!(
			block_offset >= previous_end,
			"chunk entry {} is not monotonic or overlaps a previous chunk",
			index
		);
		ensure!(
			block_end <= footer_start,
			"chunk entry {} extends beyond the footer",
			index
		);

		if index + 1 == entries.len() {
			ensure!(
				entry.block_flags & BLOCK_FLAG_FINAL != 0,
				"last chunk must be marked final"
			);
		} else {
			ensure!(
				entry.block_flags & BLOCK_FLAG_FINAL == 0,
				"non-final chunk {} has the final flag set",
				index
			);
		}

		previous_end = block_end;
	}

	Ok(())
}

fn read_u16(data: &[u8], offset: usize) -> Result<u16> {
	let end = offset
		.checked_add(2)
		.ok_or_else(|| anyhow::anyhow!("u16 offset overflowed usize"))?;
	ensure!(end <= data.len(), "buffer too small for u16 at offset {}", offset);
	Ok(u16::from_le_bytes(
		data[offset..end].try_into().expect("u16 slice"),
	))
}

fn read_u32(data: &[u8], offset: usize) -> Result<u32> {
	let end = offset
		.checked_add(4)
		.ok_or_else(|| anyhow::anyhow!("u32 offset overflowed usize"))?;
	ensure!(end <= data.len(), "buffer too small for u32 at offset {}", offset);
	Ok(u32::from_le_bytes(
		data[offset..end].try_into().expect("u32 slice"),
	))
}

fn read_u64(data: &[u8], offset: usize) -> Result<u64> {
	let end = offset
		.checked_add(8)
		.ok_or_else(|| anyhow::anyhow!("u64 offset overflowed usize"))?;
	ensure!(end <= data.len(), "buffer too small for u64 at offset {}", offset);
	Ok(u64::from_le_bytes(
		data[offset..end].try_into().expect("u64 slice"),
	))
}

pub(crate) fn crc32_init() -> u32 {
	0xFFFF_FFFF
}

fn crc32_update_byte(mut crc: u32, byte: u8) -> u32 {
	crc ^= u32::from(byte);
	for _ in 0..8 {
		if crc & 1 == 1 {
			crc = (crc >> 1) ^ CRC32_POLYNOMIAL;
		} else {
			crc >>= 1;
		}
	}
	crc
}

pub(crate) fn crc32_update(mut crc: u32, data: &[u8]) -> u32 {
	for byte in data {
		crc = crc32_update_byte(crc, *byte);
	}
	crc
}

pub(crate) fn crc32_finalize(crc: u32) -> u32 {
	!crc
}

pub(crate) fn crc32(data: &[u8]) -> u32 {
	crc32_finalize(crc32_update(crc32_init(), data))
}

fn crc32_skipping_range(data: &[u8], skip_start: usize, skip_len: usize) -> u32 {
	let mut crc = crc32_init();

	for (index, byte) in data.iter().enumerate() {
		if index >= skip_start && index < skip_start.saturating_add(skip_len) {
			continue;
		}

		crc = crc32_update_byte(crc, *byte);
	}

	crc32_finalize(crc)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn read_u16_le(bytes: &[u8], offset: usize) -> u16 {
		u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("u16 slice"))
	}

	fn read_u32_le(bytes: &[u8], offset: usize) -> u32 {
		u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("u32 slice"))
	}

	fn read_u64_le(bytes: &[u8], offset: usize) -> u64 {
		u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("u64 slice"))
	}

	fn finalize_archive(mut prefix: Vec<u8>, chunk_table_offset: usize) -> Vec<u8> {
		let footer = LxtFooter {
			chunk_table_offset: chunk_table_offset as u64,
			metadata_offset: 0,
			archive_crc32: crc32(&prefix),
		}
		.to_bytes();
		prefix.extend_from_slice(&footer);
		prefix
	}

	fn rewrite_header_crc(header_bytes: &mut [u8]) {
		let crc32 = crc32_skipping_range(header_bytes, HEADER_CRC32_OFFSET, CRC32_SIZE);
		header_bytes[HEADER_CRC32_OFFSET..HEADER_CRC32_OFFSET + CRC32_SIZE]
			.copy_from_slice(&crc32.to_le_bytes());
	}

	#[test]
	fn crc32_matches_reference_vector() {
		assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
	}

	#[test]
	fn header_serialization_writes_fixed_fields_and_tlvs() {
		let header = LxtHeader {
			version_major: 1,
			version_minor: 0,
			header_flags: 0x0003,
			chunk_count: 2,
			original_size: 4096,
			tlvs: vec![
				HeaderTlv {
					kind: 0x0001,
					value: b"hello".to_vec(),
				},
				HeaderTlv {
					kind: 0x0003,
					value: vec![7_u8; 16],
				},
			],
		};

		let bytes = header.to_bytes().expect("header should serialize");

		assert_eq!(&bytes[..4], &HEADER_MAGIC);
		assert_eq!(bytes[4], 1);
		assert_eq!(bytes[5], 0);
		assert_eq!(read_u16_le(&bytes, 6), 0x0003);
		assert_eq!(read_u32_le(&bytes, 8) as usize, bytes.len());
		assert_eq!(read_u64_le(&bytes, 12), 2);
		assert_eq!(read_u64_le(&bytes, 20), 4096);
		assert_eq!(read_u32_le(&bytes, 28), crc32_skipping_range(&bytes, 28, 4));

		assert_eq!(read_u16_le(&bytes, 32), 0x0001);
		assert_eq!(read_u16_le(&bytes, 34), 5);
		assert_eq!(&bytes[36..41], b"hello");
		assert_eq!(read_u16_le(&bytes, 41), 0x0003);
		assert_eq!(read_u16_le(&bytes, 43), 16);
		assert_eq!(&bytes[45..61], vec![7_u8; 16].as_slice());
	}

	#[test]
	fn chunk_table_serialization_writes_fixed_width_entries() {
		let bytes = serialize_chunk_table(&[
			ChunkTableEntry {
				block_offset: 64,
				compressed_size: 20,
				original_size: 32,
				block_crc32: 0xABCD_1234,
				block_flags: BLOCK_FLAG_COMPRESSED,
			},
			ChunkTableEntry {
				block_offset: 84,
				compressed_size: 18,
				original_size: 24,
				block_crc32: 0x1234_ABCD,
				block_flags: BLOCK_FLAG_COMPRESSED | BLOCK_FLAG_FINAL,
			},
		]);

		assert_eq!(bytes.len(), CHUNK_TABLE_ENTRY_SIZE * 2);
		assert_eq!(read_u64_le(&bytes, 0), 64);
		assert_eq!(read_u64_le(&bytes, 8), 20);
		assert_eq!(read_u64_le(&bytes, 16), 32);
		assert_eq!(read_u32_le(&bytes, 24), 0xABCD_1234);
		assert_eq!(read_u16_le(&bytes, 28), BLOCK_FLAG_COMPRESSED);
		assert_eq!(read_u16_le(&bytes, 30), 0);
		assert_eq!(read_u64_le(&bytes, 32), 84);
		assert_eq!(
			read_u16_le(&bytes, 60),
			BLOCK_FLAG_COMPRESSED | BLOCK_FLAG_FINAL
		);
	}

	#[test]
	fn footer_serialization_writes_crc_and_offsets() {
		let footer = LxtFooter {
			chunk_table_offset: 128,
			metadata_offset: 512,
			archive_crc32: 0xDEAD_BEEF,
		};

		let bytes = footer.to_bytes();

		assert_eq!(&bytes[..4], &FOOTER_MAGIC);
		assert_eq!(read_u32_le(&bytes, 4), FIXED_FOOTER_SIZE as u32);
		assert_eq!(read_u64_le(&bytes, 8), 128);
		assert_eq!(read_u64_le(&bytes, 16), 512);
		assert_eq!(read_u32_le(&bytes, 24), 0xDEAD_BEEF);
		assert_eq!(read_u32_le(&bytes, 28), crc32_skipping_range(&bytes, 28, 4));
	}

	#[test]
	fn archive_layout_parser_accepts_valid_offsets_and_crcs() {
		let header = LxtHeader {
			version_major: 1,
			version_minor: 0,
			header_flags: 0,
			chunk_count: 1,
			original_size: 4,
			tlvs: Vec::new(),
		};
		let header_bytes = header.to_bytes().expect("header should serialize");
		let block_data = vec![1_u8, 2, 3, 4];
		let block_offset = (header_bytes.len() + CHUNK_TABLE_ENTRY_SIZE) as u64;
		let chunk_table = serialize_chunk_table(&[ChunkTableEntry {
			block_offset,
			compressed_size: block_data.len() as u64,
			original_size: block_data.len() as u64,
			block_crc32: crc32(&block_data),
			block_flags: BLOCK_FLAG_COMPRESSED | BLOCK_FLAG_FINAL,
		}]);

		let mut archive_prefix = Vec::new();
		archive_prefix.extend_from_slice(&header_bytes);
		archive_prefix.extend_from_slice(&chunk_table);
		archive_prefix.extend_from_slice(&block_data);
		let archive = finalize_archive(archive_prefix, header_bytes.len());

		let parsed = ParsedArchiveLayout::from_bytes(&archive).expect("archive should validate");
		assert_eq!(parsed.header.chunk_count, 1);
		assert_eq!(parsed.chunk_table[0].block_offset, block_offset);
		assert_eq!(parsed.footer.chunk_table_offset, header_bytes.len() as u64);
	}

	#[test]
	fn archive_layout_parser_rejects_unsupported_versions() {
		let header = LxtHeader {
			version_major: 1,
			version_minor: 0,
			header_flags: 0,
			chunk_count: 0,
			original_size: 0,
			tlvs: Vec::new(),
		};
		let header_bytes = header.to_bytes().expect("header should serialize");
		let mut archive = finalize_archive(header_bytes, FIXED_HEADER_SIZE);

		archive[4] = 2;
		rewrite_header_crc(&mut archive[..FIXED_HEADER_SIZE]);

		let error = ParsedArchiveLayout::from_bytes(&archive).expect_err("version should fail");
		assert!(error.to_string().contains("unsupported header major version"));
	}

	#[test]
	fn archive_layout_parser_rejects_crc_mismatches() {
		let header = LxtHeader {
			version_major: 1,
			version_minor: 0,
			header_flags: 0,
			chunk_count: 0,
			original_size: 0,
			tlvs: Vec::new(),
		};
		let header_bytes = header.to_bytes().expect("header should serialize");
		let mut archive = finalize_archive(header_bytes, FIXED_HEADER_SIZE);

		archive[28] ^= 0xFF;

		let error = ParsedArchiveLayout::from_bytes(&archive).expect_err("crc should fail");
		assert!(error.to_string().contains("header crc32 mismatch"));
	}

	#[test]
	fn archive_layout_parser_rejects_overlapping_offsets() {
		let header = LxtHeader {
			version_major: 1,
			version_minor: 0,
			header_flags: 0,
			chunk_count: 2,
			original_size: 12,
			tlvs: Vec::new(),
		};
		let header_bytes = header.to_bytes().expect("header should serialize");
		let data_start = header_bytes.len() + (CHUNK_TABLE_ENTRY_SIZE * 2);
		let block_data = vec![1_u8; 12];
		let chunk_table = serialize_chunk_table(&[
			ChunkTableEntry {
				block_offset: data_start as u64,
				compressed_size: 8,
				original_size: 8,
				block_crc32: crc32(&block_data[..8]),
				block_flags: BLOCK_FLAG_COMPRESSED,
			},
			ChunkTableEntry {
				block_offset: (data_start + 4) as u64,
				compressed_size: 4,
				original_size: 4,
				block_crc32: crc32(&block_data[4..8]),
				block_flags: BLOCK_FLAG_COMPRESSED | BLOCK_FLAG_FINAL,
			},
		]);

		let mut archive_prefix = Vec::new();
		archive_prefix.extend_from_slice(&header_bytes);
		archive_prefix.extend_from_slice(&chunk_table);
		archive_prefix.extend_from_slice(&block_data);
		let archive = finalize_archive(archive_prefix, header_bytes.len());

		let error =
			ParsedArchiveLayout::from_bytes(&archive).expect_err("overlap should fail");
		assert!(error.to_string().contains("is not monotonic or overlaps"));
	}

	#[test]
	fn phase2_codec_roundtrips_through_lxt_archive() {
		let input = b"linxust archive block archive block archive block";
		let archive = encode_archive(input).expect("archive should encode");
		let decoded = decode_archive(&archive).expect("archive should decode");
		let parsed = ParsedArchiveLayout::from_bytes(&archive).expect("archive should validate");

		assert_eq!(decoded, input);
		assert_eq!(parsed.header.chunk_count, 1);
		assert_eq!(
			parsed.chunk_table[0].block_flags,
			BLOCK_FLAG_COMPRESSED | BLOCK_FLAG_FINAL
		);
	}

	#[test]
	fn phase2_codec_handles_empty_archives() {
		let archive = encode_archive(b"").expect("empty archive should encode");
		let decoded = decode_archive(&archive).expect("empty archive should decode");
		let parsed = ParsedArchiveLayout::from_bytes(&archive).expect("archive should validate");

		assert!(decoded.is_empty());
		assert_eq!(parsed.header.chunk_count, 0);
		assert!(parsed.chunk_table.is_empty());
	}
}
