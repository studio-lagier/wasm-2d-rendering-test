mod utils;

use wasm_bindgen::prelude::*;

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
    width: u32,
    height: u32,
    pixel_width: u32,
    pixel_height: u32,
    cells: Vec<Cell>,
    pixels: Vec<Pixel>
}

impl Universe {
    fn get_cell_index(&self, row: u32, col: u32) -> usize {
        (row * self.width + col) as usize
    }

    fn get_pixel_index(&self, x: u32, y: u32) -> usize {
        (y * self.pixel_width + x) as usize
    }

    fn live_neighbor_count(&self, row: u32, col: u32) -> u8 {
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
        // Whole pixel offsets only for now
        let cell_width = (self.pixel_width - 2) as f32 / self.width as f32;
        let cell_height = (self.pixel_height - 2) as f32 / self.height as f32;

        for row in 0..self.width {
            for col in 0..self.height {
                let idx = self.get_cell_index(row, col);
                let cell = self.cells[idx];

                self.draw_square(row, col, cell_width, cell_height, &cell);
            }
        }

        for row in 0 .. self.height {
            self.draw_row(row, cell_height);
        }

        for col in 0 .. self.width {
            self.draw_col(col, cell_width);
        }
    }

    fn draw_row(&mut self, row: u32, cell_height: f32) {
        // Round to whole pixel
        let y = (row as f32 * cell_height).round() as u32;

        for x in 0..self.pixel_width {
            let idx = self.get_pixel_index(x, y);
            self.pixels[idx] = Pixel(0xdd, 0xdd, 0xdd, 0xff);
        }
    }

    fn draw_col(&mut self, col: u32, cell_width: f32) {
        // Round to whole pixel value
        let x = (col as f32 * cell_width).round() as u32;

        for y in 0..self.pixel_height {
            let idx = self.get_pixel_index(x, y);
            self.pixels[idx] = Pixel(0xdd, 0xdd, 0xdd, 0xff);
        }
    }

    fn draw_square(&mut self, row: u32, col: u32, width: f32, height: f32, cell: &Cell) {

        let start_x = (row as f32 * width).round() as u32;
        let start_y = (col as f32 * height).round() as u32;

        let end_x = (start_x as f32 + width).round() as u32;
        let end_y = (start_y as f32 + height).round() as u32;

        let pixel = match cell {
            Cell::Alive => Pixel(0, 0, 0, 0xff),
            Cell::Dead => Pixel(0, 0, 0, 0)
        };

        for curr_x in start_x ..= end_x {
            for curr_y in start_y ..= end_y {
                let idx = self.get_pixel_index(curr_x, curr_y);
                self.pixels[idx] = pixel;
            }
        }
    }

    pub fn new(pixel_width: u32, pixel_height: u32) -> Universe {
        utils::set_panic_hook();

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

        let pixels = (0..pixel_width * pixel_height)
            .map(|_| {
                Pixel(0xff, 0xff, 0xff, 0xff)
            })
            .collect();

        Universe {
            width,
            height,
            pixel_width,
            pixel_height,
            cells,
            pixels
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    pub fn image_data(&self) -> *const Pixel {
        self.pixels.as_ptr()
    }
}