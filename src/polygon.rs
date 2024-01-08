#[derive(Debug, Clone)]
pub struct Polygon {
    pub source_color: (u8, u8, u8),
    pub vertices: Vec<[f32; 3]>,
    pub border_vertices: Vec<Vec<[f32; 3]>>,
    pub indicies: Vec<u32>,
}

impl Polygon {
    fn new(color: (u8, u8, u8)) -> Self {
        Self {
            source_color: color,
            vertices: Vec::new(),
            border_vertices: Vec::new(),
            indicies: Vec::new(),
        }
    }

    fn extend(&mut self, other: Polygon) {
        let offset = self.vertices.len() as u32;
        self.vertices.extend(other.vertices);
        self.indicies.extend(other.indicies.into_iter().map(|i| i + offset));
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Direction {
    North = 0,
    South = 1,
    East = 2,
    West = 3,
}

use std::collections::{HashMap, VecDeque, HashSet};

use Direction::*;
use bmp::Image;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Position {
    x: usize,
    y: usize,
    dir: Direction,
}

impl Position {
    // Corner vertex forward/right
    fn corner_vertex(&self) -> (f32, f32) {
        let (x, y) = (self.x as f32, self.y as f32);
        match self.dir {
            North => (x + 0.5, y + 0.5),
            South => (x - 0.5, y - 0.5),
            West => (x - 0.5, y + 0.5),
            East => (x + 0.5, y - 0.5),
        }
    }

    fn vertex(&self) -> (f32, f32) {
        match self.dir {
            North => (self.x as f32, self.y as f32 + 0.5),
            South => (self.x as f32, self.y as f32 - 0.5),
            West => (self.x as f32 - 0.5, self.y as f32),
            East => (self.x as f32 + 0.5, self.y as f32),
        }
    }

    fn rotate_right(&self) -> Self {
        match self.dir {
            North => Position { x: self.x, y: self.y, dir: East },
            South => Position { x: self.x, y: self.y, dir: West },
            West => Position { x: self.x, y: self.y, dir: North },
            East => Position { x: self.x, y: self.y, dir: South },
        }
    }

    fn rotate_left(&self) -> Self {
        match self.dir {
            North => Position { x: self.x, y: self.y, dir: West },
            South => Position { x: self.x, y: self.y, dir: East },
            West => Position { x: self.x, y: self.y, dir: South },
            East => Position { x: self.x, y: self.y, dir: North },
        }
    }

    fn move_fwd(&self, (width, height): (usize, usize)) -> Option<Self> {
        match self.dir {
            North => if self.y == height - 1 { None } else { Some(Position { x: self.x, y: self.y + 1, dir: self.dir }) },
            South => if self.y == 0 { None } else { Some(Position { x: self.x, y: self.y - 1, dir: self.dir }) },
            West => if self.x == 0 { None } else { Some(Position { x: self.x - 1, y: self.y, dir: self.dir }) },
            East => if self.x == width - 1 { None } else { Some(Position { x: self.x + 1, y: self.y, dir: self.dir }) },
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Turn {
    Left,
    Right,
    Straight,
    Corner,
}

#[derive(Debug, Clone)]
struct BorderMap {
    colors: Vec<Vec<(u8, u8, u8)>>,
    borders: Vec<Vec<[bool; 4]>>,
    border_set: HashSet<Position>,
}

impl BorderMap {
    fn new(width: usize, height: usize) -> BorderMap {
        BorderMap {
            colors: vec![vec![(0, 0, 0); height]; width],
            borders: vec![vec![[false; 4]; height]; width],
            border_set: HashSet::new(),
        }
    }

    fn get(&self, pos: &Position) -> bool {
        self.borders[pos.x][pos.y][pos.dir as usize]
    }

    fn get_clr(&self, pos: &Position) -> (u8, u8, u8) {
        self.colors[pos.x][pos.y]
    }

    fn remove(&mut self, pos: &Position) {
        self.borders[pos.x][pos.y][pos.dir as usize] = false;
        self.border_set.remove(pos);
    }

    fn insert_clr(&mut self, (x, y): (usize, usize), clr: (u8, u8, u8)) {
        self.colors[x][y] = clr;
    }

    fn insert(&mut self, pos: &Position) {
        self.borders[pos.x][pos.y][pos.dir as usize] = true;
        self.border_set.insert(*pos);
    }
    
    fn get_some_starting_point(&mut self) -> Option<Position> {
        self.border_set.iter().next().copied()
    }
    
    fn pop_next_border(&mut self, pos: &Position) -> Option<(Position, (f32, f32), Option<(f32, f32)>, Turn)> {
        let dims = (self.borders.len(), self.borders[0].len());

        // Check left turn
        {
            let npos = pos.rotate_left();
            if self.get(&npos) && self.get_clr(&npos) == self.get_clr(pos) {
                // Check if there is a corner
                let is_corner = {
                    let fwd_pos = npos.move_fwd(dims);
                    let right_pos = npos.rotate_right().move_fwd(dims);

                    if right_pos == None || fwd_pos == None {
                        true
                    } else {
                        let fwd_pos = fwd_pos.unwrap();
                        let right_pos = right_pos.unwrap();
                        let fwd_right_pos = fwd_pos.rotate_right().move_fwd(dims).unwrap();

                        let dif_diag = self.get_clr(&fwd_pos) != self.get_clr(&right_pos);
                        let dif_corn = self.get_clr(&fwd_pos) == self.get_clr(&right_pos) && self.get_clr(&fwd_pos) != self.get_clr(&fwd_right_pos);
                        dif_diag || dif_corn
                    }
                };

                let corner = if is_corner {
                    //println!("{:?}: {:?} at {:?}", npos, self.get_clr(&npos), npos.corner_vertex());

                    Some(npos.corner_vertex())
                } else {
                    None
                }; 

                self.remove(&npos);
                return Some((npos, npos.vertex(), corner, Turn::Left));
            }
        }

        // Check forward
        {
            let npos = pos.rotate_left();
            if let Some(npos) = npos.move_fwd(dims) {
                let npos = npos.rotate_right();
                if self.get(&npos) && self.get_clr(&npos) == self.get_clr(pos) {
                    self.remove(&npos);
                    return Some((npos, npos.vertex(), None, Turn::Straight));
                }
            }
        }

        // Check right turn
        {
            if let Some(npos) = pos.move_fwd(dims) {
                let npos = npos.rotate_left();
                if let Some(npos) = npos.move_fwd(dims) {
                    let npos = npos.rotate_left();
                    let npos = npos.rotate_left();

                    let corner = pos.rotate_left().move_fwd(dims).unwrap();
                    if self.get(&npos) && self.get_clr(&npos) == self.get_clr(pos) && self.get_clr(&pos) == self.get_clr(&corner) {
                        self.remove(&npos);
                        return Some((npos, npos.vertex(), None, Turn::Right));
                    }
                }
            }
        }
        
        None
    }

    fn prune_vertices(vertices: &mut Vec<(f32, f32)>, queue: &mut VecDeque<Turn>) {
        // Straight lines
        if queue.len() >= 2 && queue[0] == Turn::Straight && queue[1] == Turn::Straight {
            vertices.pop();
        } // 986862 -> 746658

        // Diagonals
        if queue.len() >= 2 && queue[0] == Turn::Left && queue[1] == Turn::Right {
            vertices.pop();
        } 
        if queue.len() >= 2 && queue[0] == Turn::Right && queue[1] == Turn::Left {
            vertices.pop();
        } // 746658 -> 435539

        // Corners
        if queue.len() >= 3 && queue[0] == Turn::Corner && queue[2] == Turn::Straight {
            let corner = vertices.pop();
            vertices.pop();
            vertices.push(corner.unwrap());
        }
        if queue.len() >= 3 && queue[0] == Turn::Straight && queue[1] == Turn::Corner { // Corner added before Left turn
            vertices.pop();
        } // 435539 -> 417005

        // Maybe need a more complex algorithm for this. This is a bit of a hack
    }

    fn pop_polygon(&mut self) -> Option<(RawPolygon, (u8, u8, u8))> {
        let origin = match self.get_some_starting_point() {
            Some(p) => p,
            None => return None,
        };

        //println!("Starting at {:?}", origin);

        let mut vertices: Vec<(f32, f32)> = Vec::new();
        let color = self.get_clr(&origin);

        //println!("Starting at {:?}", origin);

        let mut queue = VecDeque::new();

        let (mut lefts, mut rights) = (0, 0); 
        let mut pos = origin;
        while let Some((npos, vertex, corner, turn)) = self.pop_next_border(&pos) {
            match turn {
                Turn::Left => lefts += 1,
                Turn::Right => rights += 1,
                Turn::Straight => (),
                _ => (),
            }

            while queue.len() > 4 {
                queue.pop_back();
            }

            queue.push_front(turn);
            if let Some(corner) = corner {
                vertices.push((corner.0, corner.1));
                queue.push_front(Turn::Corner)
            }

            Self::prune_vertices(&mut vertices, &mut queue);

            vertices.push((vertex.0, vertex.1));
            pos = npos;
        }

        let is_hole = rights > lefts;
        //if is_hole { vertices.reverse() }
        let dims = (self.borders.len(), self.borders[0].len());
        return Some((RawPolygon { is_hole, verticies: vertices, point_inside: origin.move_fwd(dims), holes: Vec::new() }, color));
    }

    fn load(img: Image) -> Self {
        println!("Image dimensions: {}x{}", img.get_width(), img.get_height());

        let (width, height) = (img.get_width(), img.get_height());

        let mut borders = BorderMap::new(width as usize, height as usize);

        for (x, y) in img.coordinates() {
            let act_y = height - y - 1;
            let color = img.get_pixel(x, y);
            borders.insert_clr((x as usize, act_y as usize), (color.r, color.g, color.b));
            if x == 0 || img.get_pixel(x - 1, y) != color {
                let pos = Position { x: x as usize, y: act_y as usize, dir: Direction::West };
                borders.insert(&pos)
            }
            if x == width - 1 || img.get_pixel(x + 1, y) != color {
                let pos = Position { x: x as usize, y: act_y as usize, dir: Direction::East };
                borders.insert(&pos)
            }
            if y == 0 || img.get_pixel(x, y - 1) != color {
                let pos = Position { x: x as usize, y: act_y as usize, dir: Direction::North };
                borders.insert(&pos)
            }
            if y == height - 1 || img.get_pixel(x, y + 1) != color {
                let pos = Position { x: x as usize, y: act_y as usize, dir: Direction::South };
                borders.insert(&pos)
            }
        }

        borders
    }
}

#[derive(Debug, Clone)]
struct RawPolygon {
    is_hole: bool,
    verticies: Vec<(f32, f32)>,
    point_inside: Option<Position>,
    holes: Vec<RawPolygon>,
}

impl RawPolygon {
    fn is_inside(&self, (x, y): (f32, f32)) -> bool {
        let mut c = false;
        for i in 0..self.verticies.len() {
            let j = (i + 1) % self.verticies.len();
            if ((self.verticies[i].1 > y) != (self.verticies[j].1 > y)) && (x < (self.verticies[j].0 - self.verticies[i].0) * (y - self.verticies[i].1) / (self.verticies[j].1 - self.verticies[i].1) + self.verticies[i].0) {
                c = !c;
            }
        }
        return c;
    }

    fn border_vertices(&self) -> Vec<[f32; 3]> {
        self.verticies.iter().map(|(x, y)| [*x, *y, 0.0]).collect()
    }

    fn vertices_indices(&self) -> (Vec<[f32; 3]>, Vec<u32>) {
        let mut raw_verticies = vec![self.verticies.iter().map(|(x, y)| vec![*x, *y]).collect::<Vec<_>>()];

        for hole in self.holes.iter() {
            raw_verticies.push(hole.verticies.iter().map(|(x, y)| vec![*x, *y]).collect::<Vec<_>>());
        }

        let (vertices, holes, dimensions) = earcutr::flatten(&raw_verticies);
        let triangles = earcutr::earcut(&vertices, &holes, dimensions).unwrap();

        let verticies: Vec<[f32; 3]> = vertices.chunks(2).map(|chunk| [chunk[0], chunk[1], 0.0]).collect();

        (verticies, triangles.into_iter().map(|i| i as u32).collect())
    }
}

fn finish_polygons(polygons: HashMap<(u8, u8, u8), Vec<RawPolygon>>) -> Vec<Polygon> {
    let mut finished_polygons: Vec<Polygon> = Vec::new();

    for (color, raw_polys) in polygons {
        let holes = raw_polys.iter().filter(|poly| poly.is_hole).cloned().collect::<Vec<_>>();
        let mut non_holes = raw_polys.into_iter().filter(|poly| !poly.is_hole).collect::<Vec<_>>();

        //println!("Finishing polygon: {:?}", color);

        let mut polygon = Polygon {
            source_color: color,
            vertices: Vec::new(),
            border_vertices: Vec::new(),
            indicies: Vec::new(),
        };

        for hole in holes {
            let mut found = false;
            for non_hole in &mut non_holes {
                let point_in_hole = (hole.point_inside.unwrap().x as f32, hole.point_inside.unwrap().y as f32);
                if non_hole.is_inside(point_in_hole) {
                    polygon.border_vertices.push(hole.border_vertices());
                    non_hole.holes.push(hole);
                    found = true;
                    //println!("Found hole");
                    break;
                }
            }
            if !found {
                panic!("Hole not found");
            }
        }

        for poly in non_holes {
            let (vertices, indices) = poly.vertices_indices();

            let vertices_before = polygon.vertices.len();
            polygon.vertices.extend_from_slice(&vertices);
            polygon.indicies.extend(indices.into_iter().map(|i| i + vertices_before as u32));


            polygon.border_vertices.push(poly.border_vertices());
            //finished_polygons.push(Polygon::new((color.0, color.1, color.2), poly));
            //break;
        }

        finished_polygons.push(polygon);
    }

    finished_polygons
}

pub fn load_polygons(img: Image) -> Vec<Polygon> {
    let before = std::time::Instant::now();
    let mut borders = BorderMap::load(img);
    println!("Loaded in {}ms", before.elapsed().as_millis());

    let mut raw_polys: HashMap<(u8, u8, u8), Vec<RawPolygon>> = HashMap::new();

    let before = std::time::Instant::now();
    while let Some((poly, color)) = borders.pop_polygon() {
        match raw_polys.get_mut(&color) {
            Some(vec) => vec.push(poly),
            None => { raw_polys.insert(color, vec![poly]); },
        }
    }
    println!("Found all polygons in {}ms", before.elapsed().as_millis());

    let before = std::time::Instant::now();
    let res = finish_polygons(raw_polys);
    println!("Finished polygons in {}ms", before.elapsed().as_millis());

    res
}
