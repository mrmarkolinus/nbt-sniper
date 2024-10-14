use std::io::{Cursor, Seek, SeekFrom};
use thiserror::Error;
use std::io;
use serde::Serialize;
use serde_json;

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

#[derive(Debug, Copy, Clone, Serialize)]
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


#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
struct NbtTagPosition_RawBytes {
    byte_start_all: u64,
    byte_end_all: u64,
    byte_end_all_with_children: u64,
    byte_start_id: u64,
    byte_end_id: u64,
    byte_start_name: u64,
    byte_end_name: u64,
    byte_start_value: u64,
    byte_end_value: u64,
}

impl NbtTagPosition_RawBytes {
    pub fn new() -> NbtTagPosition_RawBytes {
        NbtTagPosition_RawBytes {
            byte_start_all: 0,
            byte_end_all: 0,
            byte_end_all_with_children: 0,
            byte_start_id: 0,
            byte_end_id: 0,
            byte_start_name: 0,
            byte_end_name: 0,
            byte_start_value: 0,
            byte_end_value: 0,
        }
    }

    pub fn reset(&mut self) {
        self.byte_start_all = 0;
        self.byte_end_all = 0;
        self.byte_end_all_with_children = 0;
        self.byte_start_id = 0;
        self.byte_end_id = 0;
        self.byte_start_name = 0;
        self.byte_end_name = 0;
        self.byte_start_value = 0;
        self.byte_end_value = 0;
    }

    pub fn byte_start_all(&self) -> u64 {
        self.byte_start_all
    }

    pub fn byte_end_all(&self) -> u64 {
        self.byte_end_all
    }

    pub fn byte_end_all_with_children(&self) -> u64 {
        self.byte_end_all_with_children
    }   

    pub fn byte_start_id(&self) -> u64 {
        self.byte_start_id
    }

    pub fn byte_end_id(&self) -> u64 {
        self.byte_end_id
    }

    pub fn byte_start_name(&self) -> u64 {
        self.byte_start_name
    }

    pub fn byte_end_name(&self) -> u64 {
        self.byte_end_name
    }

    pub fn byte_start_value(&self) -> u64 {
        self.byte_start_value
    }

    pub fn byte_end_value(&self) -> u64 {
        self.byte_end_value
    }

    pub fn set_byte_start_all(&mut self, byte_start_all: u64) {
        self.byte_start_all = byte_start_all;
    }

    pub fn set_byte_end_all(&mut self, byte_end_all: u64) {
        self.byte_end_all = byte_end_all;
    }

    pub fn set_byte_end_all_with_children(&mut self, byte_end_all_with_children: u64) {
        self.byte_end_all_with_children = byte_end_all_with_children;
    }

    pub fn set_byte_start_id(&mut self, byte_start_id: u64) {
        self.byte_start_id = byte_start_id;
    }

    pub fn set_byte_end_id(&mut self, byte_end_id: u64) {
        self.byte_end_id = byte_end_id;
    }

    pub fn set_byte_start_name(&mut self, byte_start_name: u64) {
        self.byte_start_name = byte_start_name;
    }

    pub fn set_byte_end_name(&mut self, byte_end_name: u64) {
        self.byte_end_name = byte_end_name;
    }

    pub fn set_byte_start_value(&mut self, byte_start_value: u64) {
        self.byte_start_value = byte_start_value;
    }

    pub fn set_byte_end_value(&mut self, byte_end_value: u64) {
        self.byte_end_value = byte_end_value;
    }

}


#[derive(Debug, Clone, Serialize)]
pub struct NbtTagPosition {
    raw_bytes: NbtTagPosition_RawBytes,
    index: usize,
    depth: i64,
    parent: usize,
    children: Vec<usize>,
} 

