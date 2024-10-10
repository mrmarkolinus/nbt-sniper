use flate2::read::GzDecoder;
use std::fs;
use std::io::BufReader;
use std::io::Read;
use std::io::Cursor;


pub mod nbt;

fn main() {
    let buffer = read_file("files/bigtest.nbt").unwrap();
    let mut cursor = Cursor::new(buffer);

    let test_tag_sequence = nbt::NbtTagSequence::from_buf(&mut cursor).unwrap();
    
    let test_tag = test_tag_sequence.nbt_tags();

    test_tag.iter().for_each(|x| println!("{:?}", x));
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
