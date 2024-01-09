use std::fs;

use bevy::{utils::hashbrown::HashMap, asset::Handle};

use crate::{polygon::*, SEA_MATERIAL_HANDLE, LAND_MATERIAL_HANDLE};

enum TerrainType {
    Sea,
    Lake,
    Land,
}

pub fn color_polys(polys: &mut Vec<Polygon>) {
    let seas: Vec<u32> = fs::read_to_string("seas.txt").unwrap().split_whitespace().map(|p| p.parse::<u32>().unwrap()).collect();
    let lakes: Vec<u32> = fs::read_to_string("lakes.txt").unwrap().split_whitespace().map(|p| p.parse::<u32>().unwrap()).collect();
    let mut colors: HashMap<(u8, u8, u8), TerrainType> = HashMap::new();

    let clr_str = fs::read_to_string("colors.txt").unwrap();
    let raw_colors = clr_str.split_whitespace();
    for color in raw_colors {
        if !color.chars().next().unwrap().is_numeric() {
            continue;
        }
        let mut color = color.split(";");
        let id = color.next().unwrap().parse::<u32>().unwrap();
        let r = color.next().unwrap().parse::<u8>().unwrap();
        let g = color.next().unwrap().parse::<u8>().unwrap();
        let b = color.next().unwrap().parse::<u8>().unwrap();
        
        let terrain = {
            if seas.contains(&id) {
                TerrainType::Sea
            } else if lakes.contains(&id) {
                TerrainType::Lake
            } else {
                TerrainType::Land
            }
        };
        colors.insert((r, g, b), terrain);
    }

    for poly in polys {
        let mut color = Handle::default();
        if let Some(terrain) = colors.get(&poly.source_color) {
            match terrain {
                TerrainType::Sea => {
                    color = SEA_MATERIAL_HANDLE;
                },
                TerrainType::Lake => {
                    color = SEA_MATERIAL_HANDLE;
                },
                TerrainType::Land => {
                    color = LAND_MATERIAL_HANDLE;
                },
            }
        } else {
            color = LAND_MATERIAL_HANDLE;
        }
        poly.mat_handle = color;
    }
}