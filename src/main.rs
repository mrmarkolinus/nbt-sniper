use flate2::read::GzDecoder;
use nbt::NbtData;
use std::fs;
use std::io::BufReader;
use std::io::Read;
use std::io::Cursor;


pub mod nbt;

fn main() {
    let buffer = read_file("files/bigtest.nbt").unwrap();

    let nbtdata = nbt::NbtData::from_buf(buffer).unwrap();
    let test_tag = nbtdata.nbt_tags();
    
    //test_tag.iter().for_each(|x| println!("{:?}", x));
    //test_tag.iter().for_each(|x| format_tag(x));
    format_output(&nbtdata);
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
