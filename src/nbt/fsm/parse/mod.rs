use byteorder::{BigEndian, ReadBytesExt};
use core::{panic, str};
use std::io::Cursor;

use crate::nbt;


pub fn nbt_tag_id(cursor: &mut Cursor<Vec<u8>>) -> Option<nbt::NbtTagId> {
    let id = cursor.read_u8().expect("Error reading byte from cursor");
    
    match nbt::NbtTagId::from_u8(id) {
        None => None,
        Some(id) => Some(id),
    }
}

pub fn nbt_tag_string(cursor: &mut Cursor<Vec<u8>>) -> Result<String, nbt::NbtReadError> {
    let name_len = cursor.read_i16::<BigEndian>().unwrap();
    let mut name = String::with_capacity(name_len as usize);

    for _ in 0..name_len {
        let ch = match cursor.read_u8() {
            Ok(ch) => ch,
            Err(e) => return Err(nbt::NbtReadError::Io(e)),
        };
        name.push(ch as char)
    }
    
    Ok(name)
}

pub fn nbt_tag(cursor: &mut Cursor<Vec<u8>>, tag_id: &nbt::NbtTagId) -> Result<nbt::NbtTagType, nbt::NbtReadError> {
    let tag_value = match tag_id {
        nbt::NbtTagId::End => nbt::NbtTagType::End(None),
        
        nbt::NbtTagId::Byte => {        
            let raw_tag_value = match cursor.read_i8() {
                Ok(x) => x,
                Err(e) => return Err(nbt::NbtReadError::Io(e)),
            };
            nbt::NbtTagType::Byte(raw_tag_value)
        },

        nbt::NbtTagId::Short => {
            let raw_tag_value = match cursor.read_i16::<BigEndian>() {
                Ok(x) => x,
                Err(e) => return Err(nbt::NbtReadError::Io(e)),
            };
            nbt::NbtTagType::Short(raw_tag_value)
        },

        nbt::NbtTagId::Int => {
            let raw_tag_value = match cursor.read_i32::<BigEndian>() {
                Ok(x) => x,
                Err(e) => return Err(nbt::NbtReadError::Io(e)),
            };
            nbt::NbtTagType::Int(raw_tag_value)
        },

        nbt::NbtTagId::Long => {
            let raw_tag_value = match cursor.read_i64::<BigEndian>() {
                Ok(x) => x,
                Err(e) => return Err(nbt::NbtReadError::Io(e)),
            };
            nbt::NbtTagType::Long(raw_tag_value)
        },

        nbt::NbtTagId::Float => {
            let raw_tag_value = match cursor.read_f32::<BigEndian>() {
                Ok(x) => x,
                Err(e) => return Err(nbt::NbtReadError::Io(e)),
            };
            nbt::NbtTagType::Float(raw_tag_value)
        },

        nbt::NbtTagId::Double => {
            let raw_tag_value = match cursor.read_f64::<BigEndian>() {
                Ok(x) => x,
                Err(e) => return Err(nbt::NbtReadError::Io(e)),
            };
            nbt::NbtTagType::Double(raw_tag_value)
        },
        
        nbt::NbtTagId::ByteArray => {
            let len = match cursor.read_i32::<BigEndian>() {
                Ok(x) => x,
                Err(e) => return Err(nbt::NbtReadError::Io(e)),
            };

            if len > 65_536 {
                //TODO error handling
            }

            let mut buf = Vec::with_capacity(len as usize);
            for _ in 0..len {
                let x = cursor.read_i8().unwrap();
                buf.push(x);
            }

            nbt::NbtTagType::ByteArray(buf)
        },

        nbt::NbtTagId::String => {
            let raw_tag_value = nbt_tag_string(cursor);
            
            match raw_tag_value {
                Ok(value) => nbt::NbtTagType::String(value),
                Err(e) => return Err(e),     
            }
        },

        nbt::NbtTagId::List => {
            /* let raw_tag_value = match NbtTag::parse_nbt_tag_string(cursor) {
                //here we only store the nbt name and the type of the elements in the list, we parse the list in the next iterations
                Ok(x) => x,
                Err(e) => return Err(e),
            }; */
            let list_elem_tag_ids =  match nbt_tag_id(cursor) {
                None => return Err(nbt::NbtReadError::InvalidContent),
                Some(list_elem_tag_ids) => list_elem_tag_ids,
            };
            let len = cursor.read_i32::<BigEndian>().unwrap();
            if len > 65_536 {
                //TODO error handling
                panic!("List length is too large");
            }
            nbt::NbtTagType::List((list_elem_tag_ids, len))
        },
        
        nbt::NbtTagId::Compound => {
           /*  let compound_values = match NbtTag::parse_nbt_tag_string(cursor) {
                //here we only store the nbt name, we parse the compound in the next iterations
                Ok(values) => values,
                Err(e) => return Err(e),
            }; */
            nbt::NbtTagType::Compound("".to_string())
        },
        
        nbt::NbtTagId::IntArray => {
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

            nbt::NbtTagType::IntArray(buf)
        },
        nbt::NbtTagId::LongArray => {
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

            nbt::NbtTagType::LongArray(buf)
        }
    };

    Ok(tag_value)
}