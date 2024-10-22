use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
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

#[derive(Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct NbtFile {
    file_path: String,
    nbtdata: nbt::NbtData,
}

impl Debug for NbtFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for nbttag in self.nbtdata.nbt_tags() {
            for _ in 0..nbttag.position().depth() {
                write!(f, "\t")?;
            }

            let nbttag_value = nbttag.value();
            let tag_name = nbttag.name();

            match nbttag_value {
                nbt::NbtTagType::End(_) => write!(f, "End - {}", tag_name)?,
                nbt::NbtTagType::Byte(x) => write!(f, "{}[Byte]: {}", tag_name, x)?,
                nbt::NbtTagType::Short(x) => write!(f, "{}[Short]: {}", tag_name, x)?,
                nbt::NbtTagType::Int(x) => write!(f, "{}[Int]: {}", tag_name, x)?,
                nbt::NbtTagType::Long(x) => write!(f, "{}[Long]: {}", tag_name, x)?,
                nbt::NbtTagType::Float(x) => write!(f, "{}[Float]: {}", tag_name, x)?,
                nbt::NbtTagType::Double(x) => write!(f, "{}[Double]: {}", tag_name, x)?,
                nbt::NbtTagType::ByteArray(_) => {
                    write!(f, "{}[ByteArray]: [Values... see dump]", tag_name)?
                }
                nbt::NbtTagType::String(x) => write!(f, "{}[String]: {:?}", tag_name, x)?,
                nbt::NbtTagType::List(x) => write!(f, "{}[List]: {:?}", tag_name, x)?,
                nbt::NbtTagType::Compound(_) => write!(f, "{}[Compound]: ", tag_name)?,
                nbt::NbtTagType::IntArray(_) => {
                    write!(f, "{}[IntArray]: [Values... see dump]", tag_name)?
                }
                nbt::NbtTagType::LongArray(_) => {
                    write!(f, "{}[LongArray]: [Values... see dump]", tag_name)?
                }
            }
            writeln!(f)?;

            NbtFile::formatted_raw_values(f, nbttag, self.as_raw_bytes())?;

            writeln!(f)?;
        }

        Ok(())
    }
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

    pub fn as_raw_bytes(&self) -> &Vec<u8> {
        &self.nbtdata.raw_bytes()
    }

    pub fn nbt_hashmap(&self) -> &HashMap<String, usize> {
        &self.nbtdata.tags_map()
    }

    /*     pub fn format_output(&self) {
        for nbttag in self.nbtdata.nbt_tags() {
            for _ in 0..nbttag.position().depth() {
                print!("   ");
            }
            Self::display_tag(nbttag, &self.nbtdata.raw_bytes());

            println!();
        }
    } */

    /*     pub fn hex_dump(&self) -> String {
        let f: std::fmt::Formatter;
        _ = Self::formatted_raw_bytes(f,self.nbtdata.raw_bytes(), 0);
    } */

    fn read_file(file_path: &str) -> std::io::Result<Vec<u8>> {
        // Open the file and create a buffered reader for efficient reading
        let file = fs::File::open(file_path)?;

        let buf_reader = BufReader::new(file);
        let mut decoder = GzDecoder::new(buf_reader);
        let mut decompressed_data = Vec::new();

        decoder.read_to_end(&mut decompressed_data)?;
        Ok(decompressed_data)
    }

    fn formatted_raw_values(
        f: &mut std::fmt::Formatter,
        nbttag: &nbt::NbtTag,
        rawbytes: &Vec<u8>,
    ) -> std::fmt::Result {
        for _ in 0..nbttag.position().depth() {
            write!(f, "\t")?;
        }
        write!(f, "Raw Bytes: ")?;
        write!(
            f,
            "ID[{}:{}] ",
            match nbttag.position().byte_start_id() {
                Some(x) => x.to_string(),
                None => "N/A".to_string(),
            },
            match nbttag.position().byte_end_id() {
                Some(x) => x.to_string(),
                None => "N/A".to_string(),
            }
        )?;
        write!(
            f,
            "Name[{}:{}] ",
            match nbttag.position().byte_start_name() {
                Some(x) => x.to_string(),
                None => "N/A".to_string(),
            },
            match nbttag.position().byte_end_name() {
                Some(x) => x.to_string(),
                None => "N/A".to_string(),
            }
        )?;

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

                write!(f, "Value[{}:{}]", byte_start, byte_end)?;
            }
            nbt::NbtTagType::List(_x) => {
                if let Some(x) = nbttag.position().byte_start_value() {
                    byte_start = x;
                }
                byte_end = nbttag.position().byte_end_all_with_children();
                byte_start_dump = nbttag.position().byte_start_all();
                byte_end_dump = nbttag.position().byte_end_all();

                write!(f, "Value[{}:{}]", byte_start, byte_end)?;
            }
            nbt::NbtTagType::End(_x) => {
                byte_start_dump = nbttag.position().byte_start_all();
                byte_end_dump = nbttag.position().byte_end_all();

                write!(f, "Value[{}]", "N/A")?;
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

                write!(f, "Value[{}:{}]", byte_start, byte_end)?;
            }
        }

        let dump_hex = &rawbytes[byte_start_dump..byte_end_dump];

        writeln!(f)?;
        for _ in 0..nbttag.position().depth() {
            write!(f, "\t")?;
        }
        write!(f, "Hex Dump[{}:{}]", byte_start_dump, byte_end_dump)?;
        writeln!(f)?;
        Self::formatted_raw_bytes(f, dump_hex, nbttag.position().depth())?;

        Ok(())
    }

    fn formatted_raw_bytes(
        f: &mut std::fmt::Formatter,
        rawbytes: &[u8],
        depth: i64,
    ) -> std::fmt::Result {
        for _ in 0..depth {
            write!(f, "\t")?;
        }

        for i in 0..rawbytes.len() {
            let byte = rawbytes[i];
            // Print a space every 4 bytes for grouping
            if i % 4 == 0 && i % 32 != 0 {
                write!(f, " ")?;
            }
            // Print a new line every 32 bytes
            if i % 32 == 0 && i != 0 {
                writeln!(f)?;
                for _ in 0..depth {
                    write!(f, "\t")?;
                }
            }
            // Print the byte as hex
            write!(f, "{:02X} ", byte)?;
        }
        // Print a final new line
        write!(f, "\n")?;

        Ok(())
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
