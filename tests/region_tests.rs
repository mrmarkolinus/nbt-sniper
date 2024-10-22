use nbtsniper::region;


#[test]
fn read_region_file() {
    let region_file = region::RegionFile::read("tests/files/inputs/r.-1.0.mca".to_string());

    for chunk in region_file.chunks() {
        println!("-------------------------");
        println!("Offset: {}", chunk.offset());
        println!("Size: {}", chunk.size());
    }
}