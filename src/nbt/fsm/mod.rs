use crate::nbt;
use std::io::{Cursor, Seek, SeekFrom};

mod parse;

pub enum ParseNbtFsm {
    Normal,
    List,
    EndOfFile
}

#[derive(Debug, Clone)]
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

pub struct NbtParser <'a>{
    state: ParseNbtFsm,
    list_parser: NbtListParser,
    cursor: &'a mut Cursor<Vec<u8>>,
    index: usize,
    tree_depth: i64,
}

impl<'a> NbtParser <'a>{
    pub fn new(state: ParseNbtFsm, cursor: &mut Cursor<Vec<u8>>) -> NbtParser {
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

fn set_new_parent_index(tag_sequence : &mut nbt::NbtTagSequence, depth_delta: &i64, nbt_parser: &mut NbtParser, nbt_parent_index: &mut usize) -> Result<(), nbt::NbtReadError> {

    match depth_delta {
        0 => {
            // nothing to do, old parent remains valid since we didnt go deeper
            },
        1 => {
            // we moved down in the nbt tree. This tag is the children of the tag in previous depth level
            *nbt_parent_index = nbt_parser.index() - 1; 
        },
        -1 => {
            //we moved up in the nbt tree. we need to restore the previous parent index
            //the new parent is the parent of the previous parent
            *nbt_parent_index = tag_sequence.tags[*nbt_parent_index].parent(); 
        },
        -2 => {
            //we moved up in the nbt tree. we need to restore the previous parent index
            //this case is only hit when a list of compound is finished
            // -1 because we exit the compound
            // -1 because we exit the list
            let nbt_grandparent_index = tag_sequence.tags[*nbt_parent_index].parent();
            *nbt_parent_index = tag_sequence.tags[nbt_grandparent_index].parent(); 
        }
        _ => {
            //this should never happen, because delta_depth can only be -2, -1, 0, 1
            return Err(nbt::NbtReadError::InvalidNbtDepth)
        }
    }
    Ok(())
}

fn store_list_ctx(unfinished_lists: &mut Vec<NbtListParser>, nbt_parser: &mut NbtParser) {
    unfinished_lists.push(nbt_parser.list_parser.clone());
    nbt_parser.list_parser.reset();
}

fn restore_list_ctx(unfinished_lists: &mut Vec<NbtListParser>, nbt_parser: &mut NbtParser) -> bool {
    match unfinished_lists.pop() { 
        //the list of compounds was not yet finished, restore the ctx
        Some(previous_list_parser) => {
            nbt_parser.list_parser = previous_list_parser;
            true
        }
        // the list of compounds was finished and we do not need to restore the ctx
        // only in this case we will have a depth_delta of -2 because
        // compound is finished (=-1) and the list as well (=-1)
        None => false
    } 
}

fn exit_nbttag_compound(tag_sequence : &mut nbt::NbtTagSequence, unfinished_lists: &mut Vec<NbtListParser>, nbt_parser: &mut NbtParser, nbt_parent_index: usize) -> i64{
    
    let mut depth_delta = -1;

    // the tag End is the last in a compound, so its parent is the compound
    // if the grandparent is a list, we need to change the state back to list
    // because reading a list is different than reading any other tag
    let nbt_grandparent_index = tag_sequence.tags[nbt_parent_index].parent();
    let gp_nbt_tag = tag_sequence.tags[nbt_grandparent_index].value();
    
    match gp_nbt_tag {
        nbt::NbtTagType::List(_) => {
            if restore_list_ctx(unfinished_lists, nbt_parser) {
                nbt_parser.change_state_to(ParseNbtFsm::List);
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

pub fn parse(test_sequence : &mut nbt::NbtTagSequence, nbt_parser: &mut NbtParser) -> Result<(), nbt::NbtReadError> {
    
    let cursor = &mut nbt_parser.cursor();
    let mut tag_id;
    let mut depth_delta= 0;
    let total_bytes = cursor.seek(SeekFrom::End(0)).unwrap();
    let mut nbt_parent_index = 0;
    let mut unfinished_lists = Vec::new();

    cursor.seek(SeekFrom::Start(0)).unwrap();
    loop {

        let byte_start = cursor.position();   
        let mut tag_name = String::new();
        let mut tag_value = nbt::NbtTagType::End(None);
        
        nbt_parser.tree_depth+=depth_delta;
        set_new_parent_index(test_sequence, &depth_delta, nbt_parser, &mut nbt_parent_index)?;
        depth_delta = 0;

        match nbt_parser.state() {
            ParseNbtFsm::Normal => {
                tag_id = match parse::nbt_tag_id(cursor) {
                    Ok(id) => {
                        match id {
                            Some(id) => id,
                            None => return Err(nbt::NbtReadError::InvalidContent)
                        }
                    },
                    Err(e) => return Err(e)
                };

                if let nbt::NbtTagId::End = tag_id {
                    depth_delta = exit_nbttag_compound(test_sequence, &mut unfinished_lists, nbt_parser, nbt_parent_index);
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
                }
                
            },

            ParseNbtFsm::List => {
                tag_id = *nbt_parser.list_parser.tag_id();
                tag_name = "".to_string();
                tag_value = parse::nbt_tag(cursor, &tag_id)?;
                
                if nbt_parser.list_parser.is_end() {   
                    nbt_parser.change_state_to(ParseNbtFsm::Normal); 
                    nbt_parser.list_parser.reset();
                    
                    if let nbt::NbtTagId::Compound = tag_id {
                        depth_delta += 1;
                        // if we are in a list of compound, we need to exist the list parser and go back to normal
                        // the list is finished, so we do not need to store the list parser status
                        nbt_parser.change_state_to(ParseNbtFsm::Normal);
                    }
                    else {
                        depth_delta -= 1 
                    }
                }
                else {
                    nbt_parser.list_parser.increment();  

                    if let nbt::NbtTagId::Compound = tag_id {
                        depth_delta += 1;
    
                        // if we are in a list of compound, we need to exist the list parser and go back to normal
                        // but we also need to store the point in the list were we are
                        nbt_parser.change_state_to(ParseNbtFsm::Normal);
                        store_list_ctx(&mut unfinished_lists, nbt_parser);
                    }
                }
            },

            ParseNbtFsm::EndOfFile => {
                break;
            },
        }

        let byte_end = cursor.position();
        
        let new_nbt_tag = nbt::NbtTag { name: tag_name, 
                                                value: tag_value, 
                                                byte_start: byte_start, 
                                                byte_end: byte_end,
                                                index: *nbt_parser.index(),
                                                depth: nbt_parser.tree_depth,
                                                parent: nbt_parent_index,
                                                children: Vec::new()};  
        
        test_sequence.tags.push(new_nbt_tag);

        nbt_parser.increment_index();
        if byte_end >= total_bytes {
            nbt_parser.change_state_to(ParseNbtFsm::EndOfFile);
            break; //TODO Remove
        }
    }

    Ok(())
}