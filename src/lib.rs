mod utils;

use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlCanvasElement, CanvasRenderingContext2d};
use wasm_bindgen::JsCast;
use piet::*;
use kurbo::*;
use piet_web::*;

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
pub struct Universe {
    width: i32,
    height: i32,
    pixel_width: u32,
    pixel_height: u32,
    cells: Vec<Cell>,
    canvas: HtmlCanvasElement,
    start_point: Point,
    end_point: Point,
    line_brush: Brush,
    live_brush: Brush,
    dead_brush: Brush
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

    fn render_board(&mut self) {
        let window = window().unwrap();
        let mut rc = WebRenderContext::new(self.get_context(), window);

        let cell_width = (self.pixel_width as f32 - 2.) / self.width as f32;
        let cell_height = (self.pixel_height as f32 - 2.) / self.height as f32;

        let mut alive = BezPath::new();
        let mut dead = BezPath::new();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_cell_index(row, col);
                let cell = self.cells[idx];

                let x0 = (col as f32 * cell_width) as f64;
                let y0 = (row as f32 * cell_height) as f64;
                let x1 = x0 + cell_width as f64;
                let y1 = y0 + cell_height as f64;

                let rect = Rect::new(x0, y0, x1, y1);

                match cell {
                    Cell::Alive => alive.extend(rect.path_elements(1.)),
                    Cell::Dead => dead.extend(rect.path_elements(1.))
                }
            }
        }

        rc.fill(alive, &self.live_brush);
        rc.fill(dead, &self.dead_brush);

        let mut grid = BezPath::new();

        for row in 0..=self.height {
            let y = (row as f32 * cell_height) + 1.;
            self.start_point.x = 0.;
            self.start_point.y = y as f64;

            self.end_point.x = self.pixel_width as f64;
            self.end_point.y = y as f64;

            grid.move_to(self.start_point);
            grid.line_to(self.end_point);
        }

        for col in 0..=self.width {
            let x = (col as f32 * cell_width) + 1.;

            self.start_point.x = x as f64;
            self.start_point.y = 0.;

            self.end_point.x = x as f64;
            self.end_point.y = self.pixel_height as f64;

            grid.move_to(self.start_point);
            grid.line_to(self.end_point);
        }

        rc.stroke(grid, &self.line_brush, 0.5);
    }

    pub fn new(pixel_width: u32, pixel_height: u32, canvas_id: &str ) -> Universe {
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

            let window = window().unwrap();
            let document = window.document().unwrap();
            let canvas = document
                .get_element_by_id(canvas_id)
                .unwrap()
                .dyn_into::<HtmlCanvasElement>()
                .unwrap();

            canvas.set_width(pixel_width);
            canvas.set_height(pixel_height);

            let context = canvas.get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .unwrap();

            let mut rc = WebRenderContext::new(context, window);
            let line_brush = rc.solid_brush(Color::rgb8(0xdd, 0xdd, 0xdd));
            let live_brush = rc.solid_brush(Color::rgb8(0, 0, 0));
            let dead_brush = rc.solid_brush(Color::rgb8(0xff, 0xff, 0xff));

            let start_point = Point::ORIGIN;
            let end_point = Point::ORIGIN;

        Universe {
            width,
            height,
            pixel_width,
            pixel_height,
            cells,
            canvas,
            start_point,
            end_point,
            line_brush,
            live_brush,
            dead_brush
        }
    }

    fn get_context(&self) -> CanvasRenderingContext2d {
        self.canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap()
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
}