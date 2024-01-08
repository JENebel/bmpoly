use std::collections::HashMap;

use bevy::{asset::{Handle, Asset, AssetApp}, reflect::TypePath, app::{App, Plugin}};

use crate::{polygon::Polygon, border_segment::BorderSegment};

pub struct ProvincePlugin;

impl Plugin for ProvincePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Province>();
    }
}

pub struct ProvinceMap {
    map: HashMap<u32, Handle<Province>>,
}

#[derive(Asset, TypePath)]
pub struct Province {
    id: u32,
    polygons: Vec<Polygon>,
    border_segments: Vec<BorderSegment>,
    neighbors: Vec<u32>,
}