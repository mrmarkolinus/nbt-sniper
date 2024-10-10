use crate::nbt;
use std::io::{Cursor, Seek, SeekFrom};

mod parse;

pub enum ParseNbtFsm {
    Normal,
    List,
    EndOfFile
}

struct NbtListParser {
    list_tag_id: nbt::NbtTagId,
    list_len: i32,
    list_elem_count: i32,
}

impl NbtListParser {
    pub fn new() -> NbtListParser {
        NbtListParser { list_tag_id: nbt::NbtTagId::End, list_len: 0, list_elem_count: 0 }
    }

    pub fn set_id(&mut self, tag_id: nbt::NbtTagId) {
        self.list_tag_id = tag_id;
    }

    pub fn tag_id(&self) -> &nbt::NbtTagId {
        &self.list_tag_id
    }

    pub fn set_len(&mut self, len: i32) {
        self.list_len = len;
    }

    pub fn increment(&mut self) {
        self.list_elem_count = self.list_elem_count + 1;
    }

    pub fn reset(&mut self) {
        self.list_tag_id = nbt::NbtTagId::End;
        self.list_len = 0;
        self.list_elem_count = 0;
    }

    pub fn is_end(&self) -> bool {
        self.list_elem_count >= self.list_len - 1
    }
}

pub struct NbtParser {
    state: ParseNbtFsm,
    list_parser: NbtListParser,
    cursor: Cursor<Vec<u8>>,
    index: usize,
    tree_depth: i64,
}

impl NbtParser {
    pub fn new(state: ParseNbtFsm, cursor: Cursor<Vec<u8>>) -> NbtParser {
        NbtParser { state: state, 
                    list_parser: NbtListParser::new(),
                    cursor: cursor,
                    index: 0,
                    tree_depth: 0
                    }
    }

    pub fn change_state_to(&mut self, state: ParseNbtFsm) {
        self.state = state;
    }

    pub fn state(&self) -> &ParseNbtFsm {
        &self.state
    }

    pub fn cursor(&mut self) -> Cursor<Vec<u8>> {
        self.cursor.clone() //TODO: use &mut self.cursor
    }

    pub fn index(&self) -> &usize {
        &self.index
    }

    pub fn increment_index(&mut self) {
        self.index = self.index + 1;
    }
}

pub fn parse(test_sequence : &mut nbt::NbtTagSequence, nbt_parser: &mut NbtParser) {//-> (NbtTagId, String, NbtTagType) {
    
    let mut cursor = nbt_parser.cursor();
    let mut tag_id;
    let mut depth_delta= 0;
    let total_bytes = cursor.seek(SeekFrom::End(0)).unwrap();
    
    cursor.seek(SeekFrom::Start(0)).unwrap();
    loop {

        let byte_start = cursor.position();   
        let mut tag_name = String::new();
        let mut tag_value = nbt::NbtTagType::End(None);
        
        nbt_parser.tree_depth+=depth_delta;
        depth_delta = 0;

        match nbt_parser.state() {
            ParseNbtFsm::Normal => {
                tag_id = parse::nbt_tag_id(&mut cursor).unwrap();

                if let nbt::NbtTagId::End = tag_id {
                    depth_delta -= 1;
                }
                else {
                    tag_name = parse::nbt_tag_string(&mut cursor).unwrap();    
                    tag_value = parse::nbt_tag(&mut cursor, &tag_id).unwrap();

                    if let nbt::NbtTagType::List(ref list_elem_tag_ids) = tag_value {
                        nbt_parser.list_parser.set_id(list_elem_tag_ids.0);
                        nbt_parser.list_parser.set_len(list_elem_tag_ids.1);
                        nbt_parser.change_state_to(ParseNbtFsm::List); 
                        depth_delta += 1;
                    }

                    if let nbt::NbtTagId::Compound = tag_id {
                        depth_delta += 1;
                    }
                }
                
            },

            ParseNbtFsm::List => {
                if nbt_parser.list_parser.is_end() {
                    tag_id = *nbt_parser.list_parser.tag_id();
                    nbt_parser.list_parser.reset();
                    nbt_parser.change_state_to(ParseNbtFsm::Normal); 
                    depth_delta -= 1;
                }
                else {
                    nbt_parser.list_parser.increment(); 
                    tag_id = *nbt_parser.list_parser.tag_id();            
                }

                tag_name = "".to_string();
                tag_value = parse::nbt_tag(&mut cursor, &tag_id).unwrap();

                if let nbt::NbtTagId::Compound = tag_id {
                    depth_delta += 1;
                }
            },

            ParseNbtFsm::EndOfFile => {
                break;
            },
        }

        let byte_end = cursor.position();



        test_sequence.tags.push(nbt::NbtTag { name: tag_name, 
                                        value: tag_value, 
                                        byte_start: byte_start, 
                                        byte_end: byte_end,
                                        index: *nbt_parser.index(),
                                        depth: nbt_parser.tree_depth});

        nbt_parser.increment_index();

        if byte_end >= total_bytes {
            nbt_parser.change_state_to(ParseNbtFsm::EndOfFile);
            break; //TODO Remove
        }
    }
}