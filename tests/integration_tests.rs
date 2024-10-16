use nbt_sniper::MinecraftBinary;

#[test]
fn bigtest() {
    let mc_bin = MinecraftBinary::read("tests/files/bigtest.nbt".to_string());
    mc_bin.format_output();
    mc_bin.to_json();
}
