use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::io::{Cursor, Seek, SeekFrom};
use thiserror::Error;

mod fsm;

const MAX_LIST_LENGTH: i32 = 32767;
const MAX_BYTE_ARRAY_LENGTH: i32 = 32767;
const MAX_INT_ARRAY_LENGTH: i32 = 32767;
const MAX_LONG_ARRAY_LENGTH: i32 = 32767;

#[cfg(test)]
mod tests;

#[derive(Error, Debug)]
pub enum NbtReadError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error), // Automatically convert `io::Error` to `NbtReadError`

    #[error("Invalid NBT Tag Id")]
    InvalidContent, // Custom error for content validation

    #[error("Invalid NBT Tree Depth")]
    InvalidNbtDepth, // Custom error for tag id validation

    #[error("Invalid NBT List lenght")]
    InvalidNbtListLenght, // if list is longer than MAX_LIST_LENGTH

    #[error("NBT List longer than declared")]
    NbtListLongerThanDeclared, // if list is longer than the size read in the NBT file

    #[error("Invalid NBT ByteArray lenght")]
    InvalidNbtByteArrayLenght, // if array is longer than MAX_BYTE_ARRAY_LENGTH

    #[error("Invalid NBT IntArray lenght")]
    InvalidNbtIntArrayLenght, // if array is longer than MAX_BYTE_ARRAY_LENGTH

    #[error("Invalid NBT LongArray lenght")]
    InvalidNbtLongArrayLenght, // if array is longer than MAX_BYTE_ARRAY_LENGTH

    #[error("Invalid NBT Root Tag Id: NbtFile must start with a compound tag")]
    InvalidNbtRootTagId, // if root tag is not a compound

    #[error("Negative NBT Tag Length")]
    NegativeNbtTagLenght, // if tag length is negative

    #[error("NBT file is empty")]
    EmptyFile, // if file is empty
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
pub enum NbtTagId {
    #[default]
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
    pub fn new() -> NbtTagId {
        NbtTagId::default()
    }

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

