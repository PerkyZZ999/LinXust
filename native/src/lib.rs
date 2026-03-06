mod lxt;
mod lxt_streaming;

use napi_derive::napi;
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;

#[napi]
pub fn hello_from_rust(name: String) -> String {
  format!("Hello, {name}, from Rust!")
}

const LZ77_WINDOW_SIZE: usize = 4096;
const LZ77_MIN_MATCH: usize = 3;
const LZ77_MAX_MATCH: usize = 255;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Lz77Token {
  Literal(u8),
  Match { distance: u16, length: u8 },
}

#[derive(Default, Debug)]
struct BitWriter {
  bytes: Vec<u8>,
  current_byte: u8,
  bits_filled: u8,
}

impl BitWriter {
  fn write_bit(&mut self, bit: bool) {
    self.current_byte <<= 1;
    if bit {
      self.current_byte |= 1;
    }
    self.bits_filled += 1;
    if self.bits_filled == 8 {
      self.bytes.push(self.current_byte);
      self.current_byte = 0;
      self.bits_filled = 0;
    }
  }

  fn write_bits(&mut self, value: u32, count: u8) {
    for shift in (0..count).rev() {
      self.write_bit(((value >> shift) & 1) == 1);
    }
  }

  fn into_bytes(mut self) -> Vec<u8> {
    if self.bits_filled > 0 {
      self.current_byte <<= 8 - self.bits_filled;
      self.bytes.push(self.current_byte);
    }
    self.bytes
  }
}

struct BitReader<'a> {
  data: &'a [u8],
  byte_index: usize,
  bits_read: u8,
}

impl<'a> BitReader<'a> {
  fn new(data: &'a [u8]) -> Self {
    Self {
      data,
      byte_index: 0,
      bits_read: 0,
    }
  }

  fn read_bit(&mut self) -> Option<bool> {
    if self.byte_index >= self.data.len() {
      return None;
    }
    let byte = self.data[self.byte_index];
    let bit = ((byte >> (7 - self.bits_read)) & 1) == 1;
    self.bits_read += 1;
    if self.bits_read == 8 {
      self.bits_read = 0;
      self.byte_index += 1;
    }
    Some(bit)
  }

  fn read_bits(&mut self, count: u8) -> Option<u32> {
    let mut value = 0_u32;
    for _ in 0..count {
      value <<= 1;
      value |= self.read_bit()? as u32;
    }
    Some(value)
  }
}