impl NbtTagPosition {
    pub fn new() -> NbtTagPosition {
        NbtTagPosition {
            raw_bytes: NbtTagPosition_RawBytes {    
                byte_start_all: 0,
                byte_end_all: 0,
                byte_end_all_with_children: 0,
                byte_start_id: 0,
                byte_end_id: 0,
                byte_start_name: 0,
                byte_end_name: 0,
                byte_start_value: 0,
                byte_end_value: 0,
            },
            index: 0,
            depth: 0,
            parent: 0,
            children: Vec::new(),
        } 
    }

    pub fn reset(&mut self) {
        self.raw_bytes.reset();
        self.index = 0;
        self.depth = 0;
        self.parent = 0;
        self.children = Vec::new();
    }
    
    pub fn byte_start_all(&self) -> u64 {
        self.raw_bytes.byte_end_all()
    }

    pub fn byte_end_all(&self) -> u64 {
        self.raw_bytes.byte_end_all()
    }

    pub fn byte_end_all_with_children(&self) -> u64 {
        self.raw_bytes.byte_end_all_with_children()
    }   

    pub fn byte_start_id(&self) -> u64 {
        self.raw_bytes.byte_start_id()
    }

    pub fn byte_end_id(&self) -> u64 {
        self.raw_bytes.byte_end_id()
    }

    pub fn byte_start_name(&self) -> u64 {
        self.raw_bytes.byte_start_name()
    }

    pub fn byte_end_name(&self) -> u64 {
        self.raw_bytes.byte_end_name()
    }

    pub fn byte_start_value(&self) -> u64 {
        self.raw_bytes.byte_start_value()
    }

    pub fn byte_end_value(&self) -> u64 {
        self.raw_bytes.byte_end_value()
    }

    pub fn set_byte_start_all(&mut self, byte_start_all: u64) {
        self.raw_bytes.set_byte_start_all(byte_start_all)
    }

    pub fn set_byte_end_all(&mut self, byte_end_all: u64) {
        self.raw_bytes.set_byte_end_all(byte_end_all)
    }

    pub fn set_byte_end_all_with_children(&mut self, byte_end_all_with_children: u64) {
        self.raw_bytes.set_byte_end_all_with_children(byte_end_all_with_children)
    }

    pub fn set_byte_start_id(&mut self, byte_start_id: u64) {
        self.raw_bytes.set_byte_start_id(byte_start_id)
    }

    pub fn set_byte_end_id(&mut self, byte_end_id: u64) {
        self.raw_bytes.set_byte_end_id(byte_end_id)
    }

    pub fn set_byte_start_name(&mut self, byte_start_name: u64) {
        self.raw_bytes.set_byte_start_name(byte_start_name)
    }

    pub fn set_byte_end_name(&mut self, byte_end_name: u64) {
        self.raw_bytes.set_byte_end_name(byte_end_name)
    }

    pub fn set_byte_start_value(&mut self, byte_start_value: u64) {
        self.raw_bytes.set_byte_start_value(byte_start_value)
    }

    pub fn set_byte_end_value(&mut self, byte_end_value: u64) {
        self.raw_bytes.set_byte_end_value(byte_end_value)
    }
    
    pub fn children(&mut self) -> &mut Vec<usize> {
        &mut self.children
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn set_index(&mut self, index: usize) {
        self.index = index;
    }   

    pub fn depth(&self) -> i64 {
        self.depth
    }

    pub fn set_depth(&mut self, depth: i64) {
        self.depth = depth;
    }

    pub fn parent(&self) -> usize {
        self.parent
    }

    pub fn set_parent(&mut self, parent: usize) {
        self.parent = parent;
    }

}

#[derive(Debug, Clone, Serialize)] 
pub struct NbtTag {
    name: String,
    value: NbtTagType,
    position: NbtTagPosition,
}

impl NbtTag {
    pub fn value(&self) -> &NbtTagType {
        &self.value
    }

