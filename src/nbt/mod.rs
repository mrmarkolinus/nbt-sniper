use byteorder::{BigEndian, ReadBytesExt};
use core::{panic, str};
use std::io::Cursor;

use thiserror::Error;
use std::io;

mod fsm;

#[derive(Error, Debug)]
pub enum NbtReadError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),  // Automatically convert `io::Error` to `NbtReadError`
    
    #[error("Invalid NBT Tag Id")]
    InvalidContent,  // Custom error for content validation
}

#[derive(Debug, Copy, Clone)]
pub enum NbtTagId {
    End = 0,
    Byte = 1,
    Short = 2,
    Int = 3,
    Long = 4,
    Float = 5,
    Double = 6,
    ByteArray = 7,
    String = 8,
    List = 9,
    Compound = 10,
    IntArray = 11,
    LongArray = 12,
}

pub struct NbtTagSequence {
    tags: Vec<NbtTag>,
}


#[derive(Debug)]
pub struct NbtTag {
    name: String,
    value: NbtTagType,
    byte_start: u64,
    byte_end: u64,
    index: usize,
    depth: i64
}

impl NbtTag {
    pub fn value(&self) -> &NbtTagType {
        &self.value
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn byte_start(&self) -> u64 {
        self.byte_start
    }

    pub fn byte_end(&self) -> u64 {
        self.byte_end
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn depth(&self) -> i64 {
        self.depth
    }
}

#[derive(Debug)]
pub enum NbtTagType {
    End(Option<u8>),
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List((NbtTagId, i32)), //only store the name and the lenght of the list + the type of the elements in the list
    Compound(String), //only store the name of the compound
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}


impl NbtTagId {
    pub fn from_u8(value: u8) -> Option<NbtTagId> {
        match value {
            0 => Some(NbtTagId::End),
            1 => Some(NbtTagId::Byte),
            2 => Some(NbtTagId::Short),
            3 => Some(NbtTagId::Int),
            4 => Some(NbtTagId::Long),
            5 => Some(NbtTagId::Float),
            6 => Some(NbtTagId::Double),
            7 => Some(NbtTagId::ByteArray),
            8 => Some(NbtTagId::String),
            9 => Some(NbtTagId::List),
            10 => Some(NbtTagId::Compound),
            11 => Some(NbtTagId::IntArray),
            12 => Some(NbtTagId::LongArray),
            _ => None, // Return None if the value doesn't match any variant
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            NbtTagId::End => 0,
            NbtTagId::Byte => 1,
            NbtTagId::Short => 2,
            NbtTagId::Int => 3,
            NbtTagId::Long => 4,
            NbtTagId::Float => 5,
            NbtTagId::Double => 6,
            NbtTagId::ByteArray => 7,
            NbtTagId::String => 8,
            NbtTagId::List => 9,
            NbtTagId::Compound => 10,
            NbtTagId::IntArray => 11,
            NbtTagId::LongArray => 12,
        }
    }
}


impl NbtTagSequence {

    pub fn from_buf(cursor: &mut Cursor<Vec<u8>>) -> Result<NbtTagSequence, NbtReadError> {
        let mut nbt_parser = fsm::NbtParser::new(fsm::ParseNbtFsm::Normal, cursor.clone());
        let mut test_sequence = NbtTagSequence::new();
        fsm::parse(&mut test_sequence, &mut nbt_parser)?;
        
        Ok(test_sequence)
    }

    pub fn new() -> NbtTagSequence {
        NbtTagSequence { tags: Vec::<NbtTag>::new() } 
    }

    pub fn nbt_tags(&self) -> &Vec<NbtTag> {
        &self.tags
    }
}