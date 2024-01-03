use std::collections::{HashMap, HashSet};

use bevy::input::mouse;
use bevy::prelude::*;
use bevy::render::mesh::{self, PrimitiveTopology};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Direction {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

use crate::Direction::*;

impl Direction {
    fn from_int(dir: usize) -> Direction {
        match dir {
            0 => Up,
            1 => Down,
            2 => Left,
            3 => Right,
            _ => panic!("Invalid direction")
        }
    }

    fn apply_offset(&self, pixel: (f32, f32)) -> (f32, f32) {
        match self {
            Up => (pixel.0, pixel.1 + 0.5),
            Down => (pixel.0, pixel.1 - 0.5),
            Left => (pixel.0 - 0.5, pixel.1),
            Right => (pixel.0 + 0.5, pixel.1),
        }
    }

    fn apply_corner_offset(corner: (Self, Self), pixel: (f32, f32)) -> (f32, f32) {
        match corner {
            (Up, Right) => (pixel.0 + 0.5, pixel.1 + 0.5),
            (Up, Left) => (pixel.0 - 0.5, pixel.1 + 0.5),
            (Down, Right) => (pixel.0 + 0.5, pixel.1 - 0.5),
            (Down, Left) => (pixel.0 - 0.5, pixel.1 - 0.5),

            (Right, Up) => (pixel.0 + 0.5, pixel.1 + 0.5),
            (Right, Down) => (pixel.0 + 0.5, pixel.1 - 0.5),
            (Left, Up) => (pixel.0 - 0.5, pixel.1 + 0.5),
            (Left, Down) => (pixel.0 - 0.5, pixel.1 - 0.5),

            _ => panic!("Invalid corner")
        }
    }

    fn rotate_left(&self) -> Direction {
        match self {
            Up => Left,
            Down => Right,
            Left => Down,
            Right => Up,
        }
    }

    fn add_direction(&self, pixel: (usize, usize)) -> Option<(usize, usize)> {
        match self {
            Up => Some((pixel.0, pixel.1 + 1)),
            Down => if pixel.1 == 0 { None } else { Some((pixel.0, pixel.1 - 1)) },
            Left => if pixel.0 == 0 { None } else { Some((pixel.0 - 1, pixel.1)) },
            Right => Some((pixel.0 + 1, pixel.1)),
        }
    }

    fn add_half_direction(&self, pos: (f32, f32)) -> (f32, f32) {
        match self {
            Up => (pos.0 as f32, pos.1 as f32 + 0.5),
            Down => (pos.0 as f32, pos.1 as f32 - 0.5),
            Left => (pos.0 as f32 - 0.5, pos.1 as f32),
            Right => (pos.0 as f32 + 0.5, pos.1 as f32),
        }
    }
}

struct BorderMap {
    borders: Vec<Vec<((u8, u8, u8), [bool; 4])>>,
    //border_pixels: HashMap<(usize, usize), bool>,
}

impl BorderMap {
    fn new(width: usize, height: usize) -> BorderMap {
        BorderMap {
            borders: vec![vec![((0, 0, 0), [false; 4]); height]; width]
        }
    }

    fn get(&self, x: usize, y: usize, dir: Direction) -> bool {
        if x >= self.borders.len() || y >= self.borders[x].len() {
            return false;
        }
        self.borders[x][y].1[dir as usize]
    }

    fn get_clr(&self, x: usize, y: usize) -> (u8, u8, u8) {
        self.borders[x][y].0
    }

    fn remove(&mut self, x: usize, y: usize, dir: Direction) {
        self.borders[x][y].1[dir as usize] = false;
    }

    fn insert(&mut self, x: usize, y: usize, dir: Direction, clr: (u8, u8, u8)) {
        self.borders[x][y].0 = clr;
        self.borders[x][y].1[dir as usize] = true;
    }

    fn get_some_starting_point(&self) -> Option<(usize, usize, Direction)> {
        for x in 0..self.borders.len() {
            for y in 0..self.borders[x].len() {
                for dir in 0..4 {
                    if self.borders[x][y].1[dir] {
                        return Some((x, y, Direction::from_int(dir)))
                    }
                }
            }
        }
        None
    }
                                                                                        // x,    y,     dir,        vertex,          corner
    fn pop_next_border(&mut self, (x, y): (usize, usize), from_dir: Direction) -> Option<(usize, usize, Direction, (f32, f32), Option<(f32, f32)>)> {
        let rot_left = from_dir.rotate_left();

        println!("Checking {}, {} from {:?}", x, y, from_dir);

        // Left turn
        if self.get(x, y, rot_left) {
            self.remove(x, y, rot_left);

            let edge = rot_left.add_half_direction((x as f32, y as f32));
            
            // Check for sharp corner
            /*let (lx, ly) = rot_left.add_direction((x, y));
            let (ux, uy) = from_dir.add_direction((x, y));
            if self.get_clr(lx, ly) != self.get_clr(ux, uy) {
                let mut corner = from_dir.add_half_direction((x as f32, y as f32));
                corner = rot_left.add_half_direction(corner);
                return Some((x, y, rot_left, edge, Some(corner)))
            }*/
            return Some((x, y, rot_left, edge, None))
        }

        // Straight hori/vert line.
        if let Some((sx, sy)) = rot_left.add_direction((x, y)) {
            if self.get(sx, sy, rot_left) {
                self.remove(sx, sy, rot_left);

                let edge = rot_left.add_half_direction((x as f32, y as f32));
                let edge = rot_left.add_half_direction(edge);
                let edge = from_dir.add_half_direction(edge);

                return Some((sx, sy, rot_left, edge, None))
            }
        }

        // Right turn
        if let Some((rx, ry)) = rot_left.add_direction((x, y)) {
            if self.get(rx, ry, from_dir) {
                self.remove(rx, ry, from_dir);

                let edge = rot_left.add_half_direction((x as f32, y as f32));
                let edge = from_dir.add_half_direction(edge);
                let edge = from_dir.add_half_direction(edge);

                return Some((rx, ry, from_dir, edge, None))
            }
        }

        println!("No border found");
        None
    }
}

fn load_polygons() -> Vec<Polygon> {
    let img = bmp::open("test.bmp").unwrap();
    
    println!("Image dimensions: {}x{}", img.get_width(), img.get_height());

    let (width, height) = (img.get_width(), img.get_height());

    let mut borders = BorderMap::new(width as usize, height as usize);

    let time_before = std::time::Instant::now();

    for (x, y) in img.coordinates() {
        let y = height - y - 1;
        let color = img.get_pixel(x, y);
        if x == 0 || img.get_pixel(x - 1, y) != color {
            borders.insert(x as usize, y as usize, Direction::Left, (color.r, color.g, color.b))
        }
        if x == width - 1 || img.get_pixel(x + 1, y) != color {
            borders.insert(x as usize, y as usize, Direction::Right, (color.r, color.g, color.b))
        }
        if y == 0 || img.get_pixel(x, y - 1) != color {
            borders.insert(x as usize, y as usize, Direction::Up, (color.r, color.g, color.b))
        }
        if y == height - 1 || img.get_pixel(x, y + 1) != color {
            borders.insert(x as usize, y as usize, Direction::Down, (color.r, color.g, color.b))
        }
    }

    let mut polygons = Vec::new();

    while let Some((ox, oy, odir)) = borders.get_some_starting_point() {
        let mut vertices: Vec<(f32, f32)> = Vec::new();
        let color = borders.get_clr(ox, oy);
        borders.remove(ox, oy, odir);

        println!("Starting at {}, {} with direction {:?}", ox, oy, odir);

        let mut dir = odir;
        let (mut x, mut y) = (ox, oy);
        while let Some((nx, ny, ndir, edge, corner)) = borders.pop_next_border((x, y), dir) {
            if let Some(corner) = corner {
                vertices.push((corner.0, corner.1));
            }
            vertices.push((edge.0, edge.1));
            println!("{}, {}", edge.0, edge.1);

            borders.remove(nx, ny, ndir);

            if ox == nx && oy == ny && odir == ndir {
                break;
            }

            dir = ndir;
            (x, y) = (nx, ny)
        }

        println!("Finished polygon");

        polygons.push(Polygon::new(Color::rgb(color.0 as f32 / 255., color.1 as f32 / 255., color.2 as f32 / 255.), vertices));
    }

    println!("Finished in {}ms", time_before.elapsed().as_millis());

    polygons
}

struct Polygon {
    color: Color,
    verticies: Vec<[f32; 3]>,
    indicies: Vec<u32>,
}

impl Polygon {
    fn new(color: Color, verticies: Vec<(f32, f32)>) -> Polygon {
        let raw_verticies: Vec<f32> = verticies.iter().flat_map(|(x, y)| vec![*x, *y]).collect();
        let verticies: Vec<[f32; 3]> = verticies.into_iter().map(|(x, y)| [x, y, 0.]).collect();

        let triangles = earcutr::earcut(&raw_verticies, &[], 2).unwrap();

        /*for i in 0..points.len() {
            println!("{}: {:?}", i, points[i]);
        }
        println!("{:?}", triangles);*/

        let mut edge_indicies: Vec<u32> = (0..verticies.len() as u32).collect();
        edge_indicies.push(0);

        Polygon {
            verticies,
            color,
            indicies: triangles.into_iter().map(|i| i as u32).collect()
        }
    }
}

#[derive(Resource, Default)]
struct Polygons {
    handle: Handle<Mesh>,
    polygons: Vec<Polygon>,
}

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .insert_resource(Polygons::default())
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Startup, meshes)
        .run();
}

fn setup (mut commands: Commands, _: ResMut<Assets<Mesh>>, _: ResMut<Assets<StandardMaterial>>) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 1000.,
            ..default()
        },
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(4., 2., 6.),
        ..default()
    });
}

fn meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mouse_input: Res<Input<MouseButton>>,
   // mut polygons: ResMut<Polygons>,
) {
    /*if polygons.polygons.len() != 0 && !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }

    if polygons.polygons.len() != 0 {
        meshes.remove(&polygons.handle);
        meshes.set_changed();
    }*/

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

        let handle = meshes.add(mesh);
        commands.spawn(PbrBundle {
            mesh: handle.clone(),
            material: materials.add(poly.color.into()),
            ..default()
        });
    }

    /*polygons.handle = handle;
    polygons.polygons = polys;*/

    println!("Reloaded mesh")
}