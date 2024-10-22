use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::io::BufReader;
use std::io::Read;
use thiserror::Error;

use crate::nbt;
use crate::NbtFileError;

#[derive(Error, Debug)]
pub enum RegionFileError {
    #[error("NbtFile error: {0}")]
    NbtFileError(#[from] NbtFileError),

    #[error("Json could not be created")]
    JsonWriteFailure, // Custom error for content validation

    #[error("Header is not exactly 4096 bytes long")]
    HeaderLengthError, // Custom error for content validation
}

const HEADER_LENGTH: usize = 4096;
const CHUNK_HEADER_LENGTH: usize = 4;
const CHUNK_HEADER_COMPRESSION: usize = CHUNK_HEADER_LENGTH + 1;
const CHUNK_SIZE_MULTIPLIER: u32 = 4096;
const CHUNK_OFFSET_BITSHIFT: u32 = 4;


pub struct RegionFile {
    file_path: String,
    num_chunks: u32,
    chunks: Vec<Chunk>,
}

impl RegionFile {
    pub fn new(file_path: String) -> RegionFile {
        RegionFile {
            file_path,
            num_chunks: 0,
            chunks: vec![Chunk::new()],
        }
    }

    pub fn read(file_path: String) -> Self {
        let buffer = Self::read_file(&file_path).unwrap();
        let region_chunks = match Self::read_header(&buffer) {
            Ok(chunks) => chunks,
            Err(e) => panic!("{}", e),
        };

        RegionFile {
            file_path,
            num_chunks: 0,
            chunks: region_chunks,
        }
    }

    fn read_header(region_content: &[u8]) -> Result<Vec<Chunk>, RegionFileError> {      
        if region_content.len() != HEADER_LENGTH {
            return Err(RegionFileError::HeaderLengthError);
        }

        let header_raw_bytes = &region_content[0..HEADER_LENGTH];
        let chunks: Vec<Chunk> = header_raw_bytes
            .chunks(4)
            .map(|chunk_info| Self::parse_chunk_position_and_size(chunk_info))
            .collect();

        Ok(chunks)
    }

    fn parse_chunk_position_and_size(chunk_info: &[u8]) -> Chunk {
        let offset = u32::from_be_bytes([chunk_info[0], chunk_info[1], chunk_info[2], 0]) << CHUNK_OFFSET_BITSHIFT;
        let size = u32::from(chunk_info[3]) * CHUNK_SIZE_MULTIPLIER;

        Chunk {
            offset,
            size,
            data: nbt::NbtData::new(Vec::new()),
        }
    }

    fn read_file(file_path: &str) -> std::io::Result<Vec<u8>> {
        // Open the file and create a buffered reader for efficient reading
        let file = fs::File::open(file_path)?;

        let buf_reader = BufReader::new(file);
        let mut decoder = GzDecoder::new(buf_reader);
        let mut decompressed_data = Vec::new();

        decoder.read_to_end(&mut decompressed_data)?;
        Ok(decompressed_data)
    }
}

pub struct Chunk {
    offset: u32,
    size: u32,
    data: nbt::NbtData,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            offset: 0,
            size: 0,
            data: nbt::NbtData::new(Vec::new()),
        }
    }

    pub fn offset(&self) -> u32 {
        self.offset
    }

    pub fn data(&self) -> &nbt::NbtData {
        &self.data
    }

    pub fn set_offset(&mut self, offset: u32) {
        self.offset = offset;
    }

    pub fn set_data(&mut self, data: nbt::NbtData) {
        self.data = data;
    }
}
