use nbtsniper::NbtFile;

#[test]
fn bigtest() {
    let mc_bin = NbtFile::read("tests/files/bigtest.nbt".to_string());
    
    println!("-------------------------");
    println!("Debug Output");
    mc_bin.format_output();
    
    println!("");
    println!("-------------------------");
    println!("Printing JSON to file");
    mc_bin.to_json("tests/files/output/out_bigtest.json");

    println!("");
    println!("-------------------------");
    println!("Raw data using dump_hex() method");
    mc_bin.dump_hex();

    println!("");
    println!("-------------------------");
    println!("Iterate NbtTags");
    mc_bin.nbt_tags().iter().for_each(|x| println!("{:?}", x));

    println!("");
    println!("-------------------------");
    println!("Raw data using as_bytes() method");
    println!("{:?}\n", mc_bin.as_bytes());
}
