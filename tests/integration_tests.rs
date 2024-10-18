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
    print!("{}\n", mc_bin.hex_dump());

    println!("");
    println!("-------------------------");
    println!("Iterate NbtTags");
    mc_bin.nbt_tags().iter().for_each(|x| println!("{:?}", x));

    println!("");
    println!("-------------------------");
    println!("Print Hashmap of NbtTags");
    println!("{:?}\n", mc_bin.nbt_hashmap());
    println!("Search a NbtTag");
    if let Some(&nbt_index) = mc_bin.nbt_hashmap().get("doubleTest") {
        println!("doubleTest: {:?}", mc_bin.nbt_tags()[nbt_index]);
    }

    println!("");
    println!("-------------------------");
    println!("Raw data using as_bytes() method");
    println!("{:?}\n", mc_bin.as_bytes());
}
