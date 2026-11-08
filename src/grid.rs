//! A character grid that game code draws into, independent of the GPU.

/// An opaque sRGB color packed as `0xAABBGGRR`, the layout `unpack4x8unorm` reads in WGSL.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color(pub u32);

impl Color {
  pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
    Self(0xff00_0000 | (b as u32) << 16 | (g as u32) << 8 | r as u32)
  }
}

pub const BLACK: Color = Color::rgb(0, 0, 0);
pub const WHITE: Color = Color::rgb(255, 255, 255);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
  pub ch: char,
  pub fg: Color,
  pub bg: Color,
}

impl Cell {
  pub const BLANK: Self = Self {
    ch: ' ',
    fg: WHITE,
    bg: BLACK,
  };
}

pub struct Grid {
  cols: u32,
  rows: u32,
  cells: Vec<Cell>,
}

impl Grid {
  pub fn new(cols: u32, rows: u32) -> Self {
    Self {
      cols,
      rows,
      cells: vec![Cell::BLANK; (cols * rows) as usize],
    }
  }

  pub fn cols(&self) -> u32 {
    self.cols
  }

  pub fn rows(&self) -> u32 {
    self.rows
  }

  /// The cells in row-major order.
  pub fn cells(&self) -> &[Cell] {
    &self.cells
  }

  pub fn clear(&mut self) {
    self.cells.fill(Cell::BLANK);
  }

  /// Writes one cell. Positions outside the grid are ignored.
  pub fn put(&mut self, x: u32, y: u32, ch: char, fg: Color, bg: Color) {
    if x < self.cols && y < self.rows {
      self.cells[(y * self.cols + x) as usize] = Cell { ch, fg, bg };
    }
  }

  /// Writes `text` left to right from `(x, y)`, one cell per `char`, clipping at the grid edge.
  pub fn print(&mut self, x: u32, y: u32, text: &str, fg: Color, bg: Color) {
    for (offset, ch) in (0..).zip(text.chars()) {
      self.put(x.saturating_add(offset), y, ch, fg, bg);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn color_packs_red_into_the_low_byte() {
    assert_eq!(Color::rgb(0x11, 0x22, 0x33), Color(0xff33_2211));
  }

  #[test]
  fn print_writes_row_major_cells() {
    let mut grid = Grid::new(4, 2);
    grid.print(1, 1, "ab", WHITE, BLACK);
    assert_eq!(grid.cells()[5].ch, 'a');
    assert_eq!(grid.cells()[6].ch, 'b');
  }

  #[test]
  fn print_clips_at_the_right_edge_without_wrapping() {
    let mut grid = Grid::new(3, 2);
    grid.print(2, 0, "abc", WHITE, BLACK);
    assert_eq!(grid.cells()[2].ch, 'a');
    assert!(grid.cells()[3..].iter().all(|cell| cell.ch == ' '));
  }

  #[test]
  fn put_ignores_positions_outside_the_grid() {
    let mut grid = Grid::new(2, 2);
    grid.put(2, 0, 'x', WHITE, BLACK);
    grid.put(0, 2, 'x', WHITE, BLACK);
    assert!(grid.cells().iter().all(|cell| *cell == Cell::BLANK));
  }

  #[test]
  fn clear_blanks_every_cell() {
    let mut grid = Grid::new(2, 1);
    grid.print(0, 0, "ab", WHITE, BLACK);
    grid.clear();
    assert!(grid.cells().iter().all(|cell| *cell == Cell::BLANK));
  }
}
