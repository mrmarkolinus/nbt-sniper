use crate::nbt;
use crate::region;
use std::path::PathBuf;

const REGION_PATH: &str = "region";
const NETHER_PATH: &str = "DIM-1";
const END_PATH: &str = "DIM1";

enum DimensionType {
    Overworld,
    Nether,
    End,
}

struct World {
    path: PathBuf,
    dimensions: Vec<Dimension>,
}

struct Dimension {
    location: DimensionType,
    region_files: Vec<region::RegionFile>,
}

impl World {
    pub fn read(path: PathBuf) -> World {
        World {
            path,
            dimensions: vec![],
        }
    }

    pub fn read_world_directory(world_path: PathBuf) -> bool {
        let mut valid_minecraft_world = true;

        if !world_path.is_dir() {
            valid_minecraft_world = false
        } else {
            let overworld_region_path = world_path.join(REGION_PATH);
            let nether_region_path = world_path.join(NETHER_PATH).join(REGION_PATH);
            let end_region_path = world_path.join(END_PATH).join(REGION_PATH);

            if !overworld_region_path.is_dir() {
                valid_minecraft_world = false;
            }
            if !nether_region_path.is_dir() {
                valid_minecraft_world = false;
            }
            if !end_region_path.is_dir() {
                valid_minecraft_world = false;
            }
        }
        valid_minecraft_world
    }
}
