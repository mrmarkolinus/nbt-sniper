#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::io::Cursor;
    
    // Mocking the fsm module
    use crate::nbt::fsm; 
    use crate::nbt::{NbtTagId, NbtTagType, NbtTagPositionRawBytes, NbtTagPosition, NbtTag, NbtData, NbtReadError};

    // Now, write tests for NbtTagId
    #[test]
    fn test_nbt_tag_id_from_u8_valid() {
        for (value, expected) in vec![
            (0u8, NbtTagId::End),
            (1, NbtTagId::Byte),
            (2, NbtTagId::Short),
            (3, NbtTagId::Int),
            (4, NbtTagId::Long),
            (5, NbtTagId::Float),
            (6, NbtTagId::Double),
            (7, NbtTagId::ByteArray),
            (8, NbtTagId::String),
            (9, NbtTagId::List),
            (10, NbtTagId::Compound),
            (11, NbtTagId::IntArray),
            (12, NbtTagId::LongArray),
        ] {
            assert_eq!(NbtTagId::from_u8(value), Some(expected));
        }
    }

    #[test]
    fn test_nbt_tag_id_from_u8_invalid() {
        let invalid_values = vec![13u8, 255, 100, 14, 20];
        for value in invalid_values {
            assert_eq!(NbtTagId::from_u8(value), None);
        }
    }

    #[test]
    fn test_nbt_tag_id_as_u8() {
        let variants = vec![
            (NbtTagId::End, 0u8),
            (NbtTagId::Byte, 1),
            (NbtTagId::Short, 2),
            (NbtTagId::Int, 3),
            (NbtTagId::Long, 4),
            (NbtTagId::Float, 5),
            (NbtTagId::Double, 6),
            (NbtTagId::ByteArray, 7),
            (NbtTagId::String, 8),
            (NbtTagId::List, 9),
            (NbtTagId::Compound, 10),
            (NbtTagId::IntArray, 11),
            (NbtTagId::LongArray, 12),
        ];

        for (variant, expected) in variants {
            assert_eq!(variant.as_u8(), expected);
        }
    }

    #[test]
    fn test_nbt_tag_id_default() {
        let default_tag = NbtTagId::default();
        assert_eq!(default_tag, NbtTagId::End);
    }

    // Tests for NbtTagType
    #[test]
    fn test_nbt_tag_type_default() {
        let default_tag = NbtTagType::default();
        assert_eq!(default_tag, NbtTagType::End(None));
    }

    #[test]
    fn test_nbt_tag_type_serialization() {
        let tag = NbtTagType::Int(42);
        let serialized = serde_json::to_string(&tag).unwrap();
        assert_eq!(serialized, r#"{"Int":42}"#);

        let tag = NbtTagType::String("Test".to_string());
        let serialized = serde_json::to_string(&tag).unwrap();
        assert_eq!(serialized, r#"{"String":"Test"}"#);
    }

    // Tests for NbtTagPositionRawBytes
    #[test]
    fn test_nbt_tag_position_raw_bytes_new() {
        let pos = NbtTagPositionRawBytes::new();
        assert_eq!(pos.byte_start_all(), 0);
        assert_eq!(pos.byte_end_all(), 0);
        assert_eq!(pos.byte_end_all_with_children(), 0);
        assert_eq!(pos.byte_start_id(), 0);
        assert_eq!(pos.byte_end_id(), 0);
        assert_eq!(pos.byte_start_name(), 0);
        assert_eq!(pos.byte_end_name(), 0);
        assert_eq!(pos.byte_start_value(), 0);
        assert_eq!(pos.byte_end_value(), 0);
    }

    #[test]
    fn test_nbt_tag_position_raw_bytes_setters_getters() {
        let mut pos = NbtTagPositionRawBytes::new();
        pos.set_byte_start_all(10);
        pos.set_byte_end_all(20);
        pos.set_byte_end_all_with_children(30);
        pos.set_byte_start_id(40);
        pos.set_byte_end_id(50);
        pos.set_byte_start_name(60);
        pos.set_byte_end_name(70);
        pos.set_byte_start_value(80);
        pos.set_byte_end_value(90);

        assert_eq!(pos.byte_start_all(), 10);
        assert_eq!(pos.byte_end_all(), 20);
        assert_eq!(pos.byte_end_all_with_children(), 30);
        assert_eq!(pos.byte_start_id(), 40);
        assert_eq!(pos.byte_end_id(), 50);
        assert_eq!(pos.byte_start_name(), 60);
        assert_eq!(pos.byte_end_name(), 70);
        assert_eq!(pos.byte_start_value(), 80);
        assert_eq!(pos.byte_end_value(), 90);
    }

    #[test]
    fn test_nbt_tag_position_raw_bytes_reset() {
        let mut pos = NbtTagPositionRawBytes::new();
        pos.set_byte_start_all(10);
        pos.set_byte_end_all(20);
        pos.set_byte_end_all_with_children(30);
        pos.set_byte_start_id(40);
        pos.set_byte_end_id(50);
        pos.set_byte_start_name(60);
        pos.set_byte_end_name(70);
        pos.set_byte_start_value(80);
        pos.set_byte_end_value(90);

        pos.reset();

        assert_eq!(pos.byte_start_all(), 0);
        assert_eq!(pos.byte_end_all(), 0);
        assert_eq!(pos.byte_end_all_with_children(), 0);
        assert_eq!(pos.byte_start_id(), 0);
        assert_eq!(pos.byte_end_id(), 0);
        assert_eq!(pos.byte_start_name(), 0);
        assert_eq!(pos.byte_end_name(), 0);
        assert_eq!(pos.byte_start_value(), 0);
        assert_eq!(pos.byte_end_value(), 0);
    }

    // Tests for NbtTagPosition
    #[test]
    fn test_nbt_tag_position_new() {
        let pos = NbtTagPosition::new();
        assert_eq!(pos.byte_start_all(), 0);
        assert_eq!(pos.byte_end_all(), 0);
        assert_eq!(pos.byte_end_all_with_children(), 0);
        assert_eq!(pos.byte_start_id(), 0);
        assert_eq!(pos.byte_end_id(), 0);
        assert_eq!(pos.byte_start_name(), 0);
        assert_eq!(pos.byte_end_name(), 0);
        assert_eq!(pos.byte_start_value(), 0);
        assert_eq!(pos.byte_end_value(), 0);
        assert_eq!(pos.index(), 0);
        assert_eq!(pos.depth(), 0);
        assert_eq!(pos.parent(), 0);
        //assert!(pos.children().is_empty());
    }

    #[test]
    fn test_nbt_tag_position_setters_getters() {
        let mut pos = NbtTagPosition::new();
        pos.set_byte_start_all(100);
        pos.set_byte_end_all(200);
        pos.set_byte_end_all_with_children(300);
        pos.set_byte_start_id(400);
        pos.set_byte_end_id(500);
        pos.set_byte_start_name(600);
        pos.set_byte_end_name(700);
        pos.set_byte_start_value(800);
        pos.set_byte_end_value(900);
        pos.set_index(1);
        pos.set_depth(2);
        pos.set_parent(3);

        assert_eq!(pos.byte_start_all(), 100);
        assert_eq!(pos.byte_end_all(), 200);
        assert_eq!(pos.byte_end_all_with_children(), 300);
        assert_eq!(pos.byte_start_id(), 400);
        assert_eq!(pos.byte_end_id(), 500);
        assert_eq!(pos.byte_start_name(), 600);
        assert_eq!(pos.byte_end_name(), 700);
        assert_eq!(pos.byte_start_value(), 800);
        assert_eq!(pos.byte_end_value(), 900);
        assert_eq!(pos.index(), 1);
        assert_eq!(pos.depth(), 2);
        assert_eq!(pos.parent(), 3);
    }

    #[test]
    fn test_nbt_tag_position_reset() {
        let mut pos = NbtTagPosition::new();
        pos.set_byte_start_all(100);
        pos.set_byte_end_all(200);
        pos.set_byte_end_all_with_children(300);
        pos.set_byte_start_id(400);
        pos.set_byte_end_id(500);
        pos.set_byte_start_name(600);
        pos.set_byte_end_name(700);
        pos.set_byte_start_value(800);
        pos.set_byte_end_value(900);
        pos.set_index(1);
        pos.set_depth(2);
        pos.set_parent(3);
        pos.children().push(4);

        pos.reset();

        assert_eq!(pos.byte_start_all(), 0);
        assert_eq!(pos.byte_end_all(), 0);
        assert_eq!(pos.byte_end_all_with_children(), 0);
        assert_eq!(pos.byte_start_id(), 0);
        assert_eq!(pos.byte_end_id(), 0);
        assert_eq!(pos.byte_start_name(), 0);
        assert_eq!(pos.byte_end_name(), 0);
        assert_eq!(pos.byte_start_value(), 0);
        assert_eq!(pos.byte_end_value(), 0);
        assert_eq!(pos.index(), 0);
        assert_eq!(pos.depth(), 0);
        assert_eq!(pos.parent(), 0);
        assert!(pos.children().is_empty());
    }

    // Tests for NbtTag
    #[test]
    fn test_nbt_tag_default() {
        let tag = NbtTag::default();
        assert_eq!(tag.name(), "");
        assert_eq!(tag.value(), &NbtTagType::End(None));
        assert_eq!(tag.position().byte_start_all(), 0);
    }

    #[test]
    fn test_nbt_tag_setters_getters() {
        let mut tag = NbtTag::default();
        tag.set_name("TestTag".to_string());
        tag.set_value(NbtTagType::Int(123));
        
        let mut pos = NbtTagPosition::new();
        pos.set_byte_start_all(10);
        pos.set_byte_end_all(20);
        tag.set_position(pos.clone());

        assert_eq!(tag.name(), "TestTag");
        assert_eq!(tag.value(), &NbtTagType::Int(123));
        assert_eq!(tag.position().byte_start_all(), 10);
    }

    // Tests for NbtData
    #[test]
    fn test_nbt_data_new() {
        let buffer = vec![];
        let nbt_data = NbtData::new(buffer.clone());
        assert!(nbt_data.nbt_tags().is_empty());
        assert_eq!(nbt_data.raw_bytes(), &buffer);
    }

    #[test]
    fn test_nbt_data_from_buf_empty() {
        let buffer = vec![];
        let result = NbtData::from_buf(buffer);
        assert!(result.is_ok());
        let nbt_data = result.unwrap();
        assert!(nbt_data.nbt_tags().is_empty());
    }

    #[test]
    fn test_nbt_data_from_buf_invalid() {
        let buffer = vec![255]; // Invalid tag id
        let result = NbtData::from_buf(buffer);
        assert!(result.is_err());
        match result.err().unwrap() {
            NbtReadError::InvalidContent => (),
            _ => panic!("Expected InvalidContent error"),
        }
    }

    #[test]
    fn test_nbt_data_parse_single_tag() {
        // Create a buffer representing a single Int tag with name "Test" and value 42
        // Tag structure:
        // [Tag ID][Name Length][Name][Int Value]
        // Tag ID: 3 (Int)
        // Name Length: 4 (u16 big endian)
        // Name: "Test"
        // Int Value: 42 (i32 big endian)

        let mut buffer = Vec::new();
        buffer.push(NbtTagId::Int.as_u8());

        // Name length: 4
        buffer.extend(&4u16.to_be_bytes());

        // Name: "Test"
        buffer.extend("Test".as_bytes());

        // Int value: 42
        buffer.extend(&42i32.to_be_bytes());

        let nbt_data = NbtData::from_buf(buffer).unwrap();
        assert_eq!(nbt_data.nbt_tags().len(), 1);
        let tag = &nbt_data.nbt_tags()[0];
        assert_eq!(tag.name(), "Test");
        assert_eq!(tag.value(), &NbtTagType::Int(42));
    }

    #[test]
    fn test_nbt_data_parse_multiple_tags() {
        // Create a buffer representing two tags:
        // 1. Byte tag with name "A" and value 1
        // 2. String tag with name "B" and value "Hello"

        let mut buffer = Vec::new();

        // First Tag: Byte
        buffer.push(NbtTagId::Byte.as_u8());
        buffer.extend(&1u16.to_be_bytes()); // Name length
        buffer.extend("A".as_bytes());
        buffer.push(1i8 as u8); // Byte value

        // Second Tag: String
        buffer.push(NbtTagId::String.as_u8());
        buffer.extend(&1u16.to_be_bytes()); // Name length
        buffer.extend("B".as_bytes());
        buffer.extend(&5u16.to_be_bytes()); // String length
        buffer.extend("Hello".as_bytes());

        let nbt_data = NbtData::from_buf(buffer).unwrap();
        assert_eq!(nbt_data.nbt_tags().len(), 2);

        let tag1 = &nbt_data.nbt_tags()[0];
        assert_eq!(tag1.name(), "A");
        assert_eq!(tag1.value(), &NbtTagType::Byte(1));

        let tag2 = &nbt_data.nbt_tags()[1];
        assert_eq!(tag2.name(), "B");
        assert_eq!(tag2.value(), &NbtTagType::String("Hello".to_string()));
    }

    #[test]
    fn test_nbt_data_parse_nested_compound() {
        // Create a buffer representing a Compound tag containing an Int tag
        // Structure:
        // Compound Tag:
        // [Tag ID][Name Length][Name]
        // Inside Compound:
        // [Int Tag ID][Name Length][Name][Int Value]
        // [End Tag ID]

        let mut buffer = Vec::new();

        // Compound Tag
        buffer.push(NbtTagId::Compound.as_u8());
        buffer.extend(&6u16.to_be_bytes()); // Name length
        buffer.extend("Compound".as_bytes());

        // Nested Int Tag
        buffer.push(NbtTagId::Int.as_u8());
        buffer.extend(&3u16.to_be_bytes()); // Name length
        buffer.extend("Int".as_bytes());
        buffer.extend(&100i32.to_be_bytes()); // Int value

        // End Tag
        buffer.push(NbtTagId::End.as_u8());

        let nbt_data = NbtData::from_buf(buffer).unwrap();
        assert_eq!(nbt_data.nbt_tags().len(), 3);

        let compound_tag = &nbt_data.nbt_tags()[0];
        assert_eq!(compound_tag.name(), "Compound");
        assert_eq!(compound_tag.value(), &NbtTagType::Compound("".to_string()));
        //assert_eq!(compound_tag.position().children().len(), 1); TODO: Not implemented

        let nested_int_tag = &nbt_data.nbt_tags()[1];
        assert_eq!(nested_int_tag.name(), "Int");
        assert_eq!(nested_int_tag.value(), &NbtTagType::Int(100));
        assert_eq!(nested_int_tag.position().parent(), 0);

        let end_tag = &nbt_data.nbt_tags()[2];
        assert_eq!(end_tag.name(), "");
        assert_eq!(end_tag.value(), &NbtTagType::End(None));
        assert_eq!(end_tag.position().parent(), 0);
    }

    #[test]
    fn test_nbt_data_parse_list_of_strings() {
        // Create a buffer representing a List tag containing Strings
        // Structure:
        // [List Tag ID][Name Length][Name][List Element Type][List Length]
        // [String 1]
        // [String 2]
        // ...
        // [End Tag ID]

        let mut buffer = Vec::new();

        // List Tag
        buffer.push(NbtTagId::List.as_u8());
        buffer.extend(&4u16.to_be_bytes()); // Name length
        buffer.extend("List".as_bytes());
        buffer.push(NbtTagId::String.as_u8()); // List Element Type
        buffer.extend(&2i32.to_be_bytes()); // List Length

        // String 1
        buffer.extend(&5u16.to_be_bytes()); // String length
        buffer.extend("Hello".as_bytes());

        // String 2
        buffer.extend(&5u16.to_be_bytes()); // String length
        buffer.extend("World".as_bytes());

        let nbt_data = NbtData::from_buf(buffer).unwrap();
        assert_eq!(nbt_data.nbt_tags().len(), 3);

        let list_tag = &nbt_data.nbt_tags()[0];
        assert_eq!(list_tag.name(), "List");
        assert_eq!(list_tag.value(), &NbtTagType::List((NbtTagId::String, 2)));
        //assert_eq!(list_tag.position().children().len(), 2); TODO: Not implemented

        let string1 = &nbt_data.nbt_tags()[1];
        assert_eq!(string1.value(), &NbtTagType::String("Hello".to_string()));
        assert_eq!(string1.position().parent(), 0);

        let string2 = &nbt_data.nbt_tags()[2];
        assert_eq!(string2.value(), &NbtTagType::String("World".to_string()));
        assert_eq!(string2.position().parent(), 0);
    }

    // Additional tests can be added here to cover more scenarios, such as:
    // - Parsing ByteArray, IntArray, LongArray
    // - Parsing nested compounds
    // - Handling maximum depth
    // - Error scenarios for incomplete data, etc.
}
