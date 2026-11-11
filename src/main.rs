mod components;
mod map;
mod player;
mod rect;
mod systems;

use components::*;
use map::*;
use player::*;

use raylib::prelude::*;
use specs::prelude::*;

mod rl;

const COLS: i32 = 80;
const ROWS: i32 = 40;
const FONT_SIZE: i32 = 18;

const FONT_TTF: &[u8] = include_bytes!("../assets/fonts/LiberationMono-Regular.ttf");

struct State {
  ecs: World,
}

impl State {
  fn run_systems(&mut self) {
    components::LeftMover.run_now(&self.ecs);
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

  // game_state.ecs.insert(new_random_map(400));
  let (map, rooms) = new_map_rooms_and_corridors();
  game_state.ecs.insert(map);

  let (player_x, player_y) = rooms[0].center();
  game_state
    .ecs
    .create_entity()
    .with(Player)
    .with(Position {
      x: player_x,
      y: player_y,
    })
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

    let map = game_state.ecs.fetch::<Vec<TileType>>();
    draw_map(&map, &mut d, &font, cell_w, cell_h);

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
