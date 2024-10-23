use std::path::PathBuf;
use nbtsniper::region;

#[test]
fn read_region_file() {
    let region_file = region::RegionFile::read(PathBuf::from("tests/files/inputs/r.-1.0.mca"));

    for chunk in region_file.chunks() {
        println!("-------------------------");
        println!("Offset: {:X}", chunk.offset());
        println!("Size: {}", chunk.size());
    }
}
