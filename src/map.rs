use raylib::prelude::*;
use rsrl::rng::RNG;

use std::cmp::{max, min};

use crate::rect::Rect;
use crate::{COLS, FONT_SIZE, ROWS};

#[derive(PartialEq, Clone, Copy)]
pub enum TileType {
  Wall,
  Floor,
}

pub fn xy_to_idx(x: i32, y: i32) -> usize {
  debug_assert!(x >= 0);
  debug_assert!(y >= 0);

  (y * COLS + x) as usize
}

/// Generate a map with wall tiles around the edge and random tiles inside. No
/// guarantee that it will look good.
pub fn new_map_random(walls: u16) -> Vec<TileType> {
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
  for _i in 0..walls {
    let x = rng.range_i32(1..=COLS - 2);
    let y = rng.range_i32(1..=ROWS - 2);
    let id = xy_to_idx(x, y);

    if id != xy_to_idx(40, 25) {
      map[id] = TileType::Wall;
    }
  }

  map
}

fn apply_horizontal_tunnel(map: &mut [TileType], x1: i32, x2: i32, y: i32) {
  for x in min(x1, x2)..=max(x1, x2) {
    let idx = xy_to_idx(x, y);
    if idx > 0 && idx < (COLS * ROWS) as usize {
      map[idx as usize] = TileType::Floor;
    }
  }
}

fn apply_vertical_tunnel(map: &mut [TileType], y1: i32, y2: i32, x: i32) {
  for y in min(y1, y2)..=max(y1, y2) {
    let idx = xy_to_idx(x, y);
    if idx > 0 && idx < (COLS * ROWS) as usize {
      map[idx as usize] = TileType::Floor;
    }
  }
}

fn apply_room_to_map(room: &Rect, map: &mut [TileType]) {
  for y in room.y1 + 1..=room.y2 {
    for x in room.x1 + 1..=room.x2 {
      map[xy_to_idx(x, y)] = TileType::Floor
    }
  }
}

pub fn new_map_rooms_and_corridors() -> (Vec<TileType>, Vec<Rect>) {
  let mut map = vec![TileType::Wall; (COLS * ROWS) as usize];

  let mut rooms: Vec<Rect> = Vec::new(); // this should be an array of size 30
  const MAX_ROOMS: i32 = 30;
  const MIN_SIZE: i32 = 6;
  const MAX_SIZE: i32 = 10;

  let mut rng = RNG::new(42);
  for _ in 0..MAX_ROOMS {
    let w = rng.range_i32(MIN_SIZE..=MAX_SIZE);
    let h = rng.range_i32(MIN_SIZE..=MAX_SIZE);
    let new_room = Rect::new(
      rng.range_i32(1..(COLS - w)) - 1,
      rng.range_i32(1..(ROWS - h)) - 1,
      w,
      h,
    );

    let mut intersects = false;
    for other_room in rooms.iter() {
      if new_room.intersect(other_room) {
        intersects = true;
        break;
      }
    }

    if !intersects {
      apply_room_to_map(&new_room, &mut map);

      if !rooms.is_empty() {
        let (new_x, new_y) = new_room.center();
        let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

        if rng.next_bool() {
          apply_horizontal_tunnel(&mut map, prev_x, new_x, prev_y);
          apply_vertical_tunnel(&mut map, prev_y, new_y, new_x);
        } else {
          apply_vertical_tunnel(&mut map, prev_y, new_y, prev_x);
          apply_horizontal_tunnel(&mut map, prev_x, new_x, new_y);
        }
      }

      rooms.push(new_room);
    }
  }

  (map, rooms)
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
