mod utils;

use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlCanvasElement, CanvasRenderingContext2d};
use wasm_bindgen::JsCast;
use winit::window::WindowBuilder;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowBuilderExtWebSys;
use femtovg::{
    //CompositeOperation,
    renderer::OpenGl,
    Align,
    Baseline,
    Canvas,
    Color,
    FillRule,
    FontId,
    ImageFlags,
    ImageId,
    LineCap,
    LineJoin,
    Paint,
    Path,
    Renderer,
    Solidity,
};

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
    window: winit::window::Window,
    canvas: Canvas<OpenGl>
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
        let alive_color = Color::rgb(0, 0, 0);
        let dead_color = Color::rgb(255, 255, 255);

        self.canvas.clear_rect(0, 0, self.pixel_width, self.pixel_height, dead_color);

        let cell_width = (self.pixel_width as f32 - 2.) / self.width as f32;
        let cell_height = (self.pixel_height as f32 - 2.) / self.height as f32;


        let mut alive_path = Path::new();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_cell_index(row, col);
                let cell = self.cells[idx];

                let x = col as f32 * cell_width;
                let y = row as f32 * cell_height;

                match cell {
                    Cell::Alive => alive_path.rect(x, y, cell_width, cell_height),
                    _ => ()
                }
            }
        }

        self.canvas.fill_path(&mut alive_path, Paint::color(alive_color));

        let mut grid = Path::new();

        for row in 0..=self.height {
            let y = (row as f32 * cell_height) + 1.;

            grid.move_to(0., y);
            grid.line_to(self.pixel_width as f32, y);
        }

        for col in 0..=self.width {
            let x = (col as f32 * cell_width) + 1.;

            grid.move_to(x, 0.);
            grid.line_to(x, self.pixel_height as f32);
        }

        self.canvas.stroke_path(&mut grid, Paint::color(alive_color));

        self.canvas.flush();
    }

    #[cfg(target_arch = "wasm32")]
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

        let canvas_el = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(canvas_id)
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();

        let renderer = OpenGl::new_from_html_canvas(&canvas_el).expect("Cannot create renderer");

        let el = EventLoop::new();

        let window = WindowBuilder::new()
            .with_canvas(Some(canvas_el))
            .build(&el)
            .unwrap();

        let dpi = window.scale_factor();

        let size = PhysicalSize::new(pixel_width as f64 * dpi, pixel_height as f64 * dpi);

        window.set_inner_size(size);

        let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");

        canvas.set_size(size.width as u32, size.height as u32, dpi as f32);
        // canvas.scale(2., 2.);

        Universe {
            width,
            height,
            pixel_width,
            pixel_height,
            cells,
            window,
            canvas
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
}