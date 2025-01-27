// TODO: In the future you need to refactor this for other simulations, take out the map logic, and
// the drawing logic and isolate that in some other class in a more generalized manner, you would
// keep the Map struct, but rename it to conway_map, and then for any other simulation you would
// have it be (sim name)_Map struct or something. Then, you would use those structs to make the
// draw calls you figured out/thought about in the main, kinda "super" Map struct.

// TODO: Implement a "stamp" or blueprint feature in which the user can stamp their own pre-saved
// game of life patterns into the map? Provide some basic ones like gliders and such
use std::{collections::HashSet, fs};

use egui::{vec2, Color32, Rect, Rounding, Shape};
use instant::{Duration, Instant};
use rand::{thread_rng, Rng};
use crate::Pos;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConwayCell {
    Alive = 1,
    Dead = 0,
}
/// "Neighbor" cells around the current cell, coordinates are organized in standard x,y format
/// ## Think of the layout like this:
/// (-1,1 ) (0,1 )  (1,1 )
/// (-1,0 ) (cell)  (1,0 )
/// (-1,-1) (0,-1)  (1,-1)
const NEIGHBORS: [(i32, i32); 8] = [
    (-1, 1),
    (0, 1),
    (1, 1),
    (-1, 0),
    (1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
];


#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Map {
    pub x_axis: i32,
    pub y_axis: i32,
    pub cell_size: f32,
    pub map_size: i32,
    pub speed: u128,
    pub fps: u32,
    pub rand_scarcity: u32,
    pub light_mode: bool,
    pub lines: bool,
    pub is_initial: bool,

    #[serde(skip)]
    last_frame_time: Instant,
    #[serde(skip)]
    cells: HashSet<Pos>,
    initial_state: HashSet<Pos>,
}

impl Default for Map {
    fn default() -> Self {
        Map::new()
    }
}

impl Map {
    pub fn new() -> Self {
        Self {
            fps: 10,
            speed: Map::fps_to_speed(10.0),
            cells: HashSet::new(),
            initial_state: HashSet::new(),
            last_frame_time: Instant::now(),
            map_size: 75,
            cell_size: 10.0,
            x_axis: 0,
            y_axis: 0,
            rand_scarcity: 3 as u32,
            light_mode: true,
            lines: false,
            is_initial: true,
        }
    }
    pub fn update_speed(&mut self) {
        self.speed = Map::fps_to_speed(self.fps as f32);
    }
    // pub fn neighbors(&self, p: &Pos) -> usize {
    //     let mut neighbors = 0;
    //     for i in NEIGHBORS {
    //         if self.cells.contains(&Pos(p.0 + i.0, p.1 + i.1)) {
    //             neighbors += 1;
    //         }
    //     }
    //     neighbors
    // }
    // NOTE: Below, I have a new toroidal function for neighbor checking, and above, I have a more
    // naive check, maybe later look into seeing if it is possible to toggle between say "hard"
    // walls and toroidal walls that just tile
    pub fn neighbors(&self, p: &Pos) -> usize {
        NEIGHBORS.iter().fold(0, |neighbors, &i| {
            let mut neighbor_pos = Pos(p.0 + i.0, p.1 + i.1);

            neighbor_pos.0 = self.apply_periodic_boundary(neighbor_pos.0, self.map_size);
            neighbor_pos.1 = self.apply_periodic_boundary(neighbor_pos.1, self.map_size);

            neighbors + self.cells.contains(&neighbor_pos) as usize
        })
    }

