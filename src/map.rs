use raylib::prelude::*;
use rsrl::rng::RNG;

use crate::{COLS, FONT_SIZE, ROWS};

#[derive(PartialEq, Clone, Copy)]
pub enum TileType {
  Wall,
  Floor,
}

pub fn xy_to_idx(x: i32, y: i32) -> usize {
  debug_assert!(x >= 0);
  debug_assert!(y >= 0);

  (y as usize * 80) + x as usize
}

pub fn new_map() -> Vec<TileType> {
  let mut map = vec![TileType::Floor; (COLS * ROWS) as usize];

  // make boundary walls
  for x in 0..COLS {
    map[xy_to_idx(x, 0)] = TileType::Wall;
    map[xy_to_idx(x, ROWS - 1)] = TileType::Wall;
  }

  for y in 0..ROWS {
    map[xy_to_idx(0, y)] = TileType::Wall;
    map[xy_to_idx(COLS - 1, y)] = TileType::Wall;
  }

  // randomly place walls
  let mut rng = RNG::new(42);
  for _i in 0..400 {
    let x = rng.range_i32(1..=COLS - 2);
    let y = rng.range_i32(1..=ROWS - 2);
    let id = xy_to_idx(x, y);

    if id != xy_to_idx(40, 25) {
      map[id] = TileType::Wall;
    }
  }

  map
}

// The background is not drawn per tile: it is black, the same color the frame
// is cleared to.
pub fn draw_map(map: &[TileType], d: &mut impl RaylibDraw, font: &Font, cell_w: i32, cell_h: i32) {
  for (idx, tile) in map.iter().enumerate() {
    let (glyph, fg) = match tile {
      TileType::Floor => ('.', Color::new(128, 128, 128, 255)),
      TileType::Wall => ('#', Color::new(0, 255, 0, 255)),
    };

    let x = (idx as i32 % COLS) * cell_w;
    let y = (idx as i32 / COLS) * cell_h;
    d.draw_text_codepoint(
      font,
      glyph as i32,
      Vector2::new(x as f32, y as f32),
      FONT_SIZE as f32,
      fg,
    );
  }
}
