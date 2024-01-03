mod polygon;
use bevy::sprite::MaterialMesh2dBundle;
use bevy_pancam::{PanCam, PanCamPlugin};
use polygon::load_polygons;

use bevy::prelude::*;
use bevy::render::mesh::{self, PrimitiveTopology};

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .add_plugins((DefaultPlugins, PanCamPlugin::default()))
        .add_systems(Startup, setup)
        .run();
}

fn setup (
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let polys = load_polygons();

    for poly in polys {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            poly.verticies.clone(),
        );
        mesh.set_indices(Some(mesh::Indices::U32(poly.indicies.clone())));
        mesh.duplicate_vertices();
        mesh.compute_flat_normals();

        let mat = materials.add(Color::rgb(poly.color.0, poly.color.1, poly.color.2).into());
        commands.spawn(MaterialMesh2dBundle {
            mesh: meshes.add(Mesh::from(mesh)).into(),
            material: mat,
            ..default()
        });
        /*commands.spawn(PbrBundle {
            mesh: handle.clone(),
            material: materials.add(Color::rgb(poly.color.0, poly.color.1, poly.color.2).into()),
            ..default()
        });*/
    }
    
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.,
            ..default()
        },
        ..default()
    });

    commands.spawn(Camera2dBundle::default())
    .insert(PanCam::default());
}