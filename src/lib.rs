mod utils;
use pathfinder_canvas::{Canvas, CanvasFontContext, Path2D, FillStyle };
use pathfinder_color::{ColorF, ColorU};
use pathfinder_content::fill::{FillRule};
use pathfinder_geometry::rect::RectF;
use pathfinder_geometry::vector::{vec2f, vec2i, Vector2I};
use pathfinder_renderer::concurrent::executor::SequentialExecutor;
use pathfinder_renderer::gpu::options::{DestFramebuffer, RendererMode, RendererOptions};
use pathfinder_renderer::gpu::renderer::Renderer;
use pathfinder_renderer::options::BuildOptions;
use pathfinder_resources::embedded::EmbeddedResourceLoader;
use pathfinder_webgl::WebGlDevice;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
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
pub struct Universe {
    width: i32,
    height: i32,
    pixel_width: u32,
    pixel_height: u32,
    cells: Vec<Cell>,
    renderer: Renderer<WebGlDevice>,
    framebuffer_size: Vector2I
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
        let font_context = CanvasFontContext::from_system_source();
        let mut canvas = Canvas::new(self.framebuffer_size.to_f32()).get_context_2d(font_context);

        let cell_width = (self.pixel_width as f32 - 2.) / self.width as f32;
        let cell_height = (self.pixel_height as f32 - 2.) / self.height as f32;

        let mut alive = Path2D::new();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_cell_index(row, col);
                let cell = self.cells[idx];

                let x0 = col as f32 * cell_width;
                let y0 = row as f32 * cell_height;

                let rect = RectF::new(vec2f(x0, y0), vec2f(cell_width, cell_height));

                match cell {
                    // This is kinda slow, not sure how we can speed it up
                    Cell::Alive => alive.rect(rect),
                    _ => ()
                }
            }
        }

        let alive_color = ColorU {
            r: 0,
            g: 0,
            b: 0,
            a: 0xff
        };

        canvas.set_fill_style(FillStyle::Color(alive_color));
        canvas.fill_path(alive, FillRule::Winding);

        let mut grid = Path2D::new();

        for row in 0..=self.height {
            let y = (row as f32 * cell_height) + 1.;

            grid.move_to(vec2f(0., y));
            grid.line_to(vec2f(self.pixel_width as f32, y));
        }

        for col in 0..=self.width {
            let x = (col as f32 * cell_width) + 1.;

            grid.move_to(vec2f(x, 0.));
            grid.line_to(vec2f(x, self.pixel_height as f32));
        }

        let grid_color = ColorU {
            r: 0xdd,
            g: 0xdd,
            b: 0xdd,
            a: 0xff
        };

        canvas.set_stroke_style(FillStyle::Color(grid_color));
        canvas.stroke_path(grid);

        // Render the canvas to screen.
        let mut scene = canvas.into_canvas().into_scene();
        scene.build_and_render(&mut self.renderer, BuildOptions::default(), SequentialExecutor);
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

            let document = web_sys::window().unwrap().document().unwrap();
            let canvas = document.get_element_by_id(canvas_id).unwrap();
            let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>().unwrap();

            let context = canvas.get_context("webgl2")
                                .unwrap()
                                .unwrap()
                                .dyn_into::<WebGl2RenderingContext>()
                                .unwrap();

            // Get the real size of the window, taking HiDPI into account.
            let framebuffer_size = vec2i(canvas.width() as i32, canvas.height() as i32);

            // Create a Pathfinder GL device.
            let pathfinder_device = WebGlDevice::new(context);

            // Create a Pathfinder renderer.
            let mode = RendererMode::default_for_device(&pathfinder_device);
            let options = RendererOptions {
                dest: DestFramebuffer::full_window(framebuffer_size),
                background_color: Some(ColorF::white()),
                ..RendererOptions::default()
            };
            let resource_loader = EmbeddedResourceLoader::new();
            let renderer = Renderer::new(pathfinder_device, &resource_loader, mode, options);

        Universe {
            width,
            height,
            pixel_width,
            pixel_height,
            cells,
            renderer,
            framebuffer_size,
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
