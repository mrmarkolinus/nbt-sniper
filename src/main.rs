use flate2::read::GzDecoder;
use std::fs;
use std::io::BufReader;
use std::io::Read;
use std::io::Cursor;


pub mod nbt;

fn main() {
    let buffer = read_file("files/bigtest.nbt").unwrap();
    let mut cursor = Cursor::new(buffer);

    let test_tag_sequence = nbt::NbtTag::parse_from_buf(&mut cursor).unwrap();
    //let mut test_tag = nbt::NbtTag::parse_from_buf(&mut cursor).unwrap();
    
    let test_tag = test_tag_sequence.nbt_tags();

    test_tag.iter().for_each(|x| println!("{:?}", x));

    //test_tag.iter().for_each(|x| println!("{:?}", x.byte_start()));


/*     let mut tag_index = 0;
    for elem in test_tag {
        println!("{:?}", elem);
        println!("Tag index: {}", tag_index);
        tag_index += 1;
    } */
}

fn read_file(file_path: &str) -> std::io::Result<Vec<u8>> {
        
    // Open the file and create a buffered reader for efficient reading
    let file = fs::File::open(file_path)?;
    
    let mut buf_reader = BufReader::new(file);
    let mut decoder = GzDecoder::new(buf_reader);
    let mut decompressed_data = Vec::new();

    decoder.read_to_end(&mut decompressed_data)?;
    Ok(decompressed_data)
}