    pub fn set_value(&mut self, value: NbtTagType) {
        self.value = value;
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn position(&self) -> &NbtTagPosition {
        &self.position
    }

    pub fn position_as_mut(&mut self) -> &mut NbtTagPosition {
        &mut self.position
    }

    pub fn set_position(&mut self, position: NbtTagPosition) {
        self.position = position;
    }

    pub fn children(&mut self) -> &mut Vec<usize> {
        &mut self.position.children
    }

    /* pub fn byte_start(&self) -> u64 {
        self.position.raw_bytes.byte_start_all
    }

    pub fn set_byte_start(&mut self, byte_start: u64) {
        self.position.raw_bytes.byte_start_all = byte_start;
    }

    pub fn byte_end(&self) -> u64 {
        self.position.raw_bytes.byte_end_all
    }

    pub fn set_byte_end(&mut self, byte_end: u64) {
        self.position.raw_bytes.byte_end_all = byte_end;
    }

    pub fn byte_end_with_children(&self) -> u64 {
        self.position.raw_bytes.byte_end_all_with_children
    }

    pub fn set_byte_end_with_children(&mut self, byte_end_with_children: u64) {
        self.position.raw_bytes.byte_end_all_with_children = byte_end_with_children;
    }

    pub fn index(&self) -> usize {
        self.position.index
    }

    pub fn set_index(&mut self, index: usize) {
        self.position.index = index;
    }   

    pub fn depth(&self) -> i64 {
        self.position.depth
    }

    pub fn set_depth(&mut self, depth: i64) {
        self.position.depth = depth;
    }

    pub fn parent(&self) -> usize {
        self.position.parent
    }

    pub fn set_parent(&mut self, parent: usize) {
        self.position.parent = parent;
    } */

}


pub struct NbtData {
    tags: Vec<NbtTag>,
    nbt_parser: fsm::NbtParser,
    raw_bytes: Vec<u8>,
}

impl NbtData {

    pub fn from_buf(file_buffer: Vec<u8>) -> Result<NbtData, NbtReadError> {

        let mut nbt_data = NbtData::new(file_buffer);
        nbt_data.parse()?;
        
        Ok(nbt_data)
    }

    pub fn new(file_buffer: Vec<u8>) -> NbtData {
        NbtData { 
            tags: Vec::<NbtTag>::new(),
            nbt_parser: fsm::NbtParser::new(fsm::ParseNbtFsm::Normal),
            raw_bytes: file_buffer
        }  
    }

    pub fn nbt_tags(&self) -> &Vec<NbtTag> {
        &self.tags
    }

    pub fn raw_bytes(&self) -> &Vec<u8> {
        &self.raw_bytes
    }

