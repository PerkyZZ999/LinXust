#![allow(dead_code)]

use anyhow::{Context, Result, ensure};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::lxt::{
	BLOCK_FLAG_COMPRESSED, BLOCK_FLAG_ENCRYPTED, CHUNK_TABLE_ENTRY_SIZE,
	ChunkTableEntry, FIXED_FOOTER_SIZE, FIXED_HEADER_SIZE, LxtFooter, LxtHeader,
	crc32, crc32_finalize, crc32_init, crc32_update, parse_chunk_table,
	serialize_chunk_table, validate_chunk_offsets,
};

struct TemporaryFile {
	path: PathBuf,
}

impl TemporaryFile {
	fn create() -> Result<(Self, File)> {
		let mut attempt = 0_u32;

		loop {
			let timestamp = SystemTime::now()
				.duration_since(UNIX_EPOCH)
				.context("system clock is before unix epoch")?
				.as_nanos();
			let path = env::temp_dir().join(format!(
				"linxust-{}-{}-{}.tmp",
				process::id(),
				timestamp,
				attempt,
			));

			match OpenOptions::new()
				.read(true)
				.write(true)
				.create_new(true)
				.open(&path)
			{
				Ok(file) => return Ok((Self { path }, file)),
				Err(error) if error.kind() == ErrorKind::AlreadyExists => {
					attempt += 1;
				}
				Err(error) => {
					return Err(error).with_context(|| {
						format!("failed to create temporary file at {}", path.display())
					});
				}
			}
		}
	}

	fn reopen(&self) -> Result<File> {
		File::open(&self.path)
			.with_context(|| format!("failed to reopen temporary file {}", self.path.display()))
	}

	fn path(&self) -> &Path {
		&self.path
	}
}

impl Drop for TemporaryFile {
	fn drop(&mut self) {
		let _ = fs::remove_file(&self.path);
	}
}

pub(crate) fn write_archive_file(
	input_path: &Path,
	output_path: &Path,
	chunk_size: usize,
) -> Result<()> {
	ensure!(chunk_size > 0, "chunk size must be greater than zero");

	let input_file =
		File::open(input_path).with_context(|| format!("failed to open {}", input_path.display()))?;
	let mut reader = BufReader::with_capacity(chunk_size, input_file);
	let (temp_blocks, temp_file) = TemporaryFile::create()?;
	let mut temp_writer = BufWriter::with_capacity(chunk_size, temp_file);
	let mut prepared_chunks = Vec::new();
	let mut buffer = vec![0_u8; chunk_size];
	let mut original_size = 0_u64;

	loop {
		let bytes_read = reader
			.read(&mut buffer)
			.with_context(|| format!("failed reading {}", input_path.display()))?;
		if bytes_read == 0 {
			break;
		}

		let block = &buffer[..bytes_read];
		let compressed_block = crate::compress_phase2(block);
		temp_writer
			.write_all(&compressed_block)
			.context("failed to write compressed block to temporary file")?;

		prepared_chunks.push((compressed_block.len() as u64, bytes_read as u64, crc32(&compressed_block)));
		original_size = original_size
			.checked_add(bytes_read as u64)
			.ok_or_else(|| anyhow::anyhow!("archive original size overflowed u64"))?;
	}

	temp_writer.flush().context("failed to flush temporary block file")?;
	drop(temp_writer);

	let header = LxtHeader {
		version_major: 1,
		version_minor: 0,
		header_flags: 0,
		chunk_count: prepared_chunks.len() as u64,
		original_size,
		tlvs: Vec::new(),
	};
	let header_bytes = header.to_bytes()?;
	let mut next_block_offset = header_bytes
		.len()
		.checked_add(prepared_chunks.len() * CHUNK_TABLE_ENTRY_SIZE)
		.ok_or_else(|| anyhow::anyhow!("block offset overflowed usize"))? as u64;
	let mut chunk_table = Vec::with_capacity(prepared_chunks.len());

	for (index, (compressed_size, block_original_size, block_crc32)) in prepared_chunks.iter().enumerate() {
		let mut block_flags = BLOCK_FLAG_COMPRESSED;
		if index + 1 == prepared_chunks.len() {
			block_flags |= crate::lxt::BLOCK_FLAG_FINAL;
		}

		chunk_table.push(ChunkTableEntry {
			block_offset: next_block_offset,
			compressed_size: *compressed_size,
			original_size: *block_original_size,
			block_crc32: *block_crc32,
			block_flags,
		});
		next_block_offset = next_block_offset
			.checked_add(*compressed_size)
			.ok_or_else(|| anyhow::anyhow!("block offset overflowed u64"))?;
	}

	let chunk_table_bytes = serialize_chunk_table(&chunk_table);
	let output_file = File::create(output_path)
		.with_context(|| format!("failed to create {}", output_path.display()))?;
	let mut writer = BufWriter::with_capacity(chunk_size, output_file);
	let mut archive_crc = crc32_init();

	writer
		.write_all(&header_bytes)
		.with_context(|| format!("failed to write {}", output_path.display()))?;
	archive_crc = crc32_update(archive_crc, &header_bytes);

	writer
		.write_all(&chunk_table_bytes)
		.with_context(|| format!("failed to write {}", output_path.display()))?;
	archive_crc = crc32_update(archive_crc, &chunk_table_bytes);

	let mut temp_reader = BufReader::with_capacity(chunk_size, temp_blocks.reopen()?);
	loop {
		let bytes_read = temp_reader
			.read(&mut buffer)
			.context("failed to read compressed blocks from temporary file")?;
		if bytes_read == 0 {
			break;
		}

		writer
			.write_all(&buffer[..bytes_read])
			.with_context(|| format!("failed to write {}", output_path.display()))?;
		archive_crc = crc32_update(archive_crc, &buffer[..bytes_read]);
	}

	let footer = LxtFooter {
		chunk_table_offset: header_bytes.len() as u64,
		metadata_offset: 0,
		archive_crc32: crc32_finalize(archive_crc),
	}
	.to_bytes();
	writer
		.write_all(&footer)
		.with_context(|| format!("failed to write {}", output_path.display()))?;
	writer.flush().with_context(|| format!("failed to flush {}", output_path.display()))?;

	Ok(())
}

