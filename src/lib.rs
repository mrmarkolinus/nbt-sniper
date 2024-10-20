use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::io::BufReader;
use std::io::Read;
use thiserror::Error;

pub mod nbt;

#[derive(Error, Debug)]
pub enum NbtFileError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error), // Automatically convert `io::Error` to `NbtReadError`

    #[error("Json could not be created")]
    JsonWriteFailure, // Custom error for content validation
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct NbtFile {
    file_path: String,
    nbtdata: nbt::NbtData,
}

impl NbtFile {
    pub fn new() -> Self {
        NbtFile::default()
    }

    pub fn read(file_path: String) -> Self {
        let buffer = Self::read_file(&file_path).unwrap();
        let nbtdata = nbt::NbtData::from_buf(buffer).unwrap();
        NbtFile { file_path, nbtdata }
    }

    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    pub fn nbt_tags(&self) -> &Vec<nbt::NbtTag> {
        &self.nbtdata.nbt_tags()
    }

    pub fn as_bytes(&self) -> &Vec<u8> {
        self.nbtdata.raw_bytes()
    }

    pub fn nbt_hashmap(&self) -> &HashMap<String, usize> {
        &self.nbtdata.tags_map()
    }

    fn read_file(file_path: &str) -> std::io::Result<Vec<u8>> {
        // Open the file and create a buffered reader for efficient reading
        let file = fs::File::open(file_path)?;

        let buf_reader = BufReader::new(file);
        let mut decoder = GzDecoder::new(buf_reader);
        let mut decompressed_data = Vec::new();

        decoder.read_to_end(&mut decompressed_data)?;
        Ok(decompressed_data)
    }

    pub fn format_output(&self) {
        for nbttag in self.nbtdata.nbt_tags() {
            for _ in 0..nbttag.position().depth() {
                print!("   ");
            }
            Self::display_tag(nbttag, &self.nbtdata.raw_bytes());

            println!();
        }
    }

    pub fn hex_dump(&self) -> String {
        Self::format_output_raw(self.nbtdata.raw_bytes(), 0)
    }

    fn display_tag(nbttag: &nbt::NbtTag, rawbytes: &Vec<u8>) {
        let nbttag_value = nbttag.value();
        let tag_name = nbttag.name();

        match nbttag_value {
            nbt::NbtTagType::End(_) => print!("End - {}", tag_name),
            nbt::NbtTagType::Byte(x) => print!("{}[Byte]: {}", tag_name, x),
            nbt::NbtTagType::Short(x) => print!("{}[Short]: {}", tag_name, x),
            nbt::NbtTagType::Int(x) => print!("{}[Int]: {}", tag_name, x),
            nbt::NbtTagType::Long(x) => print!("{}[Long]: {}", tag_name, x),
            nbt::NbtTagType::Float(x) => print!("{}[Float]: {}", tag_name, x),
            nbt::NbtTagType::Double(x) => print!("{}[Double]: {}", tag_name, x),
            nbt::NbtTagType::ByteArray(_) => {
                print!("{}[ByteArray]: [Values... see dump]", tag_name)
            }
            nbt::NbtTagType::String(x) => print!("{}[String]: {:?}", tag_name, x),
            nbt::NbtTagType::List(x) => print!("{}[List]: {:?}", tag_name, x),
            nbt::NbtTagType::Compound(_) => print!("{}[Compound]: ", tag_name),
            nbt::NbtTagType::IntArray(_) => print!("{}[IntArray]: [Values... see dump]", tag_name),
            nbt::NbtTagType::LongArray(_) => {
                print!("{}[LongArray]: [Values... see dump]", tag_name)
            }
        }
        println!("");

        Self::display_raw_values(nbttag, rawbytes);
    }

    fn display_raw_values(nbttag: &nbt::NbtTag, rawbytes: &Vec<u8>) {
        for _ in 0..nbttag.position().depth() {
            print!("   ");
        }
        print!("Raw Bytes: ");
        print!(
            "ID[{}:{}] ",
            match nbttag.position().byte_start_id() {
                Some(x) => x.to_string(),
                None => "N/A".to_string(),
            },
            match nbttag.position().byte_end_id() {
                Some(x) => x.to_string(),
                None => "N/A".to_string(),
            }
        );
        print!(
            "Name[{}:{}] ",
            match nbttag.position().byte_start_name() {
                Some(x) => x.to_string(),
                None => "N/A".to_string(),
            },
            match nbttag.position().byte_end_name() {
                Some(x) => x.to_string(),
                None => "N/A".to_string(),
            }
        );

        let mut byte_start = 0;
        let mut byte_end = 0;
        let byte_start_dump;
        let byte_end_dump;

        match nbttag.value() {
            nbt::NbtTagType::Compound(_x) => {
                if let Some(x) = nbttag.position().byte_start_value() {
                    byte_start = x;
                }
                byte_end = nbttag.position().byte_end_all_with_children();
                byte_start_dump = nbttag.position().byte_start_all();
                byte_end_dump = nbttag.position().byte_end_all();

                println!("Value[{}:{}]", byte_start, byte_end);
            }
            nbt::NbtTagType::List(_x) => {
                if let Some(x) = nbttag.position().byte_start_value() {
                    byte_start = x;
                }
                byte_end = nbttag.position().byte_end_all_with_children();
                byte_start_dump = nbttag.position().byte_start_all();
                byte_end_dump = nbttag.position().byte_end_all();

                println!("Value[{}:{}]", byte_start, byte_end);
            }
            nbt::NbtTagType::End(_x) => {
                byte_start_dump = nbttag.position().byte_start_all();
                byte_end_dump = nbttag.position().byte_end_all();

                println!("Value[{}]", "N/A");
            }
            _ => {
                if let Some(x) = nbttag.position().byte_start_value() {
                    byte_start = x;
                }

                if let Some(x) = nbttag.position().byte_end_value() {
                    byte_end = x;
                }

                byte_start_dump = nbttag.position().byte_start_all();
                byte_end_dump = nbttag.position().byte_end_all();

                println!("Value[{}:{}]", byte_start, byte_end);
            }
        }

        let dump_hex = &rawbytes[byte_start_dump..byte_end_dump];

        for _ in 0..nbttag.position().depth() {
            print!("   ");
        }
        println!("Hex Dump[{}:{}]", byte_start_dump, byte_end_dump);
        Self::format_output_raw(dump_hex, nbttag.position().depth());
    }