    fn parse(&mut self) -> Result<(), NbtReadError> {
        
        // #01 Initialize
        // #01 Initialize NbtTag content
        let mut tag_id;
        let mut new_tag_position = NbtTagPosition::new();
        let mut new_nbt_tag = NbtTag {  name: "".to_string(), 
                                                value: NbtTagType::End(None), 
                                                position: new_tag_position.clone()};  

        // #01 Initialize auxiliary information for parsing and building the NbtTag tree
        let mut cursor = Cursor::new(self.raw_bytes.clone());
        let total_bytes = cursor.seek(SeekFrom::End(0)).unwrap();
        let mut nbt_parent_index = 0;
        let mut depth_delta= 0;

    
        // #02 Parse the Nbt binary file and build the NbtTag tree
        cursor.seek(SeekFrom::Start(0)).unwrap();
        loop {

            // set the current Nbt tree depth and update the parent index (who is the parent of the processed NbtTag)
            self.nbt_parser.set_tree_depth(self.nbt_parser.tree_depth() + depth_delta);
            self.set_new_parent_index(depth_delta, &mut nbt_parent_index)?;
            depth_delta = 0;

            // reset the NbtTag information and start parsing a new NbtTag
            new_nbt_tag.set_name("".to_string());
            new_nbt_tag.set_value(NbtTagType::End(None));
            
            new_tag_position.set_byte_start(cursor.position());
            new_tag_position.set_byte_end(new_tag_position.byte_start());
            new_tag_position.set_index(0);
            new_tag_position.set_depth(0);
            new_tag_position.set_parent(0);
            

            match self.nbt_parser.state() {
                fsm::ParseNbtFsm::Normal => {
                    //(tag_id, tag_name, tag_value, depth_delta) = parse_tag_id_name_and_value(test_sequence, nbt_parser, &mut unfinished_lists, nbt_parent_index)?;
                    new_tag_position.set_byte_start_id(byte_start);
                    tag_id = match fsm::parse::nbt_tag_id(&mut cursor) {
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
                        new_nbt_tag.set_name(fsm::parse::nbt_tag_string(&mut cursor)?);    
                        new_nbt_tag.set_value(fsm::parse::nbt_tag(&mut cursor, &tag_id)?);
    
                        if let NbtTagType::List(ref list_elem_tag_ids) = new_nbt_tag.value() {
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
                    new_nbt_tag.set_name("".to_string());
                    new_nbt_tag.set_value(fsm::parse::nbt_tag(&mut cursor, &tag_id)?);
                    
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
    
            new_tag_position.set_byte_end(cursor.position());
            new_tag_position.set_index(*self.nbt_parser.index());
            new_tag_position.set_depth(*self.nbt_parser.tree_depth());
            new_tag_position.set_parent(nbt_parent_index);

            new_nbt_tag.set_position(new_tag_position.clone());
            self.tags.push(new_nbt_tag.clone());
            self.add_child_to_parent(&new_nbt_tag, nbt_parent_index);

            self.nbt_parser.increment_index();
            if new_nbt_tag.position().byte_end() >= total_bytes {
                self.nbt_parser.change_state_to(fsm::ParseNbtFsm::EndOfFile);
                break; //TODO Remove
            }
        }
    
        Ok(())
    }

    fn add_child_to_parent(&mut self, new_nbt_tag: &NbtTag, nbt_parent_index: usize) {
        
        let child_index = new_nbt_tag.position().index();
        let new_end_byte = new_nbt_tag.position().byte_end();
        
        self.tags[nbt_parent_index].position_as_mut().children().push(child_index);
        self.tags[nbt_parent_index].position_as_mut().set_byte_end_with_children(new_end_byte);
    }

    fn set_new_parent_index(&mut self, depth_delta: i64, nbt_parent_index: &mut usize) -> Result<(), NbtReadError> {

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
                *nbt_parent_index = self.tags[*nbt_parent_index].position().parent(); 
            },
            -2 => {
                //we moved up in the nbt tree. we need to restore the previous parent index
                //this case is only hit when a list of compound is finished
                // -1 because we exit the compound
                // -1 because we exit the list
                let nbt_grandparent_index = self.tags[*nbt_parent_index].position().parent();
                *nbt_parent_index = self.tags[nbt_grandparent_index].position().parent(); 
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
        let nbt_grandparent_index = self.tags[nbt_parent_index].position().parent();
        let gp_nbt_tag = self.tags[nbt_grandparent_index].value();
        
        match gp_nbt_tag {
            NbtTagType::List(_) => {
                if self.restore_list_ctx() {
                    self.nbt_parser.change_state_to(fsm::ParseNbtFsm::List);
                }
                else {
                    // only in this case we will have a depth_delta of -2 because
                    // compound is finished (=-1) and the list as well (=-1)
                    
                    //TOTO: i think there could be a bug here if we are the end of two lists
                    // example: list of compounds of list of compounds
                    depth_delta -= 1;
                }
                
            },
            _ => {
                //nothing to do 
            }
        }
        depth_delta
    }
    
    /* fn parse_tag_id_name_and_value(&mut self, nbt_parent_index: usize) -> Result<(NbtTagId, String, NbtTagType, i64), NbtReadError> {
        
        let mut tag_name = String::new();
        let mut tag_value = NbtTagType::End(None);
        let mut depth_delta = 0;
        let mut cursor = &mut self.raw_bytes;
        
        let tag_id = match fsm::parse::nbt_tag_id(cursor) {
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
     */
    
}