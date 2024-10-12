use byteorder::{BigEndian, ReadBytesExt};
use core::{panic, str};
use std::io::{Cursor, Seek, SeekFrom};

use thiserror::Error;
use std::io;

mod fsm;

#[derive(Error, Debug)]
pub enum NbtReadError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),  // Automatically convert `io::Error` to `NbtReadError`
    
    #[error("Invalid NBT Tag Id")]
    InvalidContent,  // Custom error for content validation

    #[error("Invalid NBT Tree Depth")]
    InvalidNbtDepth,  // Custom error for tag id validation
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

#[derive(Debug)]
pub struct NbtTag {
    name: String,
    value: NbtTagType,
    byte_start: u64,
    byte_end: u64,
    index: usize,
    depth: i64,
    parent: usize,
    children: Vec<usize>,
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

    pub fn parent(&self) -> usize {
        self.parent
    }

}


pub struct NbtData <'a>{
    tags: Vec<NbtTag>,
    nbt_parser: fsm::NbtParser<'a>
}

impl<'a> NbtData<'a> {

    pub fn from_buf(cursor: &mut Cursor<Vec<u8>>) -> Result<NbtData, NbtReadError> {
        let mut nbt_parser = fsm::NbtParser::new(fsm::ParseNbtFsm::Normal, cursor);
        let mut nbt_data = NbtData::new(nbt_parser);
        //fsm::parse(&mut nbttag_sequence, &mut nbt_parser)?;
        
        Ok(nbt_data)
    }

    pub fn new(nbt_parser: fsm::NbtParser<'a>) -> NbtData {
        NbtData { 
            tags: Vec::<NbtTag>::new(),
            nbt_parser: nbt_parser,
        } 
    }

    pub fn nbt_tags(&self) -> &Vec<NbtTag> {
        &self.tags
    }

    fn parse(&mut self) -> Result<(), NbtReadError> {
        
        let cursor = &mut self.nbt_parser.cursor();
        let mut tag_id;
        let mut depth_delta= 0;
        let total_bytes = cursor.seek(SeekFrom::End(0)).unwrap();
        let mut nbt_parent_index = 0;
    
        let mut byte_start;   
        let mut tag_name;
        let mut tag_value;
    
        cursor.seek(SeekFrom::Start(0)).unwrap();
        
        loop {
            byte_start = cursor.position();   
            tag_name = String::new();
            tag_value = NbtTagType::End(None);
            
            self.nbt_parser.set_tree_depth(self.nbt_parser.tree_depth() + depth_delta);
            self.set_new_parent_index(depth_delta, &mut nbt_parent_index)?;
            depth_delta = 0;
    
            match self.nbt_parser.state() {
                fsm::ParseNbtFsm::Normal => {
                    //(tag_id, tag_name, tag_value, depth_delta) = parse_tag_id_name_and_value(test_sequence, nbt_parser, &mut unfinished_lists, nbt_parent_index)?;
                      tag_id = match fsm::parse::nbt_tag_id(cursor) {
                        Ok(id) => {
                            match id {
                                Some(id) => id,
                                None => return Err(NbtReadError::InvalidContent)
                            }
                        },
                        Err(e) => return Err(e)
                    }; 
    
                    if let NbtTagId::End = tag_id {
                        depth_delta = self.exit_nbttag_compound(nbt_parent_index);
                    }
                    else {
                        tag_name = fsm::parse::nbt_tag_string(cursor)?;    
                        tag_value = fsm::parse::nbt_tag(cursor, &tag_id)?;
    
                        if let NbtTagType::List(ref list_elem_tag_ids) = tag_value {
                            self.nbt_parser.list_parser.set_id(list_elem_tag_ids.0);
                            self.nbt_parser.list_parser.set_len(list_elem_tag_ids.1);
                            self.nbt_parser.change_state_to(fsm::ParseNbtFsm::List); 
                            depth_delta += 1;
                        }
    
                        if let NbtTagId::Compound = tag_id {
                            depth_delta += 1;
                        }
                    } 
                    
                },
    
                fsm::ParseNbtFsm::List => {
                    tag_id = *self.nbt_parser.list_parser.tag_id();
                    tag_name = "".to_string();
                    tag_value = fsm::parse::nbt_tag(cursor, &tag_id)?;
                    
                    if self.nbt_parser.list_parser.is_end() {   
                        self.nbt_parser.change_state_to(fsm::ParseNbtFsm::Normal); 
                        self.nbt_parser.list_parser.reset();
                        
                        if let NbtTagId::Compound = tag_id {
                            depth_delta += 1;
                            // if we are in a list of compound, we need to exist the list parser and go back to normal
                            // the list is finished, so we do not need to store the list parser status
                            self.nbt_parser.change_state_to(fsm::ParseNbtFsm::Normal);
                        }
                        else {
                            depth_delta -= 1 
                        }
                    }
                    else {
                        self.nbt_parser.list_parser.increment();  
    
                        if let NbtTagId::Compound = tag_id {
                            depth_delta += 1;
        
                            // if we are in a list of compound, we need to exist the list parser and go back to normal
                            // but we also need to store the point in the list were we are
                            self.nbt_parser.change_state_to(fsm::ParseNbtFsm::Normal);
                            self.store_list_ctx();
                        }
                    }
                },
    
                fsm::ParseNbtFsm::EndOfFile => {
                    break;
                },
            }
    
            let byte_end = cursor.position();
            
            let new_nbt_tag = NbtTag { name: tag_name, 
                                                    value: tag_value, 
                                                    byte_start: byte_start, 
                                                    byte_end: byte_end,
                                                    index: *self.nbt_parser.index(),
                                                    depth: *self.nbt_parser.tree_depth(),
                                                    parent: nbt_parent_index,
                                                    children: Vec::new()};  
            
            self.tags.push(new_nbt_tag);
    
            self.nbt_parser.increment_index();
            if byte_end >= total_bytes {
                self.nbt_parser.change_state_to(fsm::ParseNbtFsm::EndOfFile);
                break; //TODO Remove
            }
        }
    
        Ok(())
    }