    fn format_output_raw(rawbytes: &[u8], depth: i64) -> String {
        let mut output = String::new();

        for _ in 0..depth {
            _ = write!(output, "   ");
        }

        for i in 0..rawbytes.len() {
            let byte = rawbytes[i];
            // Print a space every 4 bytes for grouping
            if i % 4 == 0 && i % 32 != 0 {
                _ = write!(output, " ");
            }
            // Print a new line every 32 bytes
            if i % 32 == 0 && i != 0 {
                _ = writeln!(output);
                for _ in 0..depth {
                    _ = writeln!(output, "   ");
                }
            }
            // Print the byte as hex
            _ = write!(output, "{:02X} ", byte);
        }
        // Print a final new line
        _ = writeln!(output);

        output
    }

    pub fn to_json(&self, output_path: &str) -> Result<(), NbtFileError> {
        let file = fs::File::create(output_path)?;
        serde_json::to_writer_pretty(file, self.nbtdata.nbt_tags())
            .map_err(|_| NbtFileError::JsonWriteFailure)?;
        Ok(())
    }
}
/*
fn main() {
    let buffer = read_file("files/bigtest.nbt").unwrap();

    let nbtdata = nbt::NbtData::from_buf(buffer).unwrap();
    let test_tag = nbtdata.nbt_tags();

    //test_tag.iter().for_each(|x| println!("{:?}", x));
    //test_tag.iter().for_each(|x| format_tag(x));
    //format_output(&nbtdata);
    output_json(&nbtdata);
}

fn read_file(file_path: &str) -> std::io::Result<Vec<u8>> {

    // Open the file and create a buffered reader for efficient reading
    let file = fs::File::open(file_path)?;

    let buf_reader = BufReader::new(file);
    let mut decoder = GzDecoder::new(buf_reader);
    let mut decompressed_data = Vec::new();

    decoder.read_to_end(&mut decompressed_data)?;
    Ok(decompressed_data)
}

fn format_output(nbtdata: &nbt::NbtData) {

    for nbttag in nbtdata.nbt_tags() {
        if nbttag.depth() > 0 {
            print!("|");
        }
        for i in 0..nbttag.depth() {
            print!("___");
        }
        display_tag(nbttag.value(), nbttag.name());
        println!();
    }

}

fn output_json(nbtdata: &nbt::NbtData) {
    // Convert the Vec to a JSON string
    //let json_output = serde_json::to_string_pretty(&nbtdata.nbt_tags()).unwrap();

    // Print the JSON
    //println!("{}", json_output);

    // Open (or create) a file to write to
    let file_path = "output.json";
    let file = File::create(file_path).expect("Impossible to create file");

    // Write the JSON to the file
    serde_json::to_writer_pretty(file, &nbtdata.nbt_tags()).unwrap();
}

fn format_output_raw(nbtdata: &nbt::NbtData) {

    let nbt_tags = nbtdata.nbt_tags();
    let mut nbt_index = 0;
    for (i, byte) in nbtdata.raw_bytes().iter().enumerate() {
        // Print a space every 4 bytes for grouping
        if i % 4 == 0 && i % 16 != 0 {
            print!(" ");
        }
        // Print a new line every 16 bytes
        if i % 16 == 0 && i != 0 {
            print!(" | ");
            //print!("{}", nbt_tags[nbt_index].name());

            println!();
        }
        // Print the byte as hex
        print!("{:02X} ", byte);
    }
    // Print a final new line
    println!();

}

fn format_tag(tag: &nbt::NbtTag) {
    for i in 0..tag.depth() {
        print!("-");
    }
    print!(">");
    println!("{}", tag.name());
}

fn display_tag(nbttag_value: &nbt::NbtTagType, tag_name: &str) {

    match nbttag_value {
        nbt::NbtTagType::End(_) => print!("End - {}", tag_name),
        nbt::NbtTagType::Byte(x) => print!("Byte - {}: {}", tag_name, x),
        nbt::NbtTagType::Short(x) => print!("Short - {}: {}", tag_name, x),
        nbt::NbtTagType::Int(x) => print!("Int - {}: {}", tag_name, x),
        nbt::NbtTagType::Long(x) => print!("Long - {}: {}", tag_name, x),
        nbt::NbtTagType::Float(x) => print!("Float - {}: {}", tag_name, x),
        nbt::NbtTagType::Double(x) => print!("Double - {}: {}", tag_name, x),
        nbt::NbtTagType::ByteArray(x) => print!("ByteArray - {}: {:?}", tag_name, x),
        nbt::NbtTagType::String(x) => print!("String - {}: {:?}", tag_name, x),
        nbt::NbtTagType::List(x) => print!("List - {}: {:?}", tag_name, x),
        nbt::NbtTagType::Compound(x) => print!("Compound - {}: {:?}", tag_name, x),
        nbt::NbtTagType::IntArray(x) => print!("IntArray - {}: {:?}", tag_name, x),
        nbt::NbtTagType::LongArray(x) => print!("LongArray - {}: {:?}", tag_name, x),
    }
}
 */
