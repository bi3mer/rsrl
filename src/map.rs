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

pub struct Map {
  pub tiles: Vec<TileType>,
  pub rooms: Vec<Rect>,
  pub width: i32,
  pub height: i32,
}

impl Map {
  /// Convert (x,y) coordinates to an index in `tiles`
  pub fn xy_to_idx(&self, x: i32, y: i32) -> usize {
    debug_assert!(x >= 0);
    debug_assert!(y >= 0);

    (y * self.width + x) as usize
  }

  /// Generate a map with wall tiles around the edge and random tiles inside. No
  /// guarantee that it will look good.
  #[expect(dead_code)]
  pub fn new_map_random(walls: u16) -> Map {
    let mut map = Map {
      tiles: vec![TileType::Floor; (COLS * ROWS) as usize],
      rooms: Vec::new(),
      width: COLS,
      height: ROWS,
    };

    // make boundary walls
    for x in 0..map.width {
      let top = map.xy_to_idx(x, 0);
      let bottom = map.xy_to_idx(x, map.height - 1);
      map.tiles[top] = TileType::Wall;
      map.tiles[bottom] = TileType::Wall;
    }

    for y in 0..map.height {
      let left = map.xy_to_idx(0, y);
      let right = map.xy_to_idx(map.width - 1, y);
      map.tiles[left] = TileType::Wall;
      map.tiles[right] = TileType::Wall;
    }

    // randomly place walls
    let mut rng = RNG::new(42);
    for _i in 0..walls {
      let x = rng.range_i32(1..=map.width - 2);
      let y = rng.range_i32(1..=map.height - 2);
      let id = map.xy_to_idx(x, y);

      if id != map.xy_to_idx(40, 25) {
        map.tiles[id] = TileType::Wall;
      }
    }

    map
  }

  fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
    for x in min(x1, x2)..=max(x1, x2) {
      let idx = self.xy_to_idx(x, y);
      if idx > 0 && idx < (self.width * self.height) as usize {
        self.tiles[idx] = TileType::Floor;
      }
    }
  }

  fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
    for y in min(y1, y2)..=max(y1, y2) {
      let idx = self.xy_to_idx(x, y);
      if idx > 0 && idx < (self.width * self.height) as usize {
        self.tiles[idx] = TileType::Floor;
      }
    }
  }

  fn apply_room_to_map(&mut self, room: &Rect) {
    for y in room.y1 + 1..=room.y2 {
      for x in room.x1 + 1..=room.x2 {
        let idx = self.xy_to_idx(x, y);
        self.tiles[idx] = TileType::Floor
      }
    }
  }

  pub fn new_map_rooms_and_corridors() -> Map {
    let mut map = Map {
      tiles: vec![TileType::Wall; (COLS * ROWS) as usize],
      rooms: Vec::new(), // this should be an array of size 30
      width: COLS,
      height: ROWS,
    };

    const MAX_ROOMS: i32 = 30;
    const MIN_SIZE: i32 = 6;
    const MAX_SIZE: i32 = 10;

    let mut rng = RNG::new(42);
    for _ in 0..MAX_ROOMS {
      let w = rng.range_i32(MIN_SIZE..=MAX_SIZE);
      let h = rng.range_i32(MIN_SIZE..=MAX_SIZE);
      let new_room = Rect::new(
        rng.range_i32(1..(map.width - w)) - 1,
        rng.range_i32(1..(map.height - h)) - 1,
        w,
        h,
      );

      let mut intersects = false;
      for other_room in map.rooms.iter() {
        if new_room.intersect(other_room) {
          intersects = true;
          break;
        }
      }

      if !intersects {
        map.apply_room_to_map(&new_room);

        if !map.rooms.is_empty() {
          let (new_x, new_y) = new_room.center();
          let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center();

          if rng.next_bool() {
            map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
            map.apply_vertical_tunnel(prev_y, new_y, new_x);
          } else {
            map.apply_vertical_tunnel(prev_y, new_y, prev_x);
            map.apply_horizontal_tunnel(prev_x, new_x, new_y);
          }
        }

        map.rooms.push(new_room);
      }
    }

    map
  }

  // The background is not drawn per tile: it is black, the same color the frame
  // is cleared to.
  pub fn draw_map(&self, d: &mut impl RaylibDraw, font: &Font, cell_w: i32, cell_h: i32) {
    for (idx, tile) in self.tiles.iter().enumerate() {
      let (glyph, fg) = match tile {
        TileType::Floor => ('.', Color::new(128, 128, 128, 255)),
        TileType::Wall => ('#', Color::new(0, 255, 0, 255)),
      };

      let x = (idx as i32 % self.width) * cell_w;
      let y = (idx as i32 / self.width) * cell_h;
      d.draw_text_codepoint(
        font,
        glyph as i32,
        Vector2::new(x as f32, y as f32),
        FONT_SIZE as f32,
        fg,
      );
    }
  }
}