#[derive(Clone, Debug)]
enum HuffmanNode {
  Leaf { symbol: u8 },
  Internal { left: usize, right: usize },
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct HeapEntry {
  freq: u32,
  index: usize,
}

impl Ord for HeapEntry {
  fn cmp(&self, other: &Self) -> Ordering {
    self
      .freq
      .cmp(&other.freq)
      .then_with(|| self.index.cmp(&other.index))
  }
}

impl PartialOrd for HeapEntry {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

fn lz77_encode(data: &[u8]) -> Vec<Lz77Token> {
  let mut tokens = Vec::new();
  let mut i = 0;

  while i < data.len() {
    let window_start = i.saturating_sub(LZ77_WINDOW_SIZE);
    let mut best_len = 0_usize;
    let mut best_distance = 0_usize;

    for j in window_start..i {
      let mut length = 0_usize;
      while i + length < data.len()
        && j + length < i
        && data[j + length] == data[i + length]
        && length < LZ77_MAX_MATCH
      {
        length += 1;
      }
      if length > best_len {
        best_len = length;
        best_distance = i - j;
      }
    }

    if best_len >= LZ77_MIN_MATCH {
      tokens.push(Lz77Token::Match {
        distance: best_distance as u16,
        length: best_len as u8,
      });
      i += best_len;
    } else {
      tokens.push(Lz77Token::Literal(data[i]));
      i += 1;
    }
  }

  tokens
}

fn lz77_decode(tokens: &[Lz77Token]) -> Option<Vec<u8>> {
  let mut output = Vec::new();
  for token in tokens {
    match token {
      Lz77Token::Literal(byte) => output.push(*byte),
      Lz77Token::Match { distance, length } => {
        let distance = *distance as usize;
        let length = *length as usize;
        if distance == 0 || distance > output.len() || length == 0 {
          return None;
        }
        for _ in 0..length {
          let source_index = output.len() - distance;
          output.push(output[source_index]);
        }
      }
    }
  }
  Some(output)
}

fn serialize_tokens(tokens: &[Lz77Token]) -> Vec<u8> {
  let mut bytes = Vec::new();
  for token in tokens {
    match token {
      Lz77Token::Literal(byte) => {
        bytes.push(0);
        bytes.push(*byte);
      }
      Lz77Token::Match { distance, length } => {
        bytes.push(1);
        bytes.extend_from_slice(&distance.to_le_bytes());
        bytes.push(*length);
      }
    }
  }
  bytes
}

fn deserialize_tokens(data: &[u8]) -> Option<Vec<Lz77Token>> {
  let mut tokens = Vec::new();
  let mut i = 0;
  while i < data.len() {
    let tag = data[i];
    i += 1;
    match tag {
      0 => {
        if i >= data.len() {
          return None;
        }
        tokens.push(Lz77Token::Literal(data[i]));
        i += 1;
      }
      1 => {
        if i + 2 >= data.len() {
          return None;
        }
        let distance = u16::from_le_bytes([data[i], data[i + 1]]);
        let length = data[i + 2];
        i += 3;
        tokens.push(Lz77Token::Match { distance, length });
      }
      _ => return None,
    }
  }
  Some(tokens)
}

fn build_huffman_tree(freqs: &[u32; 256]) -> Option<(Vec<HuffmanNode>, usize)> {
  let mut nodes = Vec::new();
  let mut heap = BinaryHeap::new();

  for (symbol, &freq) in freqs.iter().enumerate() {
    if freq > 0 {
      let index = nodes.len();
      nodes.push(HuffmanNode::Leaf { symbol: symbol as u8 });
      heap.push(Reverse(HeapEntry { freq, index }));
    }
  }

  if heap.is_empty() {
    return None;
  }

  while heap.len() > 1 {
    let Reverse(a) = heap.pop()?;
    let Reverse(b) = heap.pop()?;
    let freq = a.freq + b.freq;
    let index = nodes.len();
    nodes.push(HuffmanNode::Internal { left: a.index, right: b.index });
    heap.push(Reverse(HeapEntry { freq, index }));
  }

  let root = heap.pop()?.0.index;
  Some((nodes, root))
}

fn build_codes(
  nodes: &[HuffmanNode],
  index: usize,
  code: u32,
  bit_length: u8,
  codes: &mut [Option<(u32, u8)>; 256],
) {
  match nodes[index] {
    HuffmanNode::Leaf { symbol, .. } => {
      let code_length = if bit_length == 0 { 1 } else { bit_length };
      codes[symbol as usize] = Some((code, code_length));
    }
    HuffmanNode::Internal { left, right, .. } => {
      build_codes(nodes, left, code << 1, bit_length + 1, codes);
      build_codes(nodes, right, (code << 1) | 1, bit_length + 1, codes);
    }
  }
}

fn huffman_encode(data: &[u8]) -> Vec<u8> {
  let mut frequencies = [0_u32; 256];
  for &byte in data {
    frequencies[byte as usize] += 1;
  }

  let mut output = Vec::with_capacity(256 * 4 + 8 + data.len());
  for freq in frequencies {
    output.extend_from_slice(&freq.to_le_bytes());
  }
  output.extend_from_slice(&(data.len() as u64).to_le_bytes());

  if data.is_empty() {
    return output;
  }

  let Some((nodes, root)) = build_huffman_tree(&frequencies) else {
    return output;
  };

  let mut codes = [None; 256];
  build_codes(&nodes, root, 0, 0, &mut codes);

  let mut writer = BitWriter::default();
  for &byte in data {
    if let Some((code, bits)) = codes[byte as usize] {
      writer.write_bits(code, bits);
    }
  }
  output.extend_from_slice(&writer.into_bytes());
  output
}

fn huffman_decode(data: &[u8]) -> Option<Vec<u8>> {
  const HEADER_SIZE: usize = 256 * 4 + 8;
  if data.len() < HEADER_SIZE {
    return None;
  }

  let mut frequencies = [0_u32; 256];
  for (i, chunk) in data[..256 * 4].chunks_exact(4).enumerate() {
    frequencies[i] = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
  }

  let output_len = u64::from_le_bytes(data[256 * 4..HEADER_SIZE].try_into().ok()?) as usize;
  if output_len == 0 {
    return Some(Vec::new());
  }

  let (nodes, root) = build_huffman_tree(&frequencies)?;
  if let HuffmanNode::Leaf { symbol, .. } = nodes[root] {
    return Some(vec![symbol; output_len]);
  }

  let mut reader = BitReader::new(&data[HEADER_SIZE..]);
  let mut output = Vec::with_capacity(output_len);

  while output.len() < output_len {
    let mut current = root;
    loop {
      match nodes[current] {
        HuffmanNode::Leaf { symbol, .. } => {
          output.push(symbol);
          break;
        }
        HuffmanNode::Internal { left, right, .. } => {
          let bit = reader.read_bit()?;
          current = if bit { right } else { left };
        }
      }
    }
  }

  Some(output)
}

fn compress_phase2(data: &[u8]) -> Vec<u8> {
  let tokens = lz77_encode(data);
  let serialized = serialize_tokens(&tokens);
  huffman_encode(&serialized)
}

fn decompress_phase2(data: &[u8]) -> Option<Vec<u8>> {
  let decoded = huffman_decode(data)?;
  let tokens = deserialize_tokens(&decoded)?;
  lz77_decode(&tokens)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn bitstream_roundtrip() {
    let mut writer = BitWriter::default();
    writer.write_bits(0b101, 3);
    writer.write_bits(0b0110_1111, 8);
    writer.write_bits(0b1, 1);
    let bytes = writer.into_bytes();

    let mut reader = BitReader::new(&bytes);
    assert_eq!(reader.read_bits(3), Some(0b101));
    assert_eq!(reader.read_bits(8), Some(0b0110_1111));
    assert_eq!(reader.read_bits(1), Some(0b1));
  }

  #[test]
  fn lz77_roundtrip() {
    let input = b"abracadabra abracadabra abracadabra";
    let tokens = lz77_encode(input);
    let decoded = lz77_decode(&tokens).expect("lz77 decode should succeed");
    assert_eq!(decoded, input);
  }

  #[test]
  fn huffman_roundtrip() {
    let input = b"mississippi river";
    let encoded = huffman_encode(input);
    let decoded = huffman_decode(&encoded).expect("huffman decode should succeed");
    assert_eq!(decoded, input);
  }

  #[test]
  fn phase2_pipeline_roundtrip() {
    let input = b"linxust phase2 phase2 phase2 \0 \x01 \x01 \x01";
    let encoded = compress_phase2(input);
    let decoded = decompress_phase2(&encoded).expect("pipeline decode should succeed");
    assert_eq!(decoded, input);
  }

  #[test]
  fn phase2_pipeline_handles_empty_input() {
    let input = b"";
    let encoded = compress_phase2(input);
    let decoded = decompress_phase2(&encoded).expect("pipeline decode should succeed");
    assert_eq!(decoded, input);
  }
}
