use std::collections::HashMap;

use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::window::PrimaryWindow;
use bevy_mod_raycast::immediate::{Raycast, RaycastSettings, RaycastVisibility};
use bevy_mod_raycast::primitives::Ray3d;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_polyline2d::*;
use bmpoly::polygon::load_polygons;
use bmpoly::eu4::color_polys;
use bevy_debug_text_overlay::{screen_print, OverlayPlugin};

use bevy::prelude::*;
use bevy::render::mesh::{self, PrimitiveTopology};
use bmpoly::*;

const FILL: Visibility = Visibility::Visible;
const OUTLINE: Visibility = Visibility::Visible;
const VERTICES: bool = false;

// 70fps to beat
struct MaterialPlugin;
impl Plugin for MaterialPlugin {
    fn build(&self, app: &mut App) {
        app.world.resource_mut::<Assets<ColorMaterial>>().insert(LAND_MATERIAL_HANDLE, Color::rgb_u8(50, 140, 64).into());
        app.world.resource_mut::<Assets<ColorMaterial>>().insert(BORDER_MATERIAL_HANDLE, Color::GRAY.into());
        app.world.resource_mut::<Assets<ColorMaterial>>().insert(SELECTED_BORDER_MATERIAL_HANDLE, Color::RED.into());
        app.world.resource_mut::<Assets<ColorMaterial>>().insert(SEA_MATERIAL_HANDLE, Color::rgb_u8(80, 252, 252).into());

        app.world.resource_mut::<Assets<ColorMaterial>>().insert(SELECTED_PROV_MATERIAL_HANDLE, Color::rgb_u8(0, 0, 0).into());
    }
}

#[derive(Clone)]
struct RenderedPoly {
    base_mat: Handle<ColorMaterial>,
    _mesh: Handle<Mesh>,
    entity_id: Entity,
    border_ids: Vec<Entity>,
}

impl RenderedPoly {
    fn new(
        base_mat: Handle<ColorMaterial>,
        mesh: Handle<Mesh>,
        entity_id: Entity,
        border_ids: Vec<Entity>,
    ) -> Self {
        Self {
            base_mat,
            _mesh: mesh,
            entity_id,
            border_ids,
        }
    }
}

#[derive(Resource)]
struct PolyMap {
    map: HashMap<Entity, RenderedPoly>,
}

impl PolyMap {
    fn get(&self, id: &Entity) -> Option<RenderedPoly> {
        self.map.get(id).map(|rp| rp.clone())
    }
}

#[derive(Resource)]
struct Selected {
    rp: Option<RenderedPoly>,
}

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .insert_resource(PolyMap {
            map: HashMap::new(),
        })
        .insert_resource(Selected {
            rp: None,
        })
        .add_plugins((DefaultPlugins, PanCamPlugin::default(), Polyline2dPlugin, MaterialPlugin))

        .add_plugins(OverlayPlugin { font_size: 23.0, ..default() })
        .add_systems(Update, screen_print_text)

        .add_systems(Startup, setup)
        .add_systems(Update, click_system)
        .run();
}

#[derive(Component)]
struct PolyMesh;

