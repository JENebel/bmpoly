pub struct Polygon {
    pub color: (f32, f32, f32),
    pub verticies: Vec<[f32; 3]>,
    pub indicies: Vec<u32>,
}

impl Polygon {
    fn new(color: (f32, f32, f32), raw: RawPolygon) -> Polygon {
        let mut raw_verticies = vec![raw.verticies.iter().map(|(x, y)| vec![*x, *y]).collect::<Vec<_>>()];

        for hole in raw.holes {
            raw_verticies.push(hole.verticies.iter().map(|(x, y)| vec![*x, *y]).collect::<Vec<_>>());
        }

        let (vertices, holes, dimensions) = earcutr::flatten(&raw_verticies);
        let triangles = earcutr::earcut(&vertices, &holes, dimensions).unwrap();

        let verticies: Vec<[f32; 3]> = vertices.chunks(2).map(|chunk| [chunk[0], chunk[1], 0.0]).collect();

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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Direction {
    North = 0,
    South = 1,
    East = 2,
    West = 3,
}

use std::collections::HashMap;

use Direction::*;

impl Direction {
    fn from_int(dir: usize) -> Direction {
        match dir {
            0 => North,
            1 => South,
            2 => East,
            3 => West,
            _ => panic!("Invalid direction")
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
}

#[derive(Debug, Clone)]
struct BorderMap {
    colors: Vec<Vec<(u8, u8, u8)>>,
    borders: Vec<Vec<[bool; 4]>>,
}

impl BorderMap {
    fn new(width: usize, height: usize) -> BorderMap {
        BorderMap {
            colors: vec![vec![(0, 0, 0); height]; width],
            borders: vec![vec![[false; 4]; height]; width]
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
    }

    fn insert_clr(&mut self, (x, y): (usize, usize), clr: (u8, u8, u8)) {
        self.colors[x][y] = clr;
    }

    fn insert(&mut self, pos: &Position) {
        self.borders[pos.x][pos.y][pos.dir as usize] = true;
    }

    fn get_some_starting_point(&self) -> Option<Position> {
        for x in 0..self.borders.len() {
            for y in 0..self.borders[x].len() {
                for dir in 0..4 {
                    if self.borders[x][y][dir] {
                        return Some(Position { x, y, dir: Direction::from_int(dir) })
                    }
                }
            }
        }
        None
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

    fn pop_polygon(&mut self) -> Option<(RawPolygon, (u8, u8, u8))> {
        let origin = match self.get_some_starting_point() {
            Some(p) => p,
            None => return None,
        };

        //println!("Starting at {:?}", origin);

        let mut vertices: Vec<(f32, f32)> = Vec::new();
        let color = self.get_clr(&origin);

        //println!("Starting at {:?}", origin);

        let (mut lefts, mut rights) = (0, 0); 
        let mut pos = origin;
        while let Some((npos, vertex, corner, turn)) = { /*println!("{:?}", self);*/ self.pop_next_border(&pos)} {
            if let Some(corner) = corner {
                vertices.push((corner.0, corner.1));
            }

            //println!("{:?},     \t Pos: {:?}", turn, npos);
            match turn {
                Turn::Left => lefts += 1,
                Turn::Right => rights += 1,
                Turn::Straight => (),
            }

            vertices.push((vertex.0, vertex.1));
            pos = npos;
        }

        //println!("Lefts: {}, Rights: {}", lefts, rights);

        //println!("Found polygon: {:?}", vertices.len());
        let is_hole = rights > lefts;
        if is_hole { vertices.reverse() }
        let dims = (self.borders.len(), self.borders[0].len());
        return Some((RawPolygon { is_hole, verticies: vertices, point_inside: origin.move_fwd(dims), holes: Vec::new() }, color));
    }

    fn load(path: &str) -> Self {
        let img = bmp::open(path).unwrap();
    
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
}

fn finish_polygons(polygons: HashMap<(u8, u8, u8), Vec<RawPolygon>>) -> Vec<Polygon> {
    let mut finished_polygons: Vec<Polygon> = Vec::new();

    for (color, raw_polys) in polygons {
        let holes = raw_polys.iter().filter(|poly| poly.is_hole).cloned().collect::<Vec<_>>();
        let mut non_holes = raw_polys.into_iter().filter(|poly| !poly.is_hole).collect::<Vec<_>>();

        //println!("Finishing polygon: {:?}", color);

        for hole in holes {
            let mut found = false;
            for non_hole in &mut non_holes {
                let point_in_hole = (hole.point_inside.unwrap().x as f32, hole.point_inside.unwrap().y as f32);
                if non_hole.is_inside(point_in_hole) {
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
            finished_polygons.push(Polygon::new((color.0 as f32 / 255.0, color.1 as f32 / 255.0, color.2 as f32 / 255.0), poly));
        }
    }

    finished_polygons
}

pub fn load_polygons() -> Vec<Polygon> {
    let mut borders = BorderMap::load("assets/provinces.bmp");
    let mut raw_polys: HashMap<(u8, u8, u8), Vec<RawPolygon>> = HashMap::new();

    let time_before = std::time::Instant::now();
    while let Some((poly, color)) = borders.pop_polygon() {
        match raw_polys.get_mut(&color) {
            Some(vec) => vec.push(poly),
            None => { raw_polys.insert(color, vec![poly]); },
        }
        //break
    }

    println!("Finished in {}ms", time_before.elapsed().as_millis());

    finish_polygons(raw_polys)
}
