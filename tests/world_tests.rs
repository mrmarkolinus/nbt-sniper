use std::path::PathBuf;
use nbtsniper::world;
use log::{debug, error, log_enabled, info, Level};


#[test]
fn read_world_hermitcraft_s9() {
    env_logger::init();

    let world_s9 = world::World::read(PathBuf::from("C:/MultiMC/MultiMC/instances/Hermitcraft S9/.minecraft/saves/hermitcraft9"));

    println!("Did it!");
}
