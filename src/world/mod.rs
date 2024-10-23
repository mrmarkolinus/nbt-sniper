use crate::nbt;
use crate::region;
use std::path::{Path, PathBuf};
use log::{info, error};

const REGION_PATH: &str = "region";
const NETHER_PATH: &str = "DIM-1";
const END_PATH: &str = "DIM1";

enum DimensionType {
    Overworld,
    Nether,
    End,
}

pub struct World {
    path: PathBuf,
    dimensions: Vec<Dimension>,
}

struct Dimension {
    location: DimensionType,
    region_files: Vec<region::RegionFile>,
}

impl World {
    pub fn read(path: PathBuf) -> World {
        
        let (world_dimensions,valid_minecraft_world) = Self::read_world_directory(path.clone());

        World {
            path,
            dimensions: world_dimensions,
        }
    }

    fn read_world_directory(world_path: PathBuf) -> (Vec<Dimension>, bool) {
        let mut valid_minecraft_world = true;

        let mut overworld_regions = Vec::<region::RegionFile>::new();
        let mut nether_regions = Vec::<region::RegionFile>::new();
        let mut end_regions = Vec::<region::RegionFile>::new();

        if !world_path.is_dir() {
            valid_minecraft_world = false
        } else {
            let overworld_region_path = world_path.join(REGION_PATH);
            let nether_region_path = world_path.join(NETHER_PATH).join(REGION_PATH);
            let end_region_path = world_path.join(END_PATH).join(REGION_PATH);

            if !overworld_region_path.is_dir() {
                valid_minecraft_world = false;
            } else {
                Self::read_region_directory(overworld_region_path, &mut overworld_regions);
            }
            if !nether_region_path.is_dir() {
                valid_minecraft_world = false;
            } else {
                Self::read_region_directory(nether_region_path, &mut nether_regions);
            }
            if !end_region_path.is_dir() {
                valid_minecraft_world = false;
            } else {
                Self::read_region_directory(end_region_path, &mut end_regions);
            }
        }

        let dimensions = vec![
            Dimension {
                location: DimensionType::Overworld,
                region_files: overworld_regions,
            },
            Dimension {
                location: DimensionType::Nether,
                region_files: nether_regions,
            },
            Dimension {
                location: DimensionType::End,
                region_files: end_regions,
            },
        ];

        (dimensions,valid_minecraft_world)
    }

    fn read_region_directory(region_path: PathBuf, regions: &mut Vec<region::RegionFile>) -> std::io::Result<()> {

        match std::fs::read_dir(region_path) {
            Ok(entries) => {
                for entry in entries {
                    if let Ok(entry) = entry {
                        info!("Reading region file {entry:?}");
                        regions.push(region::RegionFile::read(entry.path()));     
                    }
                }
            },
            Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Error in reading the region files")),
        }
        Ok(())
    }
}
