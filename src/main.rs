use macroquad::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Cell {
    Alive = 1,
    Dead = 0,
}

pub struct Universe {
  width: i32,
  height: i32,
  pixel_width: u32,
  pixel_height: u32,
  cells: Vec<Cell>
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
      clear_background(WHITE);

      let cell_width = (self.pixel_width as f32 - 2.) / self.width as f32;
      let cell_height = (self.pixel_height as f32 - 2.) / self.height as f32;

      for row in 0..self.height {
          for col in 0..self.width {
              let idx = self.get_cell_index(row, col);
              let cell = self.cells[idx];

              let x = col as f32 * cell_width;
              let y = row as f32 * cell_height;

              match cell {
                  Cell::Alive => draw_rectangle(x, y, cell_width, cell_height, BLACK),
                  _ => ()
              }
          }
      }

      for row in 0..=self.height {
          let y = (row as f32 * cell_height) + 1.;

          draw_line(0., y, self.pixel_width as f32, y, 1., GRAY);
      }

      for col in 0..=self.width {
          let x = (col as f32 * cell_width) + 1.;

          draw_line(x, 0., x, self.pixel_height as f32, 1., GRAY);
      }
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

    Universe {
        width,
        height,
        pixel_width,
        pixel_height,
        cells
    }
  }

  pub fn cells(&self) -> *const Cell {
    self.cells.as_ptr()
  }
}

#[macroquad::main("Life")]
async fn main() {
    let w = screen_width() as usize;
    let h = screen_height() as usize;

    let mut universe = Universe::new(w as u32, h as u32);

    loop {
      universe.tick();
      let fps = macroquad::time::get_fps();
      draw_text(&fps.to_string(), 20.0, 20.0, 20.0, DARKGRAY);
      next_frame().await
    }
}