use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::io::BufReader;
use std::io::Read;
use thiserror::Error;
use std::path::{PathBuf, Path};

mod fmt;
pub mod nbt;
pub mod region;
pub mod world;

#[derive(Error, Debug)]
pub enum NbtFileError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error), // Automatically convert `io::Error` to `NbtReadError`

    #[error("Json could not be created")]
    JsonWriteFailure, // Custom error for content validation
}

#[derive(Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct NbtFile {
    file_path: String,
    nbtdata: nbt::NbtData,
}

impl NbtFile {
    pub fn new() -> Self {
        NbtFile::default()
    }

    pub fn read(file_path: String) -> Self {
        let buffer = Self::read_file(&file_path).unwrap();
        let nbtdata = nbt::NbtData::from_buf(buffer).unwrap();
        NbtFile { file_path, nbtdata }
    }

    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    pub fn nbt_tags(&self) -> &Vec<nbt::NbtTag> {
        &self.nbtdata.nbt_tags()
    }

    pub fn as_raw_bytes(&self) -> &Vec<u8> {
        &self.nbtdata.raw_bytes()
    }

    pub fn nbt_hashmap(&self) -> &HashMap<String, usize> {
        &self.nbtdata.tags_map()
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

    pub fn to_json(&self, output_path: &str) -> Result<(), NbtFileError> {
        let file = fs::File::create(output_path)?;
        serde_json::to_writer_pretty(file, self.nbtdata.nbt_tags())
            .map_err(|_| NbtFileError::JsonWriteFailure)?;
        Ok(())
    }
}
