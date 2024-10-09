use byteorder::{BigEndian, ReadBytesExt};
use core::{panic, str};
use std::io::{Cursor, Seek, SeekFrom};

use thiserror::Error;
use std::io;

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

enum ParseNbtFsm {
    Normal,
    End,
    List
}

struct NbtListParser {
    list_tag_id: NbtTagId,
    list_len: i32,
    list_elem_count: i32,
}

impl NbtListParser {
    pub fn new() -> NbtListParser {
        NbtListParser { list_tag_id: NbtTagId::End, list_len: 0, list_elem_count: 0 }
    }
}

struct NbtParser {
    state: ParseNbtFsm,
    list_parser: NbtListParser
}

impl NbtParser {
    pub fn new(state: ParseNbtFsm) -> NbtParser {
        NbtParser { state: state, 
                    list_parser: NbtListParser::new() }
    }

    pub fn change_state_to(&mut self, state: ParseNbtFsm) {
        self.state = state;
    }

    pub fn state(&self) -> &ParseNbtFsm {
        &self.state
    }
}

impl NbtTagSequence {
    pub fn new() -> NbtTagSequence {
        NbtTagSequence { tags: Vec::<NbtTag>::new() } 
    }

    pub fn nbt_tags(&self) -> &Vec<NbtTag> {
        &self.tags
    }
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

    pub fn parse_from_buf(cursor: &mut Cursor<Vec<u8>>) -> Result<NbtTagSequence, NbtReadError> {
    
/*         let (nbt_root_present, 
            tag_id, 
            tag_name) = NbtTag::nbt_root_present(cursor);

        if nbt_root_present == false {
            return NbtTag { name: tag_name, value: NbtTagType::End(None) };
        }
        else {
            let mut compound_values = Vec::<NbtTag>::new();
            let compound = NbtTag::traverse_nbt_tree(cursor);

            compound_values.push(compound);
            NbtTag { name: tag_name, value: NbtTagType::Compound(compound_values) }
        } */

        let mut nbt_index = 0;

        /* let nbt_tag = match NbtTag::traverse_nbt_tree(cursor, &mut nbt_index) {
            Ok(nbt_tag) => nbt_tag,
            Err(e) => return Err(e),
        };      
 */     let mut nbt_parser = NbtParser::new(ParseNbtFsm::Normal);
        let mut test_sequence = NbtTagSequence::new();
        NbtTag::traverse_nbt_tree(cursor, &mut test_sequence, &mut nbt_parser, &mut nbt_index);
        
        Ok(test_sequence)
    }

    fn traverse_nbt_tree(cursor: &mut Cursor<Vec<u8>>, test_sequence : &mut NbtTagSequence, nbt_parser: &mut NbtParser, nbt_index: &mut usize) { //-> Result<NbtReadError> {
        
        let mut finished_reading = false;

        let mut list_tag_id = NbtTagId::End;
        let mut list_len = 0;
        let mut list_elem_count = 0;
        let mut reading_list = false;

        

        let mut tag_id;

        let total_bytes = cursor.seek(SeekFrom::End(0)).unwrap();
        cursor.seek(SeekFrom::Start(0)).unwrap();

        while finished_reading == false {

            let byte_start = cursor.position();
            
            let mut tag_name = String::new();
            let mut tag_value = NbtTagType::End(None);

            if let ParseNbtFsm::List = nbt_parser.state() {
                //a list is a sequence of NBT Tags without names and tags id.
                
                if list_elem_count < list_len - 1 {
                    list_elem_count = list_elem_count + 1;
                }
                else {
                    list_elem_count = 0;
                    list_len = 0;
                    //reading_list = false;
                    nbt_parser.change_state_to(ParseNbtFsm::Normal); 
                }

                tag_id = list_tag_id;
                tag_name = "".to_string();
            }
            else {
                tag_id = NbtTag::parse_nbt_tag_id(cursor).unwrap();
                if let NbtTagId::End = tag_id {
                    // nothing to do
                }
                else {
                    tag_name = NbtTag::parse_nbt_tag_string(cursor).unwrap();
                }
            }

            if let NbtTagId::End = tag_id {
                // nothing to do
            }
            else {
                tag_value = NbtTag::parse_nbt_tag(cursor, &tag_id, nbt_index).unwrap();

                if let NbtTagType::List(ref list_elem_tag_ids) = tag_value {
                    list_tag_id = list_elem_tag_ids.0;  
                    list_len = list_elem_tag_ids.1; 
                    //reading_list = true;
                    nbt_parser.change_state_to(ParseNbtFsm::List); 
                }
            }
            /* println!("-------------------------------");
            println!("Tag ID: {:?}", tag_id);
            println!("Tag Name: {:?}", tag_name);
            println!("Tag Value: {:?}", tag_value); */


            let byte_end = cursor.position();

            let current_index = *nbt_index;
            *nbt_index = current_index + 1;

            test_sequence.tags.push(NbtTag { name: tag_name, 
                                            value: tag_value, 
                                            byte_start: byte_start, 
                                            byte_end: byte_end,
                                            index: current_index });

            if byte_end >= total_bytes {
                finished_reading = true;
            }
        }
            
    }

    

/*     fn nbt_root_present(cursor: &mut Cursor<Vec<u8>>) -> (bool, Option<NbtTagId>, String) {
        
        let tag_id = match NbtTag::parse_nbt_tag_id(cursor) {
            None => return (false, None, "".to_string()),
            Some(tag_id) => tag_id,
        };

        let tag_name = match NbtTag::parse_nbt_tag_string(cursor) {
            Ok(tag_name) => tag_name,
            Err(e) => return (false, None, e),
        };

        match tag_id {
            NbtTagId::Compound => (true, Some(tag_id), tag_name),
            _ => (false, Some(tag_id), tag_name),
        }
    } */

