use std::any::Any;

use nbtsniper::{nbt, NbtFile};

#[test]
fn bigtest() {
    let mc_bin = NbtFile::read("tests/files/bigtest.nbt".to_string());

    println!("-------------------------");
    println!("Debug Output");
    mc_bin.format_output();

    println!("");
    println!("-------------------------");
    println!("Printing JSON to file");
    mc_bin.to_json("tests/files/output/out_bigtest.json");

    println!("");
    println!("-------------------------");
    println!("Raw data using dump_hex() method");
    print!("{}\n", mc_bin.hex_dump());

    println!("");
    println!("-------------------------");
    println!("Iterate NbtTags");
    mc_bin.nbt_tags().iter().for_each(|x| println!("{:?}", x));

    println!("");
    println!("-------------------------");
    println!("Print Hashmap of NbtTags");
    println!("{:?}\n", mc_bin.nbt_hashmap());
    println!("Search a NbtTag");
    if let Some(&nbt_index) = mc_bin.nbt_hashmap().get("doubleTest") {
        println!("doubleTest: {:?}", mc_bin.nbt_tags()[nbt_index]);
    }

    println!("");
    println!("-------------------------");
    println!("Raw data using as_bytes() method");
    println!("{:?}\n", mc_bin.as_bytes());
}

#[test]
fn test_bigtest_nbt_tags_names() {
    let mc_bin = NbtFile::read("tests/files/bigtest.nbt".to_string());

    let nbt_names = vec![
        "Level",
            "longTest",
            "shortTest",
            "stringTest",
            "floatTest",
            "intTest",
            "nested compound test",
                "ham",
                    "name",
                    "value",
                    "",
                "egg",
                    "name",
                    "value",
                    "",
                "",
            "listTest (long)",
                "", 
                "",
                "",
                "",
                "",
            "listTest (compound)",
                "",
                    "name",
                    "created-on",
                    "",
                "",
                    "name",
                    "created-on",
                    "",
            "byteTest",
            "byteArrayTest (the first 1000 values of (n*n*255+n*7)%100, starting with n=0 (0, 62, 34, 16, 8, ...))",
            "doubleTest",
            "",
    ];

    for (i, nbttag) in mc_bin.nbt_tags().iter().enumerate() {
        println!("{}: {}", i, nbttag.name());
        assert_eq!(nbttag.name(), nbt_names[i]);
    }
}

#[test]
fn test_bigtest_nbt_tags_types() {
    let mc_bin = NbtFile::read("tests/files/bigtest.nbt".to_string());

    let nbt_tag_types = vec![
        nbt::NbtTagId::Compound,
        nbt::NbtTagId::Long,
        nbt::NbtTagId::Short,
        nbt::NbtTagId::String,
        nbt::NbtTagId::Float,
        nbt::NbtTagId::Int,
        nbt::NbtTagId::Compound,
        nbt::NbtTagId::Compound,
        nbt::NbtTagId::String,
        nbt::NbtTagId::Float,
        nbt::NbtTagId::End,
        nbt::NbtTagId::Compound,
        nbt::NbtTagId::String,
        nbt::NbtTagId::Float,
        nbt::NbtTagId::End,
        nbt::NbtTagId::End,
        nbt::NbtTagId::List,
        nbt::NbtTagId::Long,
        nbt::NbtTagId::Long,
        nbt::NbtTagId::Long,
        nbt::NbtTagId::Long,
        nbt::NbtTagId::Long,
        nbt::NbtTagId::List,
        nbt::NbtTagId::Compound,
        nbt::NbtTagId::String,
        nbt::NbtTagId::Long,
        nbt::NbtTagId::End,
        nbt::NbtTagId::Compound,
        nbt::NbtTagId::String,
        nbt::NbtTagId::Long,
        nbt::NbtTagId::End,
        nbt::NbtTagId::Byte,
        nbt::NbtTagId::ByteArray,
        nbt::NbtTagId::Double,
        nbt::NbtTagId::End,
    ];

    for (i, nbttag) in mc_bin.nbt_tags().iter().enumerate() {
        println!("{}: {}", i, nbttag.name());
        assert_eq!(nbttag.value().into_id(), nbt_tag_types[i]);
    }
}

#[test]
fn test_bigtest_start_end_bytes_are_continuous() {
    let mc_bin = NbtFile::read("tests/files/bigtest.nbt".to_string());
    let mut last_byte_position = -1;
    let mut current_byte_position = 0;

    /*     for nbttag in mc_bin.nbt_tags() {
        current_byte_position = nbttag.position().byte_start_all() as i64;
        assert_eq!(current_byte_position, last_byte_position + 1);
        last_byte_position = current_byte_position;
        current_byte_position
    } */
}
