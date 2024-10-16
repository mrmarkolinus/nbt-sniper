use nbt_sniper::NbtFile;

#[test]
fn bigtest() {
    let mc_bin = NbtFile::read("tests/files/bigtest.nbt".to_string());
    mc_bin.format_output();
    mc_bin.to_json("tests/files/output/out_bigtest.json");
}