    fn parse_nbt_tag_id(cursor: &mut Cursor<Vec<u8>>) -> Option<NbtTagId> {
        let id = cursor.read_u8().expect("Error reading byte from cursor");
        
        match NbtTagId::from_u8(id) {
            None => None,
            Some(id) => Some(id),
        }
    }

    fn parse_nbt_tag_string(cursor: &mut Cursor<Vec<u8>>) -> Result<String, NbtReadError> {
        let name_len = cursor.read_i16::<BigEndian>().unwrap();
        let mut name = String::with_capacity(name_len as usize);
    
        for _ in 0..name_len {
            let ch = match cursor.read_u8() {
                Ok(ch) => ch,
                Err(e) => return Err(NbtReadError::Io(e)),
            };
            name.push(ch as char)
        }
        
        Ok(name)
    }

    /* fn parse_nbt_tag_list(cursor: &mut Cursor<Vec<u8>>, nbt_index: &mut usize) -> Result<Vec<NbtTag>, NbtReadError> {

        let list_tag_id = match NbtTag::parse_nbt_tag_id(cursor) {
            None => return Err(NbtReadError::InvalidContent),
            Some(list_tag_id) => list_tag_id,
        };

        let list_len = match cursor.read_i32::<BigEndian>() {
            Ok(x) => x,
            Err(e) => return Err(NbtReadError::Io(e)),
        };
        
        if list_len > 65_536 {
            //TODO error handling
        }

        let mut list = Vec::with_capacity(list_len as usize);
        for _ in 0..list_len {
            
            let byte_start = cursor.position();
            let nbt_list_element = match NbtTag::parse_nbt_tag(cursor, &list_tag_id, nbt_index) {
                Ok(nbt_list_element) => nbt_list_element,
                Err(e) => return Err(e),   
            };
            let byte_end = cursor.position();
            let current_index = *nbt_index;
            *nbt_index = current_index + 1;

            list.push(NbtTag {  name: "".to_string(), 
                                value: nbt_list_element, 
                                byte_start: byte_start, 
                                byte_end: byte_end,
                                index: current_index });
        }

        Ok(list)

    } */


/*     fn parse_nbt_tag_compound(cursor: &mut Cursor<Vec<u8>>, nbt_index: &mut usize) -> Result<Vec<NbtTag>, NbtReadError> {
        
        let mut compound_values = Vec::<NbtTag>::new();
        let mut compound_completely_read = false;

        while compound_completely_read == false {
            let compound_child = NbtTag::traverse_nbt_tree(cursor, nbt_index);
            
            match compound_child { 
                Ok(compound_child) => {
                    match compound_child.value {
                        NbtTagType::End(None) => compound_completely_read = true,
                        _   => compound_completely_read = false 
                    } 
                
                    compound_values.push(compound_child);
                },
                Err(e) => return Err(e),    
            }
            
        }   
        
        Ok(compound_values)
    } */

