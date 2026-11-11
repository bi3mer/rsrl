use std::cmp::{max, min};

use raylib::prelude::*;
use specs::prelude::*;

use crate::components::*;
use crate::map::{xy_to_idx, TileType};
use crate::{rl, State, COLS, ROWS};

pub fn player_input(gs: &mut State, rl: &RaylibHandle) {
  if rl::pressed(rl, KeyboardKey::KEY_LEFT) {
    try_move_player(-1, 0, &mut gs.ecs);
  } else if rl::pressed(rl, KeyboardKey::KEY_RIGHT) {
    try_move_player(1, 0, &mut gs.ecs);
  } else if rl::pressed(rl, KeyboardKey::KEY_UP) {
    try_move_player(0, -1, &mut gs.ecs);
  } else if rl::pressed(rl, KeyboardKey::KEY_DOWN) {
    try_move_player(0, 1, &mut gs.ecs);
  }
}

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
  let mut positions = ecs.write_storage::<Position>();
  let mut players = ecs.write_storage::<Player>();
  let map = ecs.fetch::<Vec<TileType>>();

  for (_player, pos) in (&mut players, &mut positions).join() {
    let x = min(COLS - 1, max(0, pos.x + delta_x));
    let y = min(ROWS - 1, max(0, pos.y + delta_y));
    let dst_id = xy_to_idx(x, y);

    debug_assert!(dst_id < map.len());

    if map[dst_id] != TileType::Wall {
      pos.x = x;
      pos.y = y;
    }
  }
}
