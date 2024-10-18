use serde::{Deserialize, Serialize};

use crate::nbt;

pub mod parse;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default, Serialize, Deserialize)]
pub enum ParseNbtFsmState {
    #[default]
    Normal,
    List,
    EndOfFile,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default, Serialize, Deserialize)]
pub struct NbtListParser {
    list_tag_id: nbt::NbtTagId,
    list_len: i32,
    list_elem_count: i32,
}

impl NbtListParser {
    pub fn new() -> NbtListParser {
        NbtListParser {
            list_tag_id: nbt::NbtTagId::default(),
            list_len: 0,
            list_elem_count: 0,
        }
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

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, Serialize, Deserialize)]
pub struct NbtParser {
    state: ParseNbtFsmState,
    pub list_parser: NbtListParser,
    pub unfinished_lists: Vec<NbtListParser>,
    //cursor: &'a mut Cursor<Vec<u8>>,
    index: usize,
    tree_depth: i64,
}

impl NbtParser {
    pub fn new() -> NbtParser {
        NbtParser {
            state: ParseNbtFsmState::default(),
            list_parser: NbtListParser::new(),
            unfinished_lists: Vec::<NbtListParser>::new(),
            //cursor: cursor,
            index: 0,
            tree_depth: 0,
        }
    }

    pub fn change_state_to(&mut self, state: ParseNbtFsmState) {
        self.state = state;
    }

    pub fn state(&self) -> &ParseNbtFsmState {
        &self.state
    }

    /* pub fn cursor(&mut self) -> Cursor<Vec<u8>> {
        self.cursor.clone() //TODO: use &mut self.cursor
    } */

    pub fn index(&self) -> &usize {
        &self.index
    }

    pub fn tree_depth(&self) -> &i64 {
        &self.tree_depth
    }

    pub fn set_tree_depth(&mut self, depth: i64) {
        self.tree_depth = depth;
    }

