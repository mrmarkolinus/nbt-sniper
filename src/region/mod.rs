use flate2::read::GzDecoder;
use flate2::read::ZlibDecoder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::io::BufReader;
use std::io::Read;
use thiserror::Error;

use crate::{nbt, NbtFileError};

#[derive(Error, Debug)]
pub enum RegionFileError {
    #[error("NbtFile error: {0}")]
    NbtFileError(#[from] NbtFileError),

    #[error("Json could not be created")]
    JsonWriteFailure, // Custom error for content validation

    #[error("Region Header is not exactly 4096 bytes long")]
    HeaderLengthError, // Custom error for content validation

    #[error("Chunk Header is less than 4 bytes long")]
    InvalidChunkHeaderLenght,

    #[error("Unsupported Compression Type")]
    UnsupportedCompressionType
}

const HEADER_LENGTH: usize = 4096;
const CHUNK_HEADER_LENGTH: usize = 4;
const CHUNK_HEADER_COMPRESSION: usize = CHUNK_HEADER_LENGTH + 1;
const CHUNK_PAYLOAD_START: usize = CHUNK_HEADER_LENGTH + 1;
const CHUNK_SIZE_MULTIPLIER: u32 = 4096;
const CHUNK_OFFSET_BITSHIFT: u32 = 4;

pub enum CompressionType {
    Uncompressed = 0,
    Gzip = 1,
    Zlib = 2,
}

impl CompressionType {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(CompressionType::Uncompressed),
            1 => Some(CompressionType::Gzip),
            2 => Some(CompressionType::Zlib),
            _ => None,
        }
    }

    fn to_u8(self) -> u8 {
        match self {
            CompressionType::Uncompressed => 0,
            CompressionType::Gzip => 1,
            CompressionType::Zlib => 2,
        }
    }
}

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

    pub fn chunks(&self) -> &Vec<Chunk> {
        &self.chunks
    }

    pub fn num_chunks(&self) -> u32 {
        self.num_chunks
    }

    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    pub fn read(file_path: String) -> Self {
        let buffer = Self::read_file(&file_path).unwrap();
        let mut region_chunks = match Self::read_header(&buffer) {
            Ok(chunks) => chunks,
            Err(e) => panic!("{}", e),
        };

        Self::read_chunks(&buffer, &mut region_chunks).unwrap();

        RegionFile {
            file_path,
            num_chunks: 0,
            chunks: region_chunks,
        }
    }

    fn read_chunks(buffer: &[u8], chunks: &mut Vec<Chunk>) -> Result<(), RegionFileError> {
        
        for (chunk_index, chunk) in chunks.iter_mut().enumerate() {
            
            if (chunk.offset() > 0) && (chunk.size() > 0) {
                let chunk_start_byte = chunk.offset() as usize;//(chunk.offset() * CHUNK_SIZE_MULTIPLIER) as usize;
                let chunk_end_byte = chunk_start_byte + chunk.size() as usize;
                let chunk_raw_bytes = &buffer[chunk_start_byte..chunk_end_byte];
                
                let chunk_uncompressed_raw_bytes = Self::decode_chunk(chunk_raw_bytes)?;
                
                match nbt::NbtData::from_buf(chunk_uncompressed_raw_bytes) {
                    Ok(nbt_data) => {
                        chunk.set_data(nbt_data);
                    }
                    Err(e) => {
                        return Err(RegionFileError::UnsupportedCompressionType); //TODO CHANGE ERROR
                    }
                }
                
            }
            
        }

        Ok(())
    }

    fn decode_chunk(chunk_raw_bytes: &[u8]) -> Result<Vec<u8>, RegionFileError> {
        
        if chunk_raw_bytes.len() < CHUNK_HEADER_LENGTH {
            return Err(RegionFileError::InvalidChunkHeaderLenght);
        }
        
            let chunk_length = u32::from_be_bytes([chunk_raw_bytes[0], chunk_raw_bytes[1], chunk_raw_bytes[2], chunk_raw_bytes[3]]);
            
            let chunk_compression_method = &chunk_raw_bytes[CHUNK_HEADER_COMPRESSION];
            let compressed_chunk_payload = &chunk_raw_bytes[CHUNK_PAYLOAD_START..CHUNK_PAYLOAD_START + chunk_length as usize];

            let uncompressed_chunk_payload = Self::unzip_chunk(compressed_chunk_payload, chunk_compression_method.clone())?;

            Ok(uncompressed_chunk_payload)
            
    }

    pub fn unzip_chunk( chunk_payload: &[u8], chunk_compression_method: u8) -> Result<Vec<u8>, RegionFileError> {
        
        let mut chunk_decompressed_payload = Vec::new();

        // Decompress chunk data
        // acoording to minecraft wiki case Gzip and not compressed are not used in practice
        // but they are officially supported
        match CompressionType::from_u8(chunk_compression_method) {
            Some(CompressionType::Gzip) => {
                // Gzip compression
                let mut decoder = GzDecoder::new(chunk_payload);
                decoder.read_to_end(&mut chunk_decompressed_payload).map_err( |_| RegionFileError::UnsupportedCompressionType )?;
            },
            Some(CompressionType::Zlib) => { 
                // Zlib compression
                let mut decoder = ZlibDecoder::new(chunk_payload);
                decoder.read_to_end(&mut chunk_decompressed_payload).map_err( |_| RegionFileError::UnsupportedCompressionType )?;
            },
            Some(CompressionType::Uncompressed) => {
                // Data is uncompressed
                chunk_decompressed_payload = chunk_payload.to_vec();
            },
            _ => return Err(RegionFileError::UnsupportedCompressionType),
        }

        Ok(chunk_decompressed_payload)
    }

    fn read_header(region_content: &[u8]) -> Result<Vec<Chunk>, RegionFileError> {      
        if region_content.len() < HEADER_LENGTH {
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

        let mut buf_reader = BufReader::new(file);
        let mut raw_data = Vec::new();
        buf_reader.read_to_end(&mut raw_data)?;
        Ok(raw_data)
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

    pub fn size(&self) -> u32 {
        self.size
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