    pub fn into_u8(&self) -> u8 {
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

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
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
    Compound(String),      //only store the name of the compound
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl Default for NbtTagType {
    fn default() -> Self {
        NbtTagType::End(None)
    }
}

impl NbtTagType {
    pub fn into_id(&self) -> NbtTagId {
        match self {
            NbtTagType::End(_) => NbtTagId::End,
            NbtTagType::Byte(_) => NbtTagId::Byte,
            NbtTagType::Short(_) => NbtTagId::Short,
            NbtTagType::Int(_) => NbtTagId::Int,
            NbtTagType::Long(_) => NbtTagId::Long,
            NbtTagType::Float(_) => NbtTagId::Float,
            NbtTagType::Double(_) => NbtTagId::Double,
            NbtTagType::ByteArray(_) => NbtTagId::ByteArray,
            NbtTagType::String(_) => NbtTagId::String,
            NbtTagType::List(_) => NbtTagId::List,
            NbtTagType::Compound(_) => NbtTagId::Compound,
            NbtTagType::IntArray(_) => NbtTagId::IntArray,
            NbtTagType::LongArray(_) => NbtTagId::LongArray,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default, Serialize, Deserialize)]
struct NbtTagPositionRawBytes {
    byte_start_all: usize,
    byte_end_all: usize,
    byte_end_all_with_children: usize,
    byte_start_id: Option<usize>,
    byte_end_id: Option<usize>,
    byte_start_name: Option<usize>,
    byte_end_name: Option<usize>,
    byte_start_value: Option<usize>,
    byte_end_value: Option<usize>,
}

impl NbtTagPositionRawBytes {
    pub fn new() -> NbtTagPositionRawBytes {
        NbtTagPositionRawBytes::default()
    }

    pub fn reset(&mut self) {
        self.byte_start_all = 0;
        self.byte_end_all = 0;
        self.byte_end_all_with_children = 0;
        self.byte_start_id = None;
        self.byte_end_id = None;
        self.byte_start_name = None;
        self.byte_end_name = None;
        self.byte_start_value = None;
        self.byte_end_value = None;
    }

    pub fn byte_start_all(&self) -> usize {
        self.byte_start_all
    }

    pub fn byte_end_all(&self) -> usize {
        self.byte_end_all
    }

    pub fn byte_end_all_with_children(&self) -> usize {
        self.byte_end_all_with_children
    }

    pub fn byte_start_id(&self) -> Option<usize> {
        self.byte_start_id
    }

    pub fn byte_end_id(&self) -> Option<usize> {
        self.byte_end_id
    }

    pub fn byte_start_name(&self) -> Option<usize> {
        self.byte_start_name
    }

    pub fn byte_end_name(&self) -> Option<usize> {
        self.byte_end_name
    }

    pub fn byte_start_value(&self) -> Option<usize> {
        self.byte_start_value
    }

    pub fn byte_end_value(&self) -> Option<usize> {
        self.byte_end_value
    }

    pub fn set_byte_start_all(&mut self, byte_start_all: usize) {
        self.byte_start_all = byte_start_all;
    }

    pub fn set_byte_end_all(&mut self, byte_end_all: usize) {
        self.byte_end_all = byte_end_all;
    }

    pub fn set_byte_end_all_with_children(&mut self, byte_end_all_with_children: usize) {
        self.byte_end_all_with_children = byte_end_all_with_children;
    }

    pub fn set_byte_start_id(&mut self, byte_start_id: usize) {
        self.byte_start_id = Some(byte_start_id);
    }

    pub fn set_byte_end_id(&mut self, byte_end_id: usize) {
        self.byte_end_id = Some(byte_end_id);
    }

    pub fn set_byte_start_name(&mut self, byte_start_name: usize) {
        self.byte_start_name = Some(byte_start_name);
    }

    pub fn set_byte_end_name(&mut self, byte_end_name: usize) {
        self.byte_end_name = Some(byte_end_name);
    }

    pub fn set_byte_start_value(&mut self, byte_start_value: usize) {
        self.byte_start_value = Some(byte_start_value);
    }

    pub fn set_byte_end_value(&mut self, byte_end_value: usize) {
        self.byte_end_value = Some(byte_end_value);
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, Serialize, Deserialize)]
pub struct NbtTagPosition {
    raw_bytes: NbtTagPositionRawBytes,
    index: usize,
    depth: i64,
    parent: usize,
    children: Vec<usize>,
}

impl NbtTagPosition {
    pub fn new() -> NbtTagPosition {
        NbtTagPosition {
            raw_bytes: NbtTagPositionRawBytes::new(),
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

    pub fn byte_start_all(&self) -> usize {
        self.raw_bytes.byte_start_all()
    }

    pub fn byte_end_all(&self) -> usize {
        self.raw_bytes.byte_end_all()
    }

    pub fn byte_end_all_with_children(&self) -> usize {
        self.raw_bytes.byte_end_all_with_children()
    }

    pub fn byte_start_id(&self) -> Option<usize> {
        self.raw_bytes.byte_start_id()
    }

    pub fn byte_end_id(&self) -> Option<usize> {
        self.raw_bytes.byte_end_id()
    }

    pub fn byte_start_name(&self) -> Option<usize> {
        self.raw_bytes.byte_start_name()
    }

    pub fn byte_end_name(&self) -> Option<usize> {
        self.raw_bytes.byte_end_name()
    }

    pub fn byte_start_value(&self) -> Option<usize> {
        self.raw_bytes.byte_start_value()
    }

    pub fn byte_end_value(&self) -> Option<usize> {
        self.raw_bytes.byte_end_value()
    }

    pub fn set_byte_start_all(&mut self, byte_start_all: usize) {
        self.raw_bytes.set_byte_start_all(byte_start_all)
    }

    pub fn set_byte_end_all(&mut self, byte_end_all: usize) {
        self.raw_bytes.set_byte_end_all(byte_end_all)
    }

    pub fn set_byte_end_all_with_children(&mut self, byte_end_all_with_children: usize) {
        self.raw_bytes
            .set_byte_end_all_with_children(byte_end_all_with_children)
    }

    pub fn set_byte_start_id(&mut self, byte_start_id: usize) {
        self.raw_bytes.set_byte_start_id(byte_start_id)
    }

    pub fn set_byte_end_id(&mut self, byte_end_id: usize) {
        self.raw_bytes.set_byte_end_id(byte_end_id)
    }

    pub fn set_byte_start_name(&mut self, byte_start_name: usize) {
        self.raw_bytes.set_byte_start_name(byte_start_name)
    }

    pub fn set_byte_end_name(&mut self, byte_end_name: usize) {
        self.raw_bytes.set_byte_end_name(byte_end_name)
    }

    pub fn set_byte_start_value(&mut self, byte_start_value: usize) {
        self.raw_bytes.set_byte_start_value(byte_start_value)
    }

    pub fn set_byte_end_value(&mut self, byte_end_value: usize) {
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

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct NbtTag {
    name: String,
    value: NbtTagType,
    position: NbtTagPosition,
}

impl NbtTag {
    pub fn new() -> NbtTag {
        NbtTag::default()
    }

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
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct NbtData {
    tags: Vec<NbtTag>,
    nbt_parser: fsm::NbtParser,
    raw_bytes: Vec<u8>,
    tags_map: HashMap<String, usize>,
}

impl NbtData {
    pub fn from_buf(file_buffer: Vec<u8>) -> Result<NbtData, NbtReadError> {
        if file_buffer.is_empty() {
            return Err(NbtReadError::EmptyFile);
        }
        let mut nbt_data = NbtData::new(file_buffer);
        nbt_data.parse()?;

        Ok(nbt_data)
    }

    pub fn new(file_buffer: Vec<u8>) -> NbtData {
        NbtData {
            tags: Vec::<NbtTag>::new(),
            nbt_parser: fsm::NbtParser::new(),
            raw_bytes: file_buffer,
            tags_map: HashMap::new(),
        }
    }

    pub fn nbt_tags(&self) -> &Vec<NbtTag> {
        &self.tags
    }

    pub fn raw_bytes(&self) -> &Vec<u8> {
        &self.raw_bytes
    }

    pub fn tags_map(&self) -> &HashMap<String, usize> {
        &self.tags_map
    }

    pub fn parse(&mut self) -> Result<(), NbtReadError> {
        // #1 Initialize
        // #11 Initialize NbtTag content
        let mut new_tag_position = NbtTagPosition::new();
        let mut new_nbt_tag = NbtTag {
            name: "".to_string(),
            value: NbtTagType::End(None),
            position: new_tag_position.clone(),
        };

        // #12 Initialize auxiliary information for parsing and building the NbtTag tree
        let mut cursor = Cursor::new(self.raw_bytes.clone());
        let total_bytes = cursor.seek(SeekFrom::End(0)).unwrap() as usize;
        let mut nbt_parent_index = 0;
        let mut depth_delta = 0;

        cursor.seek(SeekFrom::Start(0)).unwrap();
        loop {
            // #2 Reinit the loop for a new NbtTag
            // #21 set the current Nbt tree depth and update the parent index (who is the parent of the processed NbtTag)
            nbt_parent_index = self.update_nbttree_depth(nbt_parent_index, depth_delta)?;

            // #22 reset the NbtTag information and start parsing a new NbtTag
            new_nbt_tag.set_name("".to_string());
            new_nbt_tag.set_value(NbtTagType::End(None));
            self.init_tag_position(&mut new_tag_position, &mut cursor);

            // #3 Start parsing a new NbtTag
            match self.nbt_parser.state() {
                // #31  ParseFSM is in NbtRoot state: the first NbtTag of a NbtFile must always be a Root Compound Tag
                //      in this state we parse a normal tag, but we exit if the root is not a compound
                //      if the root is a compound we switch to Normal state
                fsm::ParseNbtFsmState::NbtRoot => {
                    depth_delta = self.parse_normal_state(
                        &mut new_nbt_tag,
                        &mut new_tag_position,
                        nbt_parent_index,
                        &mut cursor,
                        true,
                    )?;
                    self.nbt_parser
                        .change_state_to(fsm::ParseNbtFsmState::Normal);
                }
                // #32 ParseFSM is in normal state: we are parsing any NbtTag that is NOT a List child
                fsm::ParseNbtFsmState::Normal => {
                    depth_delta = self.parse_normal_state(
                        &mut new_nbt_tag,
                        &mut new_tag_position,
                        nbt_parent_index,
                        &mut cursor,
                        false,
                    )?;
                }
                // #33 ParseFSM is in List state: NbtTag which are chidlrend ofLists NbtTags have no names and no values
                fsm::ParseNbtFsmState::List => {
                    depth_delta = self.parse_list_state(
                        &mut new_nbt_tag,
                        &mut new_tag_position,
                        &mut cursor,
                    )?;
                }
                // #34 ParseFSM is in EndOfFile: there are no more bytes to read
                fsm::ParseNbtFsmState::EndOfFile => {
                    break;
                }
            }
            // #4 Append the parsed NbtTag to the NbtTag tree and prepare for the next iteration
            // #41 Update the NbtTag position in the NbtTag tree
            self.update_tag_position(
                &new_nbt_tag.value().into_id(),
                &mut new_tag_position,
                &mut cursor,
                nbt_parent_index,
            );
            new_nbt_tag.set_position(new_tag_position.clone());

            // #42 Append the parsed NbtTag to the NbtTag tree
            self.append_nbt_tag(&new_nbt_tag, nbt_parent_index);
            self.nbt_parser.increment_index();

            // #43 check if we are at the end of the file or we need to proceed to the next loop
            if new_nbt_tag.position().byte_end_all() >= (total_bytes - 1) {
                self.nbt_parser
                    .change_state_to(fsm::ParseNbtFsmState::EndOfFile);
                break; //TODO Remove
            }
        }

        Ok(())
    }

    fn update_tag_position(
        &mut self,
        tag_id: &NbtTagId,
        new_tag_position: &mut NbtTagPosition,
        cursor: &mut Cursor<Vec<u8>>,
        nbt_parent_index: usize,
    ) {
        let new_end_all = cursor.position() as usize;

        // byte_end must always be greater or equal to byte_start
        // in case of a list of compound, byte_end would be 1 less then byte_start if we do not check this
        // compound tag only has 1 byte id, but list children do not have tag ids, since tag id is defined by the list itself
        if new_end_all > new_tag_position.byte_start_all() {
            new_tag_position.set_byte_end_all(new_end_all - 1);
        } else {
            new_tag_position.set_byte_end_all(new_end_all);
        }
        new_tag_position.set_index(*self.nbt_parser.index());
        new_tag_position.set_depth(*self.nbt_parser.tree_depth());
        new_tag_position.set_parent(nbt_parent_index);

        match tag_id {
            NbtTagId::Compound | NbtTagId::List => {
                // do nothing, byte_end_with_children will be set when a new child is appended
            }
            _ => {
                //all the other tag ids cannot have children so we set byte_end_with_children to the same value as byte_end_all
                new_tag_position.set_byte_end_all_with_children(new_tag_position.byte_end_all());
            }
        }
    }

    fn init_tag_position(
        &mut self,
        new_tag_position: &mut NbtTagPosition,
        cursor: &mut Cursor<Vec<u8>>,
    ) {
        new_tag_position.reset();
        new_tag_position.set_byte_start_all(cursor.position() as usize);
        new_tag_position.set_byte_end_all(new_tag_position.byte_start_all());
    }

    fn append_nbt_tag(&mut self, nbt_tag: &NbtTag, nbt_parent_index: usize) {
        let index = nbt_tag.position().index();
        let name = nbt_tag.name().to_string();

        self.tags.push(nbt_tag.clone());
        self.tags_map.insert(name, index);
        self.add_child_to_parent(&nbt_tag, nbt_parent_index);
    }

    fn parse_list_state(
        &mut self,
        new_nbt_tag: &mut NbtTag,
        new_tag_position: &mut NbtTagPosition,
        cursor: &mut Cursor<Vec<u8>>,
    ) -> Result<i64, NbtReadError> {
        let mut depth_delta = 0;
        let tag_id = *self.nbt_parser.list_tag_id();
        new_nbt_tag.set_name("".to_string());

        new_tag_position.set_byte_start_value(cursor.position() as usize);
        new_nbt_tag.set_value(fsm::parse::nbt_tag(cursor, &tag_id)?);
        new_tag_position.set_byte_end_value(cursor.position() as usize);

        if self.nbt_parser.is_list_end() {
            self.nbt_parser
                .change_state_to(fsm::ParseNbtFsmState::Normal);
            self.nbt_parser.reset_list();

            if let NbtTagId::Compound = tag_id {
                depth_delta += 1;
                // if we are in a list of compound, we need to exist the list parser and go back to normal
                // the list is finished, so we do not need to store the list parser status
                self.nbt_parser
                    .change_state_to(fsm::ParseNbtFsmState::Normal);
            } else {
                depth_delta -= 1
            }
        } else {
            if (self.nbt_parser.list_index() >= MAX_LIST_LENGTH - 1)
                || (self.nbt_parser.list_index() >= self.nbt_parser.list_len() - 1)
            {
                return Err(NbtReadError::NbtListLongerThanDeclared);
            }
            self.nbt_parser.increment_list_index();

            if let NbtTagId::Compound = tag_id {
                depth_delta += 1;

                // if we are in a list of compound, we need to exist the list parser and go back to normal
                // but we also need to store the point in the list were we are
                self.nbt_parser
                    .change_state_to(fsm::ParseNbtFsmState::Normal);
                self.nbt_parser.switch_list_ctx();
            }
        }

        Ok(depth_delta)
    }

    fn parse_normal_state(
        &mut self,
        new_nbt_tag: &mut NbtTag,
        new_tag_position: &mut NbtTagPosition,
        nbt_parent_index: usize,
        cursor: &mut Cursor<Vec<u8>>,
        error_if_not_tag_compound: bool,
    ) -> Result<i64, NbtReadError> {
        let mut depth_delta = 0;

        let tag_id = self.parse_nbt_tag_id(new_tag_position, cursor)?;

        if error_if_not_tag_compound && tag_id != NbtTagId::Compound {
            return Err(NbtReadError::InvalidNbtRootTagId);
        }

        if let NbtTagId::End = tag_id {
            depth_delta = self.exit_nbttag_compound(nbt_parent_index);
        } else {
            self.parse_nbt_tag_name_and_value(new_nbt_tag, new_tag_position, tag_id, cursor)?;

            if let NbtTagType::List(ref list_elem_tag_ids) = new_nbt_tag.value() {
                self.nbt_parser.set_list_tag_id(list_elem_tag_ids.0);
                self.nbt_parser.set_list_len(list_elem_tag_ids.1);
                self.nbt_parser.change_state_to(fsm::ParseNbtFsmState::List);
                depth_delta += 1;
            }

            if let NbtTagId::Compound = tag_id {
                depth_delta += 1;
            }
        }

        Ok(depth_delta)
    }

    fn parse_nbt_tag_id(
        &mut self,
        new_tag_position: &mut NbtTagPosition,
        cursor: &mut Cursor<Vec<u8>>,
    ) -> Result<NbtTagId, NbtReadError> {
        new_tag_position.set_byte_start_id(cursor.position() as usize);
        let tag_id = match fsm::parse::nbt_tag_id(cursor) {
            Ok(id) => match id {
                Some(id) => id,
                None => return Err(NbtReadError::InvalidContent),
            },
            Err(e) => return Err(e),
        };
        new_tag_position.set_byte_end_id((cursor.position() - 1) as usize);

        Ok(tag_id)
    }

    fn parse_nbt_tag_name_and_value(
        &mut self,
        new_nbt_tag: &mut NbtTag,
        new_tag_position: &mut NbtTagPosition,
        tag_id: NbtTagId,
        cursor: &mut Cursor<Vec<u8>>,
    ) -> Result<(), NbtReadError> {
        //parse NbtTag Name
        new_tag_position.set_byte_start_name(cursor.position() as usize);
        new_nbt_tag.set_name(fsm::parse::nbt_tag_string(cursor)?);
        new_tag_position.set_byte_end_name((cursor.position() - 1) as usize);

        //parse NbtTag Value
        new_tag_position.set_byte_start_value(cursor.position() as usize);
        new_nbt_tag.set_value(fsm::parse::nbt_tag(cursor, &tag_id)?);
        new_tag_position.set_byte_end_value((cursor.position() - 1) as usize);

        Ok(())
    }

    fn update_nbttree_depth(
        &mut self,
        nbt_parent_index: usize,
        depth_delta: i64,
    ) -> Result<usize, NbtReadError> {
        let new_parent_index;
        self.nbt_parser
            .set_tree_depth(self.nbt_parser.tree_depth() + depth_delta);
        new_parent_index = self.set_new_parent_index(depth_delta, nbt_parent_index)?;

        Ok(new_parent_index)
    }

    fn add_child_to_parent(&mut self, new_nbt_tag: &NbtTag, nbt_parent_index: usize) {
        let child_index = new_nbt_tag.position().index();
        let new_end_byte = new_nbt_tag.position().byte_end_all();

        self.tags[nbt_parent_index]
            .position_as_mut()
            .children()
            .push(child_index);
        self.tags[nbt_parent_index]
            .position_as_mut()
            .set_byte_end_all_with_children(new_end_byte);
    }

    fn set_new_parent_index(
        &mut self,
        depth_delta: i64,
        nbt_parent_index: usize,
    ) -> Result<usize, NbtReadError> {
        let new_parent_index;
        match depth_delta {
            0 => {
                // nothing to do, old parent remains valid since we didnt go deeper
                new_parent_index = nbt_parent_index;
            }
            1 => {
                // we moved down in the nbt tree. This tag is the children of the tag in previous depth level
                new_parent_index = self.nbt_parser.index() - 1;
            }
            -1 => {
                //we moved up in the nbt tree. we need to restore the previous parent index
                //the new parent is the parent of the previous parent
                new_parent_index = self.tags[nbt_parent_index].position().parent();
            }
            -2 => {
                //we moved up in the nbt tree. we need to restore the previous parent index
                //this case is only hit when a list of compound is finished
                // -1 because we exit the compound
                // -1 because we exit the list
                let nbt_grandparent_index = self.tags[nbt_parent_index].position().parent();
                new_parent_index = self.tags[nbt_grandparent_index].position().parent();
            }
            _ => {
                //this should never happen, because delta_depth can only be -2, -1, 0, 1
                return Err(NbtReadError::InvalidNbtDepth);
            }
        }
        Ok(new_parent_index)
    }

    fn exit_nbttag_compound(&mut self, nbt_parent_index: usize) -> i64 {
        let mut depth_delta = -1;

        // the tag End is the last in a compound, so its parent is the compound
        // if the grandparent is a list, we need to change the state back to list
        // because reading a list is different than reading any other tag
        let nbt_grandparent_index = self.tags[nbt_parent_index].position().parent();
        let gp_nbt_tag = self.tags[nbt_grandparent_index].value();

        match gp_nbt_tag {
            NbtTagType::List(_) => {
                if self.nbt_parser.restore_list_ctx() {
                    self.nbt_parser.change_state_to(fsm::ParseNbtFsmState::List);
                } else {
                    // only in this case we will have a depth_delta of -2 because
                    // compound is finished (=-1) and the list as well (=-1)

                    //TOTO: i think there could be a bug here if we are the end of two lists
                    // example: list of compounds of list of compounds
                    depth_delta -= 1;
                }
            }
            _ => {
                //nothing to do
            }
        }
        depth_delta
    }
}
