use flate2::read::GzDecoder;
use std::fs;
use std::io::BufReader;
use std::io::Read;
use serde_json;
use std::fs::File;


pub mod nbt;

pub struct MinecraftBinary {
    file_path: String,
    nbtdata: nbt::NbtData
}

impl MinecraftBinary {
    pub fn new(file_path: String) -> Self {
        let buffer = Self::read_file(&file_path).unwrap();
        let nbtdata = nbt::NbtData::from_buf(buffer).unwrap();
        MinecraftBinary { file_path, nbtdata}  
    }

    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    pub fn nbtdata(&self) -> &nbt::NbtData {
        &self.nbtdata
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
            if nbttag.position().depth() > 0 {
                print!("|");  
            }
            for i in 0..nbttag.position().depth() {
                print!("___");
            }
            Self::display_tag(nbttag);
            println!();
        }
    
    }

    fn display_tag(nbttag: &nbt::NbtTag) {

        let nbttag_value = nbttag.value();
        let tag_name = nbttag.name();

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
            nbt::NbtTagType::Compound(x) => print!("Compound - {}: {:?} - ", tag_name, x),
            nbt::NbtTagType::IntArray(x) => print!("IntArray - {}: {:?}", tag_name, x),
            nbt::NbtTagType::LongArray(x) => print!("LongArray - {}: {:?}", tag_name, x),
        }

        Self::display_raw_values(&nbttag.position());
    }

    fn display_raw_values(position : &nbt::NbtTagPosition) {
        print!("Raw Bytes: ");
        print!("ID[{}:{}] ", position.byte_start_id(), position.byte_end_id());
        print!("Name[{}:{}] ", position.byte_start_name(), position.byte_end_name());
        print!("Value[{}:{}] ", position.byte_start_value(), position.byte_end_value());
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