    pub fn increment_index(&mut self) {
        self.index = self.index + 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nbt::NbtTagId;

    // Tests for NbtListParser
    #[test]
    fn test_nbt_list_parser_new() {
        let parser = NbtListParser::new();
        assert_eq!(parser.list_tag_id, NbtTagId::End);
        assert_eq!(parser.list_len, 0);
        assert_eq!(parser.list_elem_count, 0);
    }

    #[test]
    fn test_nbt_list_parser_set_id() {
        let mut parser = NbtListParser::new();
        parser.set_id(NbtTagId::Byte);
        assert_eq!(parser.list_tag_id, NbtTagId::Byte);

        parser.set_id(NbtTagId::String);
        assert_eq!(parser.list_tag_id, NbtTagId::String);
    }

    #[test]
    fn test_nbt_list_parser_tag_id() {
        let mut parser = NbtListParser::new();
        assert_eq!(parser.tag_id(), &NbtTagId::End);

        parser.set_id(NbtTagId::Short);
        assert_eq!(parser.tag_id(), &NbtTagId::Short);
    }

    #[test]
    fn test_nbt_list_parser_set_len() {
        let mut parser = NbtListParser::new();
        parser.set_len(10);
        assert_eq!(parser.list_len, 10);

        parser.set_len(0);
        assert_eq!(parser.list_len, 0);

        parser.set_len(-5);
        assert_eq!(parser.list_len, -5);
    }

    #[test]
    fn test_nbt_list_parser_increment() {
        let mut parser = NbtListParser::new();
        assert_eq!(parser.list_elem_count, 0);
        parser.increment();
        assert_eq!(parser.list_elem_count, 1);
        parser.increment();
        assert_eq!(parser.list_elem_count, 2);
    }

    #[test]
    fn test_nbt_list_parser_reset() {
        let mut parser = NbtListParser::new();
        parser.set_id(NbtTagId::Int);
        parser.set_len(5);
        parser.increment();
        assert_eq!(parser.list_tag_id, NbtTagId::Int);
        assert_eq!(parser.list_len, 5);
        assert_eq!(parser.list_elem_count, 1);

        parser.reset();
        assert_eq!(parser.list_tag_id, NbtTagId::End);
        assert_eq!(parser.list_len, 0);
        assert_eq!(parser.list_elem_count, 0);
    }

    #[test]
    fn test_nbt_list_parser_is_end() {
        let mut parser = NbtListParser::new();
        parser.set_len(5);
        parser.set_id(NbtTagId::Byte);

        // Initially, list_elem_count = 0, list_len = 5
        assert!(!parser.is_end());

        // After 3 increments, list_elem_count = 3
        parser.increment(); // 1
        parser.increment(); // 2
        parser.increment(); // 3
        assert!(!parser.is_end());

        // After 1 more increment, list_elem_count = 4
        parser.increment(); // 4
                            // list_len -1 = 4, so list_elem_count >= list_len -1
        assert!(parser.is_end());

        // Incrementing beyond
        parser.increment(); // 5
        assert!(parser.is_end());
    }

    #[test]
    fn test_nbt_list_parser_is_end_with_zero_length() {
        let mut parser = NbtListParser::new();
        parser.set_len(0);
        parser.set_id(NbtTagId::List);

        // list_elem_count = 0, list_len -1 = -1
        assert!(parser.is_end());
    }

    #[test]
    fn test_nbt_list_parser_is_end_with_negative_length() {
        let mut parser = NbtListParser::new();
        parser.set_len(-3);
        parser.set_id(NbtTagId::List);

        // list_elem_count = 0, list_len -1 = -4
        assert!(parser.is_end());

        parser.increment(); // 1
        assert!(parser.is_end());
    }

    // Tests for NbtParser
    #[test]
    fn test_nbt_parser_new() {
        let parser = NbtParser::new();
        match parser.state {
            ParseNbtFsmState::Normal => (),
            _ => panic!("Initial state should be Normal"),
        }
        assert_eq!(parser.list_parser.list_tag_id, NbtTagId::End);
        assert!(parser.unfinished_lists.is_empty());
        assert_eq!(parser.index, 0);
        assert_eq!(parser.tree_depth, 0);
    }

    #[test]
    fn test_nbt_parser_change_state_to() {
        let mut parser = NbtParser::new();
        parser.change_state_to(ParseNbtFsmState::List);
        match parser.state {
            ParseNbtFsmState::List => (),
            _ => panic!("State should be List"),
        }

        parser.change_state_to(ParseNbtFsmState::EndOfFile);
        match parser.state {
            ParseNbtFsmState::EndOfFile => (),
            _ => panic!("State should be EndOfFile"),
        }
    }

    #[test]
    fn test_nbt_parser_index() {
        let parser = NbtParser::new();
        assert_eq!(*parser.index(), 0);

        let mut parser = NbtParser::new();
        parser.increment_index();
        assert_eq!(*parser.index(), 1);
        parser.increment_index();
        assert_eq!(*parser.index(), 2);
    }

    #[test]
    fn test_nbt_parser_tree_depth() {
        let parser = NbtParser::new();
        assert_eq!(*parser.tree_depth(), 0);

        let mut parser = NbtParser::new();
        parser.set_tree_depth(3);
        assert_eq!(*parser.tree_depth(), 3);

        parser.set_tree_depth(-1);
        assert_eq!(*parser.tree_depth(), -1);
    }

    #[test]
    fn test_nbt_parser_set_tree_depth() {
        let mut parser = NbtParser::new();
        parser.set_tree_depth(10);
        assert_eq!(parser.tree_depth, 10);

        parser.set_tree_depth(0);
        assert_eq!(parser.tree_depth, 0);
    }

    #[test]
    fn test_nbt_parser_increment_index() {
        let mut parser = NbtParser::new();
        assert_eq!(parser.index, 0);
        parser.increment_index();
        assert_eq!(parser.index, 1);
        parser.increment_index();
        assert_eq!(parser.index, 2);
    }

    #[test]
    fn test_nbt_parser_unfinished_lists() {
        let mut parser = NbtParser::new();
        assert!(parser.unfinished_lists.is_empty());

        let list1 = NbtListParser::new();
        parser.unfinished_lists.push(list1.clone());
        assert_eq!(parser.unfinished_lists.len(), 1);
        assert_eq!(parser.unfinished_lists[0], list1);

        let list2 = NbtListParser::new();
        parser.unfinished_lists.push(list2.clone());
        assert_eq!(parser.unfinished_lists.len(), 2);
        assert_eq!(parser.unfinished_lists[1], list2);
    }

    #[test]
    fn test_nbt_parser_list_parser_methods() {
        let mut parser = NbtParser::new();
        let mut list_parser = &mut parser.list_parser;

        // Test default values
        assert_eq!(list_parser.list_tag_id, NbtTagId::End);
        assert_eq!(list_parser.list_len, 0);
        assert_eq!(list_parser.list_elem_count, 0);

        // Set ID
        list_parser.set_id(NbtTagId::Float);
        assert_eq!(list_parser.list_tag_id, NbtTagId::Float);

        // Set Length
        list_parser.set_len(20);
        assert_eq!(list_parser.list_len, 20);

        // Increment
        list_parser.increment();
        assert_eq!(list_parser.list_elem_count, 1);

        // Check is_end
        list_parser.set_len(3);
        list_parser.set_id(NbtTagId::Double);
        list_parser.list_elem_count = 1;
        assert!(!list_parser.is_end());
        list_parser.increment();
        assert!(list_parser.is_end());

        // Reset
        list_parser.reset();
        assert_eq!(list_parser.list_tag_id, NbtTagId::End);
        assert_eq!(list_parser.list_len, 0);
        assert_eq!(list_parser.list_elem_count, 0);
    }
}
