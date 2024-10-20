use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

use crate::nbt;

pub fn nbt_tag_id(
    cursor: &mut Cursor<Vec<u8>>,
) -> Result<Option<nbt::NbtTagId>, nbt::NbtReadError> {
    let id = cursor.read_u8()?;

    let tag_id = match nbt::NbtTagId::from_u8(id) {
        None => None,
        Some(id) => Some(id),
    };

    Ok(tag_id)
}

pub fn nbt_tag_string(cursor: &mut Cursor<Vec<u8>>) -> Result<String, nbt::NbtReadError> {
    let name_len = cursor.read_i16::<BigEndian>()?;
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

pub fn nbt_tag(
    cursor: &mut Cursor<Vec<u8>>,
    tag_id: &nbt::NbtTagId,
) -> Result<nbt::NbtTagType, nbt::NbtReadError> {
    let tag_value = match tag_id {
        nbt::NbtTagId::End => nbt::NbtTagType::End(None),

        nbt::NbtTagId::Byte => {
            let raw_tag_value = match cursor.read_i8() {
                Ok(x) => x,
                Err(e) => return Err(nbt::NbtReadError::Io(e)),
            };
            nbt::NbtTagType::Byte(raw_tag_value)
        }

        nbt::NbtTagId::Short => {
            let raw_tag_value = match cursor.read_i16::<BigEndian>() {
                Ok(x) => x,
                Err(e) => return Err(nbt::NbtReadError::Io(e)),
            };
            nbt::NbtTagType::Short(raw_tag_value)
        }

        nbt::NbtTagId::Int => {
            let raw_tag_value = match cursor.read_i32::<BigEndian>() {
                Ok(x) => x,
                Err(e) => return Err(nbt::NbtReadError::Io(e)),
            };
            nbt::NbtTagType::Int(raw_tag_value)
        }

        nbt::NbtTagId::Long => {
            let raw_tag_value = match cursor.read_i64::<BigEndian>() {
                Ok(x) => x,
                Err(e) => return Err(nbt::NbtReadError::Io(e)),
            };
            nbt::NbtTagType::Long(raw_tag_value)
        }

        nbt::NbtTagId::Float => {
            let raw_tag_value = match cursor.read_f32::<BigEndian>() {
                Ok(x) => x,
                Err(e) => return Err(nbt::NbtReadError::Io(e)),
            };
            nbt::NbtTagType::Float(raw_tag_value)
        }

        nbt::NbtTagId::Double => {
            let raw_tag_value = match cursor.read_f64::<BigEndian>() {
                Ok(x) => x,
                Err(e) => return Err(nbt::NbtReadError::Io(e)),
            };
            nbt::NbtTagType::Double(raw_tag_value)
        }

        nbt::NbtTagId::ByteArray => {
            let len = match cursor.read_i32::<BigEndian>() {
                Ok(x) => x,
                Err(e) => return Err(nbt::NbtReadError::Io(e)),
            };

            if len > nbt::MAX_BYTE_ARRAY_LENGTH {
                return Err(nbt::NbtReadError::InvalidNbtByteArrayLenght);
            }

            let mut buf = Vec::with_capacity(len as usize);
            for _ in 0..len {
                let x = cursor.read_i8()?;
                buf.push(x);
            }

            nbt::NbtTagType::ByteArray(buf)
        }

        nbt::NbtTagId::String => {
            let raw_tag_value = nbt_tag_string(cursor);

            match raw_tag_value {
                Ok(value) => nbt::NbtTagType::String(value),
                Err(e) => return Err(e),
            }
        }

        nbt::NbtTagId::List => {
            let list_elem_tag_ids = match nbt_tag_id(cursor) {
                Ok(id) => match id {
                    None => return Err(nbt::NbtReadError::InvalidContent),
                    Some(list_elem_tag_ids) => list_elem_tag_ids,
                },
                Err(e) => return Err(e),
            };

            let len = cursor.read_i32::<BigEndian>()?;
            if len > nbt::MAX_LIST_LENGTH {
                return Err(nbt::NbtReadError::InvalidNbtListLenght);
            }
            nbt::NbtTagType::List((list_elem_tag_ids, len))
        }

        nbt::NbtTagId::Compound => nbt::NbtTagType::Compound("".to_string()),

        nbt::NbtTagId::IntArray => {
            let len = cursor.read_i32::<BigEndian>()?;
            if len > nbt::MAX_INT_ARRAY_LENGTH {
                return Err(nbt::NbtReadError::InvalidNbtIntArrayLenght);
            }

            let mut buf = Vec::with_capacity(len as usize);
            for _ in 0..len {
                let x = cursor.read_i32::<BigEndian>()?;
                buf.push(x);
            }

            nbt::NbtTagType::IntArray(buf)
        }
        nbt::NbtTagId::LongArray => {
            let len = cursor.read_i32::<BigEndian>()?;
            if len > nbt::MAX_LONG_ARRAY_LENGTH {
                return Err(nbt::NbtReadError::InvalidNbtLongArrayLenght);
            }

            let mut buf = Vec::with_capacity(len as usize);
            for _ in 0..len {
                let x = cursor.read_i64::<BigEndian>()?;
                buf.push(x);
            }

            nbt::NbtTagType::LongArray(buf)
        }
    };

    Ok(tag_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a Cursor from a vector of bytes.
    fn make_cursor(data: Vec<u8>) -> Cursor<Vec<u8>> {
        Cursor::new(data)
    }

    // Tests for `nbt_tag_id`
    #[test]
    fn test_nbt_tag_id_valid_ids() {
        for (id_u8, expected_id) in &[
            (0u8, nbt::NbtTagId::End),
            (1u8, nbt::NbtTagId::Byte),
            (2u8, nbt::NbtTagId::Short),
            (3u8, nbt::NbtTagId::Int),
            (4u8, nbt::NbtTagId::Long),
            (5u8, nbt::NbtTagId::Float),
            (6u8, nbt::NbtTagId::Double),
            (7u8, nbt::NbtTagId::ByteArray),
            (8u8, nbt::NbtTagId::String),
            (9u8, nbt::NbtTagId::List),
            (10u8, nbt::NbtTagId::Compound),
            (11u8, nbt::NbtTagId::IntArray),
            (12u8, nbt::NbtTagId::LongArray),
        ] {
            let cursor = make_cursor(vec![*id_u8]);
            let mut cursor = cursor;
            let result = nbt_tag_id(&mut cursor).unwrap();
            assert_eq!(result, Some(*expected_id));
        }
    }

    #[test]
    fn test_nbt_tag_id_invalid_id() {
        let invalid_id = 13u8; // Assuming 0-12 are valid
        let cursor = make_cursor(vec![invalid_id]);
        let mut cursor = cursor;
        let result = nbt_tag_id(&mut cursor).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_nbt_tag_id_io_error() {
        let cursor = make_cursor(vec![]); // Empty cursor, cannot read u8
        let mut cursor = cursor;
        let result = nbt_tag_id(&mut cursor);
        assert!(matches!(result, Err(nbt::NbtReadError::Io(_))));
    }

    // Tests for `nbt_tag_string`
    #[test]
    fn test_nbt_tag_string_valid() {
        // String "Test"
        let mut data = Vec::new();
        data.extend(&2u16.to_be_bytes()); // name_len = 2
        data.push(b'T');
        data.push(b'e');
        let cursor = make_cursor(data);
        let mut cursor = cursor;
        let result = nbt_tag_string(&mut cursor).unwrap();
        assert_eq!(result, "Te");
    }

    #[test]
    fn test_nbt_tag_string_empty() {
        let mut data = Vec::new();
        data.extend(&0i16.to_be_bytes()); // name_len = 0
        let cursor = make_cursor(data);
        let mut cursor = cursor;
        let result = nbt_tag_string(&mut cursor).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_nbt_tag_string_negative_length() {
        let mut data = Vec::new();
        data.extend(&(-1i16).to_be_bytes()); // name_len = -1
                                             // Since the loop is 0..-1, which is invalid, it will not execute
                                             // The string will have capacity of usize::MAX, but we are not using it
                                             // However, converting -1i16 to usize can cause unexpected behavior
                                             // To prevent this, let's see how the code handles it
                                             // In Rust, 0..-1i16 will not compile, so likely it's cast to usize
                                             // Here, assuming it's cast as 0..(name_len as usize)
                                             // name_len as usize for -1i16 is 65535
                                             // To avoid creating a large string, let's limit the test
                                             // Instead, we can check if the function handles it gracefully
                                             // But as per the original code, it doesn't handle negative lengths
                                             // So it will try to read 65535 bytes, which is not present, leading to an error
        let cursor = make_cursor(data);
        let mut cursor = cursor;
        let result = nbt_tag_string(&mut cursor);
        assert!(matches!(result, Err(nbt::NbtReadError::Io(_))));
    }

    #[test]
    fn test_nbt_tag_string_insufficient_bytes() {
        let mut data = Vec::new();
        data.extend(&4i16.to_be_bytes()); // name_len = 4
        data.extend(&vec![b'T', b'e']); // Only 2 bytes instead of 4
        let cursor = make_cursor(data);
        let mut cursor = cursor;
        let result = nbt_tag_string(&mut cursor);
        assert!(matches!(result, Err(nbt::NbtReadError::Io(_))));
    }

    // Tests for `nbt_tag`
    #[test]
    fn test_nbt_tag_end() {
        let cursor = make_cursor(vec![]); // No data needed for End
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::End).unwrap();
        assert_eq!(result, nbt::NbtTagType::End(None));
    }

    #[test]
    fn test_nbt_tag_byte() {
        let cursor = make_cursor(vec![0x7F]); // i8::MAX
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::Byte).unwrap();
        assert_eq!(result, nbt::NbtTagType::Byte(127));
    }

    #[test]
    fn test_nbt_tag_byte_io_error() {
        let cursor = make_cursor(vec![]); // No data
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::Byte);
        assert!(matches!(result, Err(nbt::NbtReadError::Io(_))));
    }

    #[test]
    fn test_nbt_tag_short() {
        let data = vec![0u8, 42u8];
        let cursor = make_cursor(data); 
        // i16::from_be_bytes([0x00, 0x2A]) = 42
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::Short).unwrap();
        assert_eq!(result, nbt::NbtTagType::Short(42));
    }

    #[test]
    fn test_nbt_tag_short_io_error() {
        let cursor = make_cursor(vec![0x00]); // Incomplete i16
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::Short);
        assert!(matches!(result, Err(nbt::NbtReadError::Io(_))));
    }

    #[test]
    fn test_nbt_tag_int() {
        let data = vec![0u8, 0u8, 0u8, 42u8];
        let cursor = make_cursor(data); // 42
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::Int).unwrap();
        assert_eq!(result, nbt::NbtTagType::Int(42));
    }

    #[test]
    fn test_nbt_tag_int_io_error() {
        let cursor = make_cursor(vec![0x00, 0x00, 0x00]); // Incomplete i32
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::Int);
        assert!(matches!(result, Err(nbt::NbtReadError::Io(_))));
    }

      #[test]
    fn test_nbt_tag_long() {
        let data = vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,42u8];
        let cursor = make_cursor(data); // 42
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::Long).unwrap();
        assert_eq!(result, nbt::NbtTagType::Long(42));
    }

    #[test]
    fn test_nbt_tag_long_io_error() {
        let cursor = make_cursor(vec![0x00; 7]); // Incomplete i64
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::Long);
        assert!(matches!(result, Err(nbt::NbtReadError::Io(_))));
    }

    #[test]
    fn test_nbt_tag_float() {
        let cursor = make_cursor(42f32.to_be_bytes().to_vec());
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::Float).unwrap();
        assert_eq!(result, nbt::NbtTagType::Float(42f32));
    }

    #[test]
    fn test_nbt_tag_float_io_error() {
        let cursor = make_cursor(vec![0x00, 0x00, 0x00]); // Incomplete f32
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::Float);
        assert!(matches!(result, Err(nbt::NbtReadError::Io(_))));
    }

    #[test]
    fn test_nbt_tag_double() {
        let cursor = make_cursor(42f64.to_be_bytes().to_vec());
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::Double).unwrap();
        assert_eq!(result, nbt::NbtTagType::Double(42f64));
    }

    #[test]
    fn test_nbt_tag_double_io_error() {
        let cursor = make_cursor(vec![0x00; 7]); // Incomplete f64
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::Double);
        assert!(matches!(result, Err(nbt::NbtReadError::Io(_))));
    }

    #[test]
    fn test_nbt_tag_byte_array() {
        let len = 3i32.to_be_bytes();
        let data = vec![1i8, 2i8, 3i8];
        let mut combined = Vec::new();
        combined.extend(&len);
        combined.extend(&data.iter().map(|x| *x as u8).collect::<Vec<u8>>());
        let cursor = make_cursor(combined);
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::ByteArray).unwrap();
        assert_eq!(result, nbt::NbtTagType::ByteArray(vec![1, 2, 3]));
    }

    #[test]
    fn test_nbt_tag_byte_array_io_error_length() {
        let cursor = make_cursor(vec![0x00, 0x00]); // Incomplete i32 for length
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::ByteArray);
        assert!(matches!(result, Err(nbt::NbtReadError::Io(_))));
    }

    #[test]
    fn test_nbt_tag_byte_array_io_error_data() {
        let len = 5i32.to_be_bytes();
        let data = vec![1i8, 2i8]; // Only 2 bytes instead of 5
        let mut combined = Vec::new();
        combined.extend(&len);
        combined.extend(&data.iter().map(|x| *x as u8).collect::<Vec<u8>>());
        let cursor = make_cursor(combined);
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::ByteArray);
        assert!(matches!(result, Err(nbt::NbtReadError::Io(_))));
    }

    #[test]
    fn test_nbt_tag_string() {
        // name_len = 4, string "Test"
        let mut data = Vec::new();
        data.extend(&4i16.to_be_bytes());
        data.extend(&b"Test".to_vec());
        let cursor = make_cursor(data);
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::String).unwrap();
        assert_eq!(result, nbt::NbtTagType::String("Test".to_string()));
    }

    #[test]
    fn test_nbt_tag_string_error() {
        // name_len = 4, but only 3 bytes provided
        let mut data = Vec::new();
        data.extend(&4i16.to_be_bytes());
        data.extend(&b"Tes".to_vec());
        let cursor = make_cursor(data);
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::String);
        assert!(matches!(result, Err(nbt::NbtReadError::Io(_))));
    }

    #[test]
    fn test_nbt_tag_list_valid() {
        // List tag: tag_id = 1 (Byte), length = 2
        let mut data = Vec::new();
        data.push(1u8); // List element tag_id = Byte
        data.extend(&2i32.to_be_bytes()); // length = 2
        let cursor = make_cursor(data);
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::List).unwrap();
        assert_eq!(result, nbt::NbtTagType::List((nbt::NbtTagId::Byte, 2)));
    }

    #[test]
    fn test_nbt_tag_list_invalid_tag_id() -> Result<(), nbt::NbtReadError> {
        // List tag: invalid element tag_id
        let mut data = Vec::new();
        data.push(13u8); // Invalid tag_id
        data.extend(&2i32.to_be_bytes()); // length = 2
        let cursor = make_cursor(data);
        let mut cursor = cursor;
        assert!(nbt_tag(&mut cursor, &nbt::NbtTagId::List).is_err());

        Ok(())
    }

    #[test]
    fn test_nbt_tag_list_io_error_tag_id() {
        // List tag: missing tag_id
        let mut data = Vec::new();
        // No tag_id
        data.extend(&2i32.to_be_bytes()); // length = 2
        let cursor = make_cursor(data);
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::List);
        assert!(matches!(result, Err(nbt::NbtReadError::Io(_))));
    }

    #[test]
    fn test_nbt_tag_list_panic_large_length_defined() -> Result<(), nbt::NbtReadError>{
        // List tag with length > 65536
        let mut data = Vec::new();
        data.push(1u8); // List element tag_id = Byte
        
        let bad_list_len = 65_537i32;
        let high_byte = (bad_list_len >> 8) as u8; // Get the higher 8 bits
        let low_byte = (bad_list_len & 0xFF) as u8; // Get the lower 8 bits
        data.push(high_byte);
        data.push(low_byte);
        data.extend(&(65_537i32).to_be_bytes()); // length = 65_537
        
        let cursor = make_cursor(data);
        let mut cursor = cursor;
        assert!(nbt_tag(&mut cursor, &nbt::NbtTagId::List).is_err());

        Ok(())
    }

    #[test]
    fn test_nbt_tag_list_panic_large_length_real() -> Result<(), nbt::NbtReadError>{
        // List tag with length > 65536
        let mut data = Vec::new();
        data.push(1u8); // List element tag_id = Byte
        
        let bad_list_len = 65_530i32; // List len is defined smaller than 65536
        let high_byte = (bad_list_len >> 8) as u8; // Get the higher 8 bits
        let low_byte = (bad_list_len & 0xFF) as u8; // Get the lower 8 bits
        data.push(high_byte);
        data.push(low_byte);
        //but the real list length is 65_537
        data.extend(&(65_537i32).to_be_bytes()); // length = 65_537
        
        let cursor = make_cursor(data);
        let mut cursor = cursor;
        assert!(nbt_tag(&mut cursor, &nbt::NbtTagId::List).is_err());

        Ok(())
    }

    #[test]
    fn test_nbt_tag_compound() {
        let cursor = make_cursor(vec![]); // No data needed
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::Compound).unwrap();
        assert_eq!(result, nbt::NbtTagType::Compound("".to_string()));
    }

    #[test]
    fn test_nbt_tag_int_array() {
        // IntArray with length = 3 and values [1, 2, 3]
        let mut data = Vec::new();
        data.extend(&3i32.to_be_bytes()); // length = 3
        data.extend(&1i32.to_be_bytes());
        data.extend(&2i32.to_be_bytes());
        data.extend(&3i32.to_be_bytes());
        let cursor = make_cursor(data);
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::IntArray).unwrap();
        assert_eq!(result, nbt::NbtTagType::IntArray(vec![1, 2, 3]));
    }

    /*     #[test]
    fn test_nbt_tag_int_array_panic_large_length() {
        let mut data = Vec::new();
        data.extend(&(65_537i32).to_be_bytes()); // length = 65_537
        let cursor = make_cursor(data);
        let mut cursor = cursor;
        // Expect panic due to large length
        // Note: Since the function will panic before reading data, no data bytes are needed
        // However, to trigger the panic, we need to call the function
        // Use the `should_panic` attribute
        // But Rust's #[should_panic] cannot be used here, so we need to use std::panic::catch_unwind
        let result = std::panic::catch_unwind(|| {
            nbt_tag(&mut cursor, &nbt::NbtTagId::IntArray).unwrap();
        });
        assert!(result.is_err());
    } */

    #[test]
    fn test_nbt_tag_int_array_io_error_length() {
        let cursor = make_cursor(vec![0x00, 0x00, 0x00]); // Incomplete i32 for length
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::IntArray);
        assert!(matches!(result, Err(nbt::NbtReadError::Io(_))));
    }

    #[test]
    fn test_nbt_tag_int_array_io_error_data() {
        // IntArray with length = 3 but only 2 integers provided
        let mut data = Vec::new();
        data.extend(&3i32.to_be_bytes()); // length = 3
        data.extend(&1i32.to_be_bytes());
        data.extend(&2i32.to_be_bytes());
        // Missing the third integer
        let cursor = make_cursor(data);
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::IntArray);
        assert!(matches!(result, Err(nbt::NbtReadError::Io(_))));
    }

    #[test]
    fn test_nbt_tag_long_array() {
        // LongArray with length = 2 and values [1, 2]
        let mut data = Vec::new();
        data.extend(&2i32.to_be_bytes()); // length = 2
        data.extend(&1i64.to_be_bytes());
        data.extend(&2i64.to_be_bytes());
        let cursor = make_cursor(data);
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::LongArray).unwrap();
        assert_eq!(result, nbt::NbtTagType::LongArray(vec![1, 2]));
    }

    /* #[test]
    fn test_nbt_tag_long_array_panic_large_length() {
        let mut data = Vec::new();
        data.extend(&(65_537i32).to_be_bytes()); // length = 65_537
        let cursor = make_cursor(data);
        let mut cursor = cursor;
        let result = std::panic::catch_unwind(|| {
            nbt_tag(&mut cursor, &nbt::NbtTagId::LongArray).unwrap();
        });
        assert!(result.is_err());
    } */

    #[test]
    fn test_nbt_tag_long_array_io_error_length() {
        let cursor = make_cursor(vec![0x00, 0x00, 0x00]); // Incomplete i32 for length
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::LongArray);
        assert!(matches!(result, Err(nbt::NbtReadError::Io(_))));
    }

    #[test]
    fn test_nbt_tag_long_array_io_error_data() {
        // LongArray with length = 2 but only 1 long provided
        let mut data = Vec::new();
        data.extend(&2i32.to_be_bytes()); // length = 2
        data.extend(&1i64.to_be_bytes());
        // Missing the second long
        let cursor = make_cursor(data);
        let mut cursor = cursor;
        let result = nbt_tag(&mut cursor, &nbt::NbtTagId::LongArray);
        assert!(matches!(result, Err(nbt::NbtReadError::Io(_))));
    }
}
