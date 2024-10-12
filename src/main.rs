use flate2::read::GzDecoder;
use std::fs;
use std::io::BufReader;
use std::io::Read;
use std::io::Cursor;


pub mod nbt;

fn main() {
    let buffer = read_file("files/bigtest.nbt").unwrap();

    let test_tag_sequence = nbt::NbtData::from_buf(buffer).unwrap();
    
    let test_tag = test_tag_sequence.nbt_tags();
    
    test_tag.iter().for_each(|x| println!("{:?}", x));
    //test_tag.iter().for_each(|x| format_tag(x));
    //format_output(&test_tag, cursor);
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

fn format_output(nbt_tags: &Vec<nbt::NbtTag>, cursor: Cursor<Vec<u8>>) {
    
    nbt_tags.iter().for_each(|x| format_tag(x));

}

fn format_tag(tag: &nbt::NbtTag) {
    for i in 0..tag.depth() {
        print!("-");
    }
    print!(">");
    println!("{}", tag.name());
}