pub(crate) fn read_archive_file(
	archive_path: &Path,
	output_path: &Path,
	buffer_size: usize,
) -> Result<()> {
	ensure!(buffer_size > 0, "buffer size must be greater than zero");

	let archive_file = File::open(archive_path)
		.with_context(|| format!("failed to open {}", archive_path.display()))?;
	let archive_len = archive_file
		.metadata()
		.with_context(|| format!("failed to stat {}", archive_path.display()))?
		.len();
	ensure!(
		archive_len >= (FIXED_HEADER_SIZE + FIXED_FOOTER_SIZE) as u64,
		"archive is too short to contain header and footer"
	);

	let mut reader = BufReader::with_capacity(buffer_size, archive_file);
	let footer_start = archive_len - FIXED_FOOTER_SIZE as u64;
	reader
		.seek(SeekFrom::Start(footer_start))
		.context("failed to seek to footer")?;
	let mut footer_bytes = [0_u8; FIXED_FOOTER_SIZE];
	reader
		.read_exact(&mut footer_bytes)
		.context("failed to read footer")?;
	let footer = LxtFooter::from_bytes(&footer_bytes)?;

	reader
		.seek(SeekFrom::Start(0))
		.context("failed to seek to header")?;
	let mut fixed_header = [0_u8; FIXED_HEADER_SIZE];
	reader
		.read_exact(&mut fixed_header)
		.context("failed to read fixed header")?;
	let header_size = u32::from_le_bytes(
		fixed_header[8..12]
			.try_into()
			.expect("header size slice should be four bytes"),
	) as usize;
	ensure!(header_size >= FIXED_HEADER_SIZE, "header size is smaller than the fixed header");

	let mut header_bytes = vec![0_u8; header_size];
	header_bytes[..FIXED_HEADER_SIZE].copy_from_slice(&fixed_header);
	if header_size > FIXED_HEADER_SIZE {
		reader
			.read_exact(&mut header_bytes[FIXED_HEADER_SIZE..])
			.context("failed to read variable header section")?;
	}
	let header = LxtHeader::from_bytes(&header_bytes)?;
	ensure!(
		footer.chunk_table_offset == header_bytes.len() as u64,
		"chunk table offset does not immediately follow the header"
	);

	let chunk_table_len = usize::try_from(header.chunk_count)
		.map_err(|_| anyhow::anyhow!("chunk count exceeds usize"))?
		.checked_mul(CHUNK_TABLE_ENTRY_SIZE)
		.ok_or_else(|| anyhow::anyhow!("chunk table length overflowed usize"))?;
	let mut chunk_table_bytes = vec![0_u8; chunk_table_len];
	reader
		.read_exact(&mut chunk_table_bytes)
		.context("failed to read chunk table")?;
	let chunk_table = parse_chunk_table(&chunk_table_bytes, header.chunk_count)?;
	validate_chunk_offsets(
		&chunk_table,
		header_bytes.len() + chunk_table_len,
		footer_start as usize,
	)?;

	reader
		.seek(SeekFrom::Start(0))
		.context("failed to rewind for archive crc validation")?;
	let mut archive_crc = crc32_init();
	let mut buffer = vec![0_u8; buffer_size];
	let mut remaining = footer_start as usize;
	while remaining > 0 {
		let bytes_to_read = remaining.min(buffer.len());
		reader
			.read_exact(&mut buffer[..bytes_to_read])
			.context("failed while validating archive crc")?;
		archive_crc = crc32_update(archive_crc, &buffer[..bytes_to_read]);
		remaining -= bytes_to_read;
	}
	ensure!(
		footer.archive_crc32 == crc32_finalize(archive_crc),
		"archive crc32 mismatch"
	);

	let output_file = File::create(output_path)
		.with_context(|| format!("failed to create {}", output_path.display()))?;
	let mut writer = BufWriter::with_capacity(buffer_size, output_file);
	let mut bytes_written = 0_usize;

	for (index, entry) in chunk_table.iter().enumerate() {
		ensure!(
			entry.block_flags & BLOCK_FLAG_COMPRESSED != 0,
			"chunk {} is not marked as compressed",
			index
		);
		ensure!(
			entry.block_flags & BLOCK_FLAG_ENCRYPTED == 0,
			"encrypted chunks are not supported yet"
		);

		let block_offset = entry.block_offset;
		let block_len = usize::try_from(entry.compressed_size)
			.map_err(|_| anyhow::anyhow!("compressed size exceeds usize"))?;
		reader
			.seek(SeekFrom::Start(block_offset))
			.context("failed to seek to chunk payload")?;
		let mut compressed_block = vec![0_u8; block_len];
		reader
			.read_exact(&mut compressed_block)
			.context("failed to read compressed chunk")?;

		ensure!(
			crc32(&compressed_block) == entry.block_crc32,
			"block crc32 mismatch for chunk {}",
			index
		);

		let decompressed = crate::decompress_phase2(&compressed_block)
			.ok_or_else(|| anyhow::anyhow!("phase2 decode failed for chunk {}", index))?;
		let original_size = usize::try_from(entry.original_size)
			.map_err(|_| anyhow::anyhow!("original size exceeds usize"))?;
		ensure!(
			decompressed.len() == original_size,
			"chunk {} original size mismatch",
			index
		);

		writer
			.write_all(&decompressed)
			.with_context(|| format!("failed to write {}", output_path.display()))?;
		bytes_written = bytes_written
			.checked_add(decompressed.len())
			.ok_or_else(|| anyhow::anyhow!("decoded size overflowed usize"))?;
	}

	ensure!(
		bytes_written == usize::try_from(header.original_size).map_err(|_| anyhow::anyhow!("archive original size exceeds usize"))?,
		"decoded archive size mismatch"
	);
	writer.flush().with_context(|| format!("failed to flush {}", output_path.display()))?;

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	fn create_temp_path() -> TemporaryFile {
		let (temp, file) = TemporaryFile::create().expect("temp file should be created");
		drop(file);
		temp
	}

	fn write_temp_input(bytes: &[u8]) -> TemporaryFile {
		let (temp, file) = TemporaryFile::create().expect("temp input should be created");
		let mut writer = BufWriter::new(file);
		writer.write_all(bytes).expect("input should be written");
		writer.flush().expect("input should flush");
		temp
	}

	#[test]
	fn streaming_file_io_roundtrips_multi_chunk_payloads() {
		let input_bytes = b"linxust-streaming-archive-".repeat(4096);
		let input = write_temp_input(&input_bytes);
		let archive = create_temp_path();
		let output = create_temp_path();

		write_archive_file(input.path(), archive.path(), 128).expect("archive should be written");
		read_archive_file(archive.path(), output.path(), 128).expect("archive should be read");

		let output_bytes = fs::read(output.path()).expect("output should be readable");
		assert_eq!(output_bytes, input_bytes);
	}

	#[test]
	fn streaming_file_io_handles_empty_inputs() {
		let input = write_temp_input(b"");
		let archive = create_temp_path();
		let output = create_temp_path();

		write_archive_file(input.path(), archive.path(), 64).expect("empty archive should be written");
		read_archive_file(archive.path(), output.path(), 64).expect("empty archive should be read");

		let output_bytes = fs::read(output.path()).expect("output should be readable");
		assert!(output_bytes.is_empty());
	}
}