    fn parse_nbt_tag(cursor: &mut Cursor<Vec<u8>>, tag_id: &NbtTagId, nbt_index: &mut usize) -> Result<NbtTagType, NbtReadError> {
        let tag_value = match tag_id {
            NbtTagId::End => NbtTagType::End(None),
            
            NbtTagId::Byte => {        
                let raw_tag_value = match cursor.read_i8() {
                    Ok(x) => x,
                    Err(e) => return Err(NbtReadError::Io(e)),
                };
                NbtTagType::Byte(raw_tag_value)
            },

            NbtTagId::Short => {
                let raw_tag_value = match cursor.read_i16::<BigEndian>() {
                    Ok(x) => x,
                    Err(e) => return Err(NbtReadError::Io(e)),
                };
                NbtTagType::Short(raw_tag_value)
            },

            NbtTagId::Int => {
                let raw_tag_value = match cursor.read_i32::<BigEndian>() {
                    Ok(x) => x,
                    Err(e) => return Err(NbtReadError::Io(e)),
                };
                NbtTagType::Int(raw_tag_value)
            },

            NbtTagId::Long => {
                let raw_tag_value = match cursor.read_i64::<BigEndian>() {
                    Ok(x) => x,
                    Err(e) => return Err(NbtReadError::Io(e)),
                };
                NbtTagType::Long(raw_tag_value)
            },

            NbtTagId::Float => {
                let raw_tag_value = match cursor.read_f32::<BigEndian>() {
                    Ok(x) => x,
                    Err(e) => return Err(NbtReadError::Io(e)),
                };
                NbtTagType::Float(raw_tag_value)
            },

            NbtTagId::Double => {
                let raw_tag_value = match cursor.read_f64::<BigEndian>() {
                    Ok(x) => x,
                    Err(e) => return Err(NbtReadError::Io(e)),
                };
                NbtTagType::Double(raw_tag_value)
            },
            
            NbtTagId::ByteArray => {
                let len = match cursor.read_i32::<BigEndian>() {
                    Ok(x) => x,
                    Err(e) => return Err(NbtReadError::Io(e)),
                };

                if len > 65_536 {
                    //TODO error handling
                }

                let mut buf = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let x = cursor.read_i8().unwrap();
                    buf.push(x);
                }

                NbtTagType::ByteArray(buf)
            },

            NbtTagId::String => {
                let raw_tag_value = NbtTag::parse_nbt_tag_string(cursor);
                
                match raw_tag_value {
                    Ok(value) => NbtTagType::String(value),
                    Err(e) => return Err(e),     
                }
            },

            NbtTagId::List => {
                /* let raw_tag_value = match NbtTag::parse_nbt_tag_string(cursor) {
                    //here we only store the nbt name and the type of the elements in the list, we parse the list in the next iterations
                    Ok(x) => x,
                    Err(e) => return Err(e),
                }; */
                let list_elem_tag_ids =  match NbtTag::parse_nbt_tag_id(cursor) {
                    None => return Err(NbtReadError::InvalidContent),
                    Some(list_elem_tag_ids) => list_elem_tag_ids,
                };
                let len = cursor.read_i32::<BigEndian>().unwrap();
                if len > 65_536 {
                    //TODO error handling
                    panic!("List length is too large");
                }
                NbtTagType::List((list_elem_tag_ids, len))
            },
            
            NbtTagId::Compound => {
               /*  let compound_values = match NbtTag::parse_nbt_tag_string(cursor) {
                    //here we only store the nbt name, we parse the compound in the next iterations
                    Ok(values) => values,
                    Err(e) => return Err(e),
                }; */
                NbtTagType::Compound("".to_string())
            },
            
            NbtTagId::IntArray => {
                let len = cursor.read_i32::<BigEndian>().unwrap();
                if len > 65_536 {
                    //TODO error handling
                    panic!("Array length is too large");
                }

                let mut buf = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let x = cursor.read_i32::<BigEndian>().unwrap();
                    buf.push(x);
                }

                NbtTagType::IntArray(buf)
            },
            NbtTagId::LongArray => {
                let len = cursor.read_i32::<BigEndian>().unwrap();
                if len > 65_536 {
                    //TODO error handling
                    panic!("Array length is too large");
                }

                let mut buf = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let x = cursor.read_i64::<BigEndian>().unwrap();
                    buf.push(x);
                }

                NbtTagType::LongArray(buf)
            }
        };

        Ok(tag_value)
    }


}