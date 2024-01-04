mod polygon;
use bevy::sprite::MaterialMesh2dBundle;
use bevy_pancam::{PanCam, PanCamPlugin};
use polygon::load_polygons;

use bevy::prelude::*;
use bevy::render::mesh::{self, PrimitiveTopology};

const FILL: bool = true;
const OUTLINE: bool = false;
const VERTICES: bool = false;

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
    let before = std::time::Instant::now();
    let polys = load_polygons("assets/provinces.bmp");

    let mut vertices = 0;

    let before_meshes = std::time::Instant::now();
    for poly in polys {
        if FILL {
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_POSITION,
                poly.vertices.clone(),
            );
            vertices += poly.vertices.len();
            mesh.set_indices(Some(mesh::Indices::U32(poly.indicies.clone())));
            mesh.duplicate_vertices();
            mesh.compute_flat_normals();

            let mat = materials.add(Color::rgb(poly.color.0, poly.color.1, poly.color.2).into());
            commands.spawn(MaterialMesh2dBundle {
                mesh: meshes.add(Mesh::from(mesh)).into(),
                material: mat,
                ..default()
            });
        }

        if OUTLINE {
            let mut mesh = Mesh::new(PrimitiveTopology::LineStrip);
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_POSITION,
                poly.border_vertices.clone(),
            );
            let mut indices: Vec<u32> = (0..poly.border_vertices.len() as u32).collect();
            indices.push(0);
            mesh.set_indices(Some(mesh::Indices::U32(indices)));
            if FILL == false { vertices += poly.border_vertices.len() }
            mesh.duplicate_vertices();

            let mat = materials.add(Color::GRAY.into());
            commands.spawn(MaterialMesh2dBundle {
                mesh: meshes.add(Mesh::from(mesh)).into(),
                material: mat,
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                ..default()
            });
        }

        if VERTICES {
            for vertex in poly.border_vertices {
                commands.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::RED,
                        custom_size: Some(Vec2::new(0.3, 0.3)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(vertex[0], vertex[1], 2.)),
                    ..default()
                });
            }
        }
    }

    println!("Created meshes in {}ms", before_meshes.elapsed().as_millis());

    println!("Total time: {}ms", before.elapsed().as_millis());
    println!("Total vertices: {}", vertices);
    
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