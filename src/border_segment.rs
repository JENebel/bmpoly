use bevy::prelude::*;

#[derive(Asset, TypePath, Resource)]
pub struct BorderSegment {
    pub province_id: u32,
    pub neighbor_id: u32
}

impl Default for BorderSegment {
    fn default() -> Self {
        Self {
            province_id: 0,
            neighbor_id: 0,
        }
    }
}

pub struct BorderSegmentPlugin;
impl Plugin for BorderSegmentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Assets<BorderSegment>>();
    }
}