    fn set_new_parent_index(&self, depth_delta: i64, nbt_parent_index: &mut usize) -> Result<(), NbtReadError> {

        match depth_delta {
            0 => {
                // nothing to do, old parent remains valid since we didnt go deeper
                },
            1 => {
                // we moved down in the nbt tree. This tag is the children of the tag in previous depth level
                *nbt_parent_index = self.nbt_parser.index() - 1; 
            },
            -1 => {
                //we moved up in the nbt tree. we need to restore the previous parent index
                //the new parent is the parent of the previous parent
                *nbt_parent_index = self.tags[*nbt_parent_index].parent(); 
            },
            -2 => {
                //we moved up in the nbt tree. we need to restore the previous parent index
                //this case is only hit when a list of compound is finished
                // -1 because we exit the compound
                // -1 because we exit the list
                let nbt_grandparent_index = self.tags[*nbt_parent_index].parent();
                *nbt_parent_index = self.tags[nbt_grandparent_index].parent(); 
            }
            _ => {
                //this should never happen, because delta_depth can only be -2, -1, 0, 1
                return Err(NbtReadError::InvalidNbtDepth)
            }
        }
        Ok(())
    }
    
    fn store_list_ctx(&mut self) {
        let unfinished_lists = &mut self.nbt_parser.unfinished_lists;
        unfinished_lists.push(self.nbt_parser.list_parser.clone());
        self.nbt_parser.list_parser.reset();
    }
    
    fn restore_list_ctx(&mut self) -> bool {
        
        let unfinished_lists = &mut self.nbt_parser.unfinished_lists;
        
        match unfinished_lists.pop() { 
            //the list of compounds was not yet finished, restore the ctx
            Some(previous_list_parser) => {
                self.nbt_parser.list_parser = previous_list_parser;
                true
            }
            // the list of compounds was finished and we do not need to restore the ctx
            // only in this case we will have a depth_delta of -2 because
            // compound is finished (=-1) and the list as well (=-1)
            None => false
        } 
    }
    
    fn exit_nbttag_compound(&mut self, nbt_parent_index: usize) -> i64{
        
        let mut depth_delta = -1;
    
        // the tag End is the last in a compound, so its parent is the compound
        // if the grandparent is a list, we need to change the state back to list
        // because reading a list is different than reading any other tag
        let nbt_grandparent_index = self.tags[nbt_parent_index].parent();
        let gp_nbt_tag = self.tags[nbt_grandparent_index].value();
        
        match gp_nbt_tag {
            NbtTagType::List(_) => {
                if self.restore_list_ctx() {
                    self.nbt_parser.change_state_to(fsm::ParseNbtFsm::List);
                }
                else {
                    // only in this case we will have a depth_delta of -2 because
                    // compound is finished (=-1) and the list as well (=-1)
                    depth_delta -= 1;
                }
                
            },
            _ => {
                //nothing to do 
            }
        }
        depth_delta
    }
    
    fn parse_tag_id_name_and_value(&mut self, nbt_parent_index: usize) -> Result<(NbtTagId, String, NbtTagType, i64), NbtReadError> {
        
        let mut tag_name = String::new();
        let mut tag_value = NbtTagType::End(None);
        let mut depth_delta = 0;
        //let mut cursor = nbt_parser.cursor();
        
        let tag_id = match fsm::parse::nbt_tag_id(&mut self.nbt_parser.cursor()) {
            Ok(id) => {
                match id {
                    Some(id) => id,
                    None => return Err(NbtReadError::InvalidContent)
                }
            },
            Err(e) => return Err(e)
        };
    
        /* if let nbt::NbtTagId::End = tag_id {
            depth_delta = exit_nbttag_compound(tag_sequence, unfinished_lists, nbt_parser, nbt_parent_index);
        }
        else {
            tag_name = parse::nbt_tag_string(cursor)?;    
            tag_value = parse::nbt_tag(cursor, &tag_id)?;
    
            if let nbt::NbtTagType::List(ref list_elem_tag_ids) = tag_value {
                nbt_parser.list_parser.set_id(list_elem_tag_ids.0);
                nbt_parser.list_parser.set_len(list_elem_tag_ids.1);
                nbt_parser.change_state_to(ParseNbtFsm::List); 
                depth_delta += 1;
            }
    
            if let nbt::NbtTagId::Compound = tag_id {
                depth_delta += 1;
            }
        } */
    
        Ok((tag_id, tag_name, tag_value, depth_delta))
    }
    
    
}