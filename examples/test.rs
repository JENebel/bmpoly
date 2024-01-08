use bevy::prelude::*;
use bevy_pancam::PanCamPlugin;
use bevy_polyline2d::Polyline2dPlugin;
use bmpoly::{province::ProvincePlugin, border_segment::*};


fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins)
        .add_plugins(PanCamPlugin::default())
        .add_plugins(Polyline2dPlugin)
        .add_plugins(ProvincePlugin)
        .add_plugins(BorderSegmentPlugin)

        .add_systems(Startup, setup)
        .add_systems(Update, button_system)
        .run();
}

fn setup(
    //mut commands: Commands,
    mut border_segments: ResMut<Assets<BorderSegment>>,
) {
    // Add border segment
    border_segments.add(BorderSegment {
        province_id: 1,
        neighbor_id: 2,
    });
}

fn button_system(
    border_segments: ResMut<Assets<BorderSegment>>,
) {
    println!("Border segments: {}", border_segments.len());
}