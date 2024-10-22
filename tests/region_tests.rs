use nbtsniper::region;


#[test]
fn read_region_file() {
    let region_file = region::RegionFile::read("tests/files/region/r.0.0.mca".to_string());
}