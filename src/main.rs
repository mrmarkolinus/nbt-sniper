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
    
    test_tag.iter().for_each(|x| println!("{:?}", x));
    //test_tag.iter().for_each(|x| format_tag(x));
    format_output_raw(&nbtdata);
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
    
    for tag in nbtdata.nbt_tags() {
        format_tag(tag);
    }

}

fn format_output_raw(nbtdata: &nbt::NbtData) {
    
    for (i, byte) in nbtdata.raw_bytes().iter().enumerate() {
        // Print a space every 4 bytes for grouping
        if i % 4 == 0 && i % 16 != 0 {
            print!(" ");
        }
        // Print a new line every 16 bytes
        if i % 16 == 0 && i != 0 {
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