fn setup (
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut poly_map: ResMut<PolyMap>,
) {
    let before = std::time::Instant::now();
    let img = bmp::open("assets/provinces.bmp").unwrap();
    let (width, height) = (img.get_width(), img.get_height());
    let mut polys = load_polygons(img);

    let mut total_entities = 0;

    color_polys(&mut polys);

    let mut vertices = 0;

    let before_meshes = std::time::Instant::now();
    for poly in polys.into_iter() {
        let base_mat = poly.mat_handle.clone();
        
        let (id, mesh) = {
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_POSITION,
                poly.vertices.clone(),
            );
            vertices += poly.vertices.len();
            mesh.set_indices(Some(mesh::Indices::U32(poly.indicies.clone())));
            mesh.duplicate_vertices();
            mesh.compute_flat_normals();

            let mesh = meshes.add(Mesh::from(mesh));
            let id = commands.spawn((
                MaterialMesh2dBundle {
                    mesh: mesh.clone().into(),
                    material: base_mat.clone(),
                    visibility: FILL,
                    ..default()
                },
                PolyMesh {},
            )).id();
            total_entities += 1;

            (id, mesh)
        };

        let mut border_ids = Vec::new();

        for border in poly.border_vertices.iter() {
            border_ids.push({
                let polyline = Polyline2d {
                    path: border.clone(),
                    closed: true,
                    width: 0.1,
                    line_placement: Align::Left,
                };
    
                total_entities += 1;
                commands.spawn(Polyline2dBundle {
                    polyline,
                    material: BORDER_MATERIAL_HANDLE.clone(),
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                    visibility: OUTLINE,
                    ..default()
                }).id()
            });
        }

        if VERTICES {
            for border in poly.border_vertices.iter() {
                for vertex in border {
                    commands.spawn(SpriteBundle {
                        sprite: Sprite {
                            color: Color::BLUE,
                            custom_size: Some(Vec2::new(0.3, 0.3)),
                            ..default()
                        },
                        transform: Transform::from_translation(Vec3::new(vertex[0], vertex[1], 2.)),
                        ..default()
                    });
                    total_entities += 1;
                }
            }
        }

        poly_map.map.insert(id, RenderedPoly::new(
            base_mat,
            mesh,
            id,
            border_ids,
        ));

        
    }

    println!("Created meshes in {}ms", before_meshes.elapsed().as_millis());

    println!("Total time: {}ms", before.elapsed().as_millis());
    println!("Total vertices: {}", vertices);
    println!("Total entities: {}", total_entities);
    
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.,
            ..default()
        },
        ..default()
    });

    commands.spawn(Camera2dBundle {
        transform: Transform::from_translation(Vec3::new(width as f32 / 1.9, height as f32 / 1.3, 1000.)),
        projection: OrthographicProjection {
            scale: 0.5,
            ..default()
        },
        ..default()
    })
    .insert(PanCam {
        grab_buttons: vec![MouseButton::Middle],
        ..default()
    });
}

fn click_system(
    buttons: Res<Input<MouseButton>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform)>,
    q_poly_mesh: Query<With<PolyMesh>>,

    mut materials: ResMut<Assets<ColorMaterial>>,
    poly_map: ResMut<PolyMap>,
    mut selected: ResMut<Selected>,
    mut raycast: Raycast,
    mut commands: Commands,
) {
    // If mouse button clicked
    if !(buttons.just_released(MouseButton::Left) || buttons.just_released(MouseButton::Right)) {
        return;
    }

    if let Some(rp) = &selected.rp {
        let mut entity = commands.entity(rp.entity_id);
        entity.insert(rp.base_mat.clone());
        for border_id in &rp.border_ids {
            let mut entity = commands.entity(*border_id);
            entity.insert(OUTLINE);
            entity.insert(BORDER_MATERIAL_HANDLE.clone());
        }
        selected.rp = None;
    }

    // If mouse button clicked
    if !buttons.just_released(MouseButton::Left) {
        return;
    }

    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    let loc = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate()).unwrap();

    let ray = Ray3d::new(Vec3::new(loc.x, loc.y, 100.), Vec3::new(0., 0., -1.));
    let hits = raycast.cast_ray(ray, 
        &RaycastSettings::default()
            .with_visibility(RaycastVisibility::MustBeVisibleAndInView)
            .always_early_exit()
            .with_filter(&|e| q_poly_mesh.contains(e))
        );

        // Select
    if let Some(hit) = hits.get(0).map(|h| h.0) {
        if let Some(rp) = poly_map.get(&hit) {
            let mut entity = commands.entity(hit);
            let base_color = materials.get(&rp.base_mat).unwrap().color;
            let selected_mat = materials.get_mut(&SELECTED_PROV_MATERIAL_HANDLE).unwrap();
            selected_mat.color = base_color * 0.75;
            entity.insert(SELECTED_PROV_MATERIAL_HANDLE.clone());

            for border_id in &rp.border_ids {
                let mut entity = commands.entity(*border_id);
                entity.insert(OUTLINE);
                entity.insert(SELECTED_BORDER_MATERIAL_HANDLE.clone());
            }

            selected.rp = Some(rp);
        }
    }
}

fn screen_print_text(
    time: Res<Time>,
    query_e: Query<&ViewVisibility, With<Mesh2dHandle>>,
    query_scale: Query<&OrthographicProjection, With<Camera>>,
) {
    let current_time = time.elapsed_seconds_f64();
    let at_interval = |t: f64| current_time % t < time.delta_seconds_f64();
    if at_interval(0.1) {
        let last_fps = 1.0 / time.delta_seconds();
        screen_print!(col: Color::RED, "FPS: {last_fps:.0}");

        let scale = query_scale.single().scale;
        screen_print!(col: Color::RED, "Scale: {}", scale);
    }
    if at_interval(0.25) {
        screen_print!(col: Color::RED, "Entites: {}", query_e.iter().filter(|e| ***e).count());
    }
}