    pub fn gen_random(&mut self) {
        self.clear();
        for y in 0..=self.map_size - 4 {
            for x in 0..=self.map_size - 4 {
                let mut rng = thread_rng();
                let probability = rng.gen_range(0..=self.rand_scarcity);
                if probability == 1 {
                    self.cells.insert(Pos(x, y));
                }
            }
        }
       //self.cache_initial_state();
    }
    pub fn cache_initial_state(&mut self){
       self.initial_state = self.cells.clone();
       //basically anytime this has been called, AND update has not been called, we can garuntee we
       //are in the "initial" state of the app
       self.is_initial = true;
    }
    pub fn restore_initial_state(&mut self){
        self.cells = self.initial_state.clone();
        self.is_initial = true;
    }
    pub fn clear(&mut self) {
        self.cells = HashSet::new();
    }
    pub fn fps_to_speed(fps: f32) -> u128 {
        Duration::new(0, (1000000000.0 / fps) as u32).as_millis()
    }
    // NOTE: This could probably be useful for the refactor
    pub fn update(&mut self) {
        let duration_since_last_frame = Instant::now().duration_since(self.last_frame_time);
        //below line basically forces fps to work. like, it's saying "if last frame happened, but
        //is lower then our set speed, don't do SHIT!"
        if duration_since_last_frame.as_millis().lt(&self.speed) {
            return;
        }
        let mut n_cells = HashSet::new();
        let mut checked = HashSet::new();
        for cell in &self.cells {
            for step in NEIGHBORS {
                let mut xy = Pos(cell.0 + step.0, cell.1 + step.1);

                xy.0 = self.apply_periodic_boundary(xy.0, self.map_size);
                xy.1 = self.apply_periodic_boundary(xy.1, self.map_size);

                if !checked.contains(&xy) {
                    checked.insert(xy);
                    let n = self.neighbors(&xy);
                    if n == 2 && self.cells.contains(&xy) || n == 3 {
                        n_cells.insert(xy);
                    }
                }
            }
        }
        self.last_frame_time = Instant::now();
        self.cells = n_cells;
        self.is_initial = false;
    }
    // NOTE: If I end up generalizing/standardizing the way a map is implemented in some refactor
    // down the line, I should move alot of these functions to a parent mod.rs file. For now I
    // don't want to build unneccessary abstractions if it turns out I don't need them, and have
    // weird code layout with everything in tiny files when it doesn't need to be organized that
    // way.
    fn find_min(&self) -> (i32, i32) {
        let mut min_x = -1;
        let mut min_y = -1;
        for el in &self.cells {
            if min_x == -1 || el.0 < min_x {
                min_x = el.0;
            }
            if min_y == -1 || el.1 < min_y {
                min_y = el.1;
            }
        }
        (min_x, min_y)
    }
    fn find_max(&self) -> (i32, i32) {
        let mut max_x = -1;
        let mut max_y = -1;
        for el in &self.cells {
            if el.0 > max_x {
                max_x = el.0;
            }
            if el.1 > max_y {
                max_y = el.1;
            }
        }
        (max_x, max_y)
    }
    pub fn center_cells(&mut self, rect: Rect) {
        let (min_x, min_y) = self.find_min();
        let (max_x, max_y) = self.find_max();
        let mut elems_c = HashSet::new();
        if rect.max.x > rect.max.y {
            self.cell_size = ((rect.max.x - rect.min.x) as i32 / self.map_size) as f32;
        } else {
            self.cell_size = ((rect.max.y - rect.min.y) as i32 / self.map_size) as f32;
        }
        for cell in &self.cells {
            elems_c.insert(Pos(
                self.map_size / 2 - (max_x - min_x) / 2 + cell.0,
                self.map_size / 2 - (max_y - min_y) / 2 + cell.1,
            ));
        }

        self.cells = elems_c;
    }
pub fn draw_lines(&mut self, rect: Rect, shapes: &mut Vec<Shape>) {
    // Calculate stroke thickness based on cell size
    let stroke_thickness = self.exponential_easing(0.1, 50.0, 0.0, 2.0);

    // Draw vertical grid lines
    for i in 0..=self.map_size {
        let x = rect.min.x + self.cell_size as f32 * i as f32 - self.x_axis as f32;
        shapes.push(Shape::line_segment(
            [
                egui::Pos2::new(x, rect.min.y),
                egui::Pos2::new(x, rect.min.y + self.cell_size as f32 * self.map_size as f32),
            ],
            egui::Stroke::new(
                stroke_thickness,
                if i == self.map_size {
                    Color32::RED
                } else {
                    Color32::GRAY
                },
            ),
        ));
    }

    // Draw horizontal grid lines
    for i in 0..=self.map_size {
        let y = rect.min.y + self.cell_size as f32 * i as f32 - self.y_axis as f32;
        shapes.push(Shape::line_segment(
            [
                egui::Pos2::new(rect.min.x, y),
                egui::Pos2::new(rect.min.x + self.cell_size as f32 * self.map_size as f32, y),
            ],
            egui::Stroke::new(
                stroke_thickness,
                if i == self.map_size {
                    Color32::RED
                } else {
                    Color32::GRAY
                },
            ),
        ));
    }
}
    pub fn generate_cells(&self, shapes: &mut Vec<Shape>, rect: Rect) {
        for c in &self.cells {
            shapes.push(Shape::rect_filled(
                Rect {
                    min: rect.min
                        + vec2(
                            self.cell_size as f32 * c.0 as f32 - self.x_axis as f32,
                            self.cell_size as f32 * c.1 as f32 - self.y_axis as f32,
                        ),
                    max: rect.min
                        + vec2(
                            self.cell_size as f32 * (c.0 + 1) as f32 - self.x_axis as f32,
                            self.cell_size as f32 * (c.1 + 1) as f32 - self.y_axis as f32,
                        ),
                },
                Rounding::ZERO,
                //TODO: Add a slider for the user on this one that allows them to choose the color
                //if they want
                if self.light_mode {
                    Color32::BLACK
                } else {
                    Color32::WHITE
                },
            ));
        }
    }
    ///Function largely exists solely for the purpose of easing the thickness of the gridlines
    ///based on the cell size
    fn sigmoid_easing(&mut self, x_0: f32, k: f32) -> f32 {
        let exponent = -k * (self.cell_size - x_0);
        1.0 / (1.0 + exponent.exp())
    }
    fn exponential_easing(
        &mut self,
        min_cell_size: f32,
        max_cell_size: f32,
        min_thickness: f32,
        max_thickness: f32,
    ) -> f32 {
        if self.cell_size <= min_cell_size {
            return min_thickness; // Gridlines disappear when zoomed out completely
        }
        if self.cell_size > max_cell_size {
            return max_thickness; // Gridlines are thickest when zoomed in completely
        }

        let t = (self.cell_size - min_cell_size) / (max_cell_size - min_cell_size); // Normalized value between 0 and 1

        let thickness = min_thickness + t * (max_thickness - min_thickness);

        thickness
    }

    // TODO: Use this code, and a provided text box to allow users to make "blueprints"
    pub fn generate_from_file(&mut self, f: &str) {
        if fs::read_to_string(f).is_err() {
            println!("Error reading from file");
            return;
        };
        let contents = fs::read_to_string(f).expect("Error reading from file");

        let mut x = HashSet::new();
        for (ind, l) in contents.split('\n').enumerate() {
            for (i, c) in l.chars().enumerate() {
                if c == '#' {
                    x.insert(Pos(i as i32, ind as i32));
                }
            }
        }
        self.cells = x;
    }
    fn apply_periodic_boundary(&self, coord: i32, axis_size: i32) -> i32 {
        match coord {
            x if x < 0 => axis_size - 1,
            x if x >= axis_size => 0,
            x => x,
        }
    }
}
