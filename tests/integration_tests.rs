use nbt_sniper::MinecraftBinary;

#[test]
fn bigtest() {
    let mc_bin = MinecraftBinary::new("tests/files/bigtest.nbt".to_string());

    mc_bin.format_output();
}
