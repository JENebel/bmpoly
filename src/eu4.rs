use std::fs;

use bevy::utils::hashbrown::HashMap;

use crate::polygon::*;

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
        let mut color = poly.source_color;
        if let Some(terrain) = colors.get(&color) {
            match terrain {
                TerrainType::Sea => {
                    color = (80, 252, 252);
                },
                TerrainType::Lake => {
                    color = (80, 252, 252);
                },
                TerrainType::Land => {
                    color = (50, 140, 64);
                },
            }
        } else {
            color = (50, 140, 64);
        }
        poly.source_color = color;
    }
}