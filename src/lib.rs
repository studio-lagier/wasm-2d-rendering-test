mod utils;

use wasm_bindgen::prelude::*;
use tiny_skia::*;

extern crate web_sys;
// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}


// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1
}

#[wasm_bindgen]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
// r, g, b, a
pub struct Pixel (u8, u8, u8, u8);

#[wasm_bindgen]
pub struct Universe {
    width: i32,
    height: i32,
    pixel_width: u32,
    pixel_height: u32,
    cells: Vec<Cell>,
    pixmap: Pixmap
}

impl Universe {
    fn get_cell_index(&self, row: i32, col: i32) -> usize {
        (row * self.width + col) as usize
    }

    fn get_pixel_index(&self, row: i32, col: i32) -> usize {
        ((row * self.width + col) * 4) as usize
    }

    fn live_neighbor_count(&self, row: i32, col: i32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (col + delta_col) % self.width;
                let idx = self.get_cell_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }

        count
    }
}

#[wasm_bindgen]
impl Universe {
    pub fn tick(&mut self) {
        self.build_board();
        self.render_board();
    }

    fn build_board(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_cell_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                let next_cell = match (cell, live_neighbors) {
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    (Cell::Dead, 3) => Cell::Alive,
                    (otherwise, _) => otherwise,
                };

                next[idx] = next_cell;
            }
        }

        self.cells = next;
    }

    // We map cells to pixels - this would get replaced with a proper
    fn render_board(&mut self) {
        // Draw our grid
        let cell_width = (self.pixel_width - 2) as f32 / self.width as f32;
        let cell_height = (self.pixel_height - 2) as f32 / self.height as f32;

        let mut grid_color = Paint::default();
        grid_color.set_color_rgba8(0xff, 0xdd, 0xdd, 0xdd);
        let mut alive_color = Paint::default();
        alive_color.set_color_rgba8(0, 0, 0, 0xff);
        let mut dead_color = Paint::default();
        dead_color.set_color_rgba8(0xff, 0xff, 0xff, 0xff);

        // Draw our alive squares
        // let mut alive_pb = PathBuilder::new();
        // let mut dead_pb = PathBuilder::new();

        for row in 0..self.width {
            for col in 0..self.height {
                let idx = self.get_cell_index(row, col);
                let cell = self.cells[idx];

                let x = row as f32 * cell_height + 1.;
                let y = col as f32 * cell_width + 1.;

                // let loop_pb = match cell {
                //     Cell::Alive => &mut alive_pb,
                //     Cell::Dead => &mut dead_pb
                // };

                // loop_pb.move_to(x, y);
                // loop_pb.line_to(x + cell_width, y);
                // loop_pb.line_to(x + cell_width, y + cell_height);
                // loop_pb.line_to(x, y + cell_height);
                // loop_pb.close();

                let color = match cell {
                    Cell::Alive => &alive_color,
                    Cell::Dead => &dead_color
                };

                let rect = Rect::from_xywh(x, y, cell_width, cell_height).unwrap();
                self.pixmap.fill_rect(rect, &color, Transform::identity(), None);
            }
        }

        // let alive_path = alive_pb.finish().unwrap();
        // let dead_path = dead_pb.finish().unwrap();

        // self.pixmap.fill_path(&alive_path, &alive_color, FillRule::Winding, Transform::identity(), None);
        // self.pixmap.fill_path(&dead_path, &dead_color, FillRule::Winding, Transform::identity(), None);
    }

    pub fn new(pixel_width: u32, pixel_height: u32) -> Universe {
        let width = 64;
        let height = 64;

        let cells = (0..width * height)
            .map(|i| {
                if i % 2 == 0 || i % 7 == 0 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        let pixmap = Pixmap::new(pixel_width, pixel_height).unwrap();

        Universe {
            width,
            height,
            pixel_width,
            pixel_height,
            cells,
            pixmap
        }
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    pub fn image_data(&self) -> *const u8 {
        self.pixmap.data().as_ptr()
    }
}