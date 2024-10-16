use crate::nbt;

pub mod parse;

pub enum ParseNbtFsm {
    Normal,
    List,
    EndOfFile,
}

#[derive(Debug, Clone)]
pub struct NbtListParser {
    list_tag_id: nbt::NbtTagId,
    list_len: i32,
    list_elem_count: i32,
}

impl NbtListParser {
    pub fn new() -> NbtListParser {
        NbtListParser {
            list_tag_id: nbt::NbtTagId::End,
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

pub struct NbtParser {
    state: ParseNbtFsm,
    pub list_parser: NbtListParser,
    pub unfinished_lists: Vec<NbtListParser>,
    //cursor: &'a mut Cursor<Vec<u8>>,
    index: usize,
    tree_depth: i64,
}

impl NbtParser {
    pub fn new(state: ParseNbtFsm) -> NbtParser {
        NbtParser {
            state: state,
            list_parser: NbtListParser::new(),
            unfinished_lists: Vec::<NbtListParser>::new(),
            //cursor: cursor,
            index: 0,
            tree_depth: 0,
        }
    }

    pub fn change_state_to(&mut self, state: ParseNbtFsm) {
        self.state = state;
    }

    pub fn state(&self) -> &ParseNbtFsm {
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
