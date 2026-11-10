use std::cmp::{max, min};

use raylib::prelude::*;
use specs::prelude::*;
use specs_derive::Component;

mod rl;

const COLS: i32 = 80;
const ROWS: i32 = 50;
const FONT_SIZE: i32 = 18;

const FONT_TTF: &[u8] = include_bytes!("../assets/fonts/LiberationMono-Regular.ttf");

#[derive(Component, Debug)]
struct Position {
  x: i32,
  y: i32,
}

#[derive(Component)]
struct Renderable {
  glyph: char,
  fg: Color,
  bg: Color,
}

#[derive(Component)]
struct LeftMover;

// 'a is a lifetime. It can be any name, but the important point is that the
// lifetime is for the duration of the run function. Once the run function is
// complete, the storage is returned to World.
impl<'a> System<'a> for LeftMover {
  type SystemData = (ReadStorage<'a, LeftMover>, WriteStorage<'a, Position>);

  fn run(&mut self, (lefty, mut pos): Self::SystemData) {
    for (_lefty, pos) in (&lefty, &mut pos).join() {
      pos.x -= 1;
      if pos.x < 0 {
        pos.x = COLS - 1;
      }
    }
  }
}

#[derive(Component, Debug)]
struct Player;

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
  let mut positions = ecs.write_storage::<Position>();
  let mut players = ecs.write_storage::<Player>();

  for (_player, pos) in (&mut players, &mut positions).join() {
    pos.x = min(COLS - 1, max(0, pos.x + delta_x));
    pos.y = min(ROWS - 1, max(0, pos.y + delta_y));
  }
}

fn player_input(gs: &mut State, rl: &RaylibHandle) {
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

struct State {
  ecs: World,
}

impl State {
  fn run_systems(&mut self) {
    LeftMover.run_now(&self.ecs);
    self.ecs.maintain();
  }
}

fn main() {
  // The window must exist before a font can load, so it opens at a placeholder
  // size and is resized once the font is measured.
  let (mut rl, thread) = raylib::init().size(640, 480).title("rsrl").build();
  rl.set_target_fps(30);

  // raylib only rasterizes the glyphs you ask for, so list every character drawn.
  let glyphs: String = (' '..='~').chain(['☺']).collect();
  let font = rl
    .load_font_from_memory(&thread, ".ttf", FONT_TTF, FONT_SIZE, Some(&glyphs))
    .expect("load Liberation Mono");

  let (cell_w, cell_h) = rl::cell_size(&font);
  let (window_w, window_h) = (COLS * cell_w, ROWS * cell_h);
  rl.set_window_size(window_w, window_h);
  rl::center_window(&mut rl, window_w, window_h);

  let mut game_state = State { ecs: World::new() };
  game_state.ecs.register::<Position>();
  game_state.ecs.register::<Renderable>();
  game_state.ecs.register::<LeftMover>();
  game_state.ecs.register::<Player>();

  game_state
    .ecs
    .create_entity()
    .with(Player)
    .with(Position { x: 40, y: 25 })
    .with(Renderable {
      glyph: '@',
      fg: Color::YELLOW,
      bg: Color::BLACK,
    })
    .build();

  for i in 0..10 {
    game_state
      .ecs
      .create_entity()
      .with(LeftMover)
      .with(Position { x: i * 7, y: 20 })
      .with(Renderable {
        glyph: 'E',
        fg: Color::new(255, 0, 0, 255),
        bg: Color::BLACK,
      })
      .build();
  }

  while !rl.window_should_close() {
    //// Update
    player_input(&mut game_state, &rl);
    game_state.run_systems();

    //// Render
    let mut d = rl.begin_drawing(&thread);
    d.clear_background(Color::BLACK);

    let positions = game_state.ecs.read_storage::<Position>();
    let renderables = game_state.ecs.read_storage::<Renderable>();

    for (pos, render) in (&positions, &renderables).join() {
      let (x, y) = (pos.x * cell_w, pos.y * cell_h);
      d.draw_rectangle(x, y, cell_w, cell_h, render.bg);
      d.draw_text_codepoint(
        &font,
        render.glyph as i32,
        Vector2::new(x as f32, y as f32),
        FONT_SIZE as f32,
        render.fg,
      );
    }
  }
}
