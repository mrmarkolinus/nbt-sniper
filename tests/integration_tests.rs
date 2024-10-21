use std::any::Any;

use nbtsniper::{nbt, NbtFile, AsRawBytes};

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
    println!("{:?}\n", mc_bin.as_raw_bytes());
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
                    "ListCompoundListTest",
                        "",
                            "F1",
                            "B1",
                            "",
                        "",
                            "56",
                            "42",
                            "ListCompoundListCompoundListTest",
                                "",
                                "",
                            "",
                        
                    "",
            "byteTest",
            "byteArrayTest (the first 1000 values of (n*n*255+n*7)%100, starting with n=0 (0, 62, 34, 16, 8, ...))",
            "doubleTest",
            "",
    ];

    for (ii, nbttag) in mc_bin.nbt_tags().iter().enumerate() {
        println!("{}: {}", ii, nbttag.name());
        assert_eq!(nbttag.name(), nbt_names[ii]);
    }
}

#[test]
fn test_bigtest_nbt_tags_types() {
    let mc_bin = NbtFile::read("tests/files/bigtest.nbt".to_string());

    let nbt_tag_types = vec![
        nbt::NbtTagId::Compound,                        // Level
            nbt::NbtTagId::Long,                            // longTest
            nbt::NbtTagId::Short,                           // shortTest
            nbt::NbtTagId::String,                          // stringTest
            nbt::NbtTagId::Float,                           // floatTest
            nbt::NbtTagId::Int,                             // intTest
            nbt::NbtTagId::Compound,                        // nested compound test
                nbt::NbtTagId::Compound,                        // ham
                    nbt::NbtTagId::String,                          // name
                    nbt::NbtTagId::Float,                           // value
                    nbt::NbtTagId::End,                             //
                nbt::NbtTagId::Compound,                        // egg
                    nbt::NbtTagId::String,                          // name
                    nbt::NbtTagId::Float,                           // value
                    nbt::NbtTagId::End,                             //
                nbt::NbtTagId::End,                             //
            nbt::NbtTagId::List,                            // listTest (long)
                nbt::NbtTagId::Long,                            //
                nbt::NbtTagId::Long,                            //
                nbt::NbtTagId::Long,                            //
                nbt::NbtTagId::Long,                            //
                nbt::NbtTagId::Long,                            //
            nbt::NbtTagId::List,                            // listTest (compound)
                nbt::NbtTagId::Compound,                        //
                    nbt::NbtTagId::String,                          // name
                    nbt::NbtTagId::Long,                            // created-on
                    nbt::NbtTagId::End,                             //
                nbt::NbtTagId::Compound,                        //
                    nbt::NbtTagId::String,                          // name
                    nbt::NbtTagId::Long,                            // created-on
                    nbt::NbtTagId::List,                            // ListCompoundListTest
                        nbt::NbtTagId::Compound,                        //
                            nbt::NbtTagId::Float,                           // F1
                            nbt::NbtTagId::Byte,                            // B1
                            nbt::NbtTagId::End,                             //
                        nbt::NbtTagId::Compound,                        //
                            nbt::NbtTagId::Int,                             // 56
                            nbt::NbtTagId::Long,                            // 42
                            nbt::NbtTagId::List,                            // ListCompoundListCompoundListTest
                                nbt::NbtTagId::ByteArray,                       //
                                nbt::NbtTagId::ByteArray,                       //       
                            nbt::NbtTagId::End,                             //
                    nbt::NbtTagId::End,                             //
            nbt::NbtTagId::Byte,                            // byteTest
            nbt::NbtTagId::ByteArray,                       // byteArrayTest (the first 1000 values of (n*n*255+n*7)%100, starting with n=0 (0, 62, 34, 16, 8, ...))
            nbt::NbtTagId::Double,                          // doubleTest
            nbt::NbtTagId::End,                             //
    ];

    for (ii, nbttag) in mc_bin.nbt_tags().iter().enumerate() {
        println!("{}: {}", ii, nbttag.name());
        assert_eq!(nbttag.value().into_id(), nbt_tag_types[ii]);
    }
}

#[test]
fn test_bigtest_nbt_tags_depth() {
    let mc_bin = NbtFile::read("tests/files/bigtest.nbt".to_string());

    let nbt_tag_types = vec![
        0,                              // Level
            1,                              // longTest
            1,                              // shortTest
            1,                              // stringTest
            1,                              // floatTest
            1,                              // intTest
            1,                              // nested compound test
                2,                              // ham
                    3,                              // name
                    3,                              // value
                    3,                              //
                2,                              // egg
                    3,                              // name
                    3,                              // value
                    3,                              //
                2,                              //
            1,                              // listTest (long)
                2,                              //
                2,                              //
                2,                              //
                2,                              //
                2,                              //
            1,                              // listTest (compound)
                2,                              //
                    3,                              // name
                    3,                              // created-on
                    3,                              //
                2,                              //
                    3,                              // name
                    3,                            // created-on
                    3,                            // ListCompoundListTest
                        4,                          //
                            5,                          // F1
                            5,                          // B1
                            5,                          //
                        4,                          //
                            5,                          // 56
                            5,                          // 42
                            5,                          // ListCompoundListCompoundListTest
                                6,                          //
                                6,                          //       
                            5,                          //
                    3,                          //
            1,                              // byteTest
            1,                              // byteArrayTest (the first 1000 values of (n*n*255+n*7)%100, starting with n=0 (0, 62, 34, 16, 8, ...))
            1,                              // doubleTest
            1,                              //
    ];

    for (ii, nbttag) in mc_bin.nbt_tags().iter().enumerate() {
        println!("{}: {}", ii, nbttag.position().depth());
        assert_eq!(nbttag.position().depth(), nbt_tag_types[ii]);
    }
}

#[test]
fn test_bigtest_start_end_bytes_are_continuous() {
    let mc_bin = NbtFile::read("tests/files/bigtest.nbt".to_string());
    //let mut last_byte_position = -1;
    let mut curr_pos;
    let mut next_pos;

    for (ii, _) in mc_bin.nbt_tags().iter().enumerate() {
        if ii + 1 == mc_bin.nbt_tags().len() {
            break;
        }

        curr_pos = mc_bin.nbt_tags()[ii].position().byte_end_all();
        next_pos = mc_bin.nbt_tags()[ii + 1].position().byte_start_all();
        assert_eq!(curr_pos + 1, next_pos);
    }
}
