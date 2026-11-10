use std::cmp::{max, min};

use rltk::{GameState, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use specs_derive::Component;

#[derive(Component, Debug)]
struct Position {
  x: i32,
  y: i32,
}

#[derive(Component)]
struct Renderable {
  glyph: rltk::FontCharType,
  fg: RGB, // amateur hour RGB implementation
  bg: RGB,
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
        pos.x = 79;
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
    pos.x = min(79, max(0, pos.x + delta_x));
    pos.y = min(49, max(0, pos.y + delta_y));
  }
}

fn player_input(gs: &mut State, ctx: &mut Rltk) {
  // Player movement
  match ctx.key {
    None => {} // Nothing happened
    Some(key) => match key {
      VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
      VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
      VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
      VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
      _ => {}
    },
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

impl GameState for State {
  fn tick(&mut self, ctx: &mut Rltk) {
    //// Update
    player_input(self, ctx);
    self.run_systems();

    //// Render
    // clear screen
    ctx.cls();

    // get relevant info for rendering
    let positions = self.ecs.read_storage::<Position>();
    let renderables = self.ecs.read_storage::<Renderable>();

    // this is a literal join, where it will only return entities with both
    // a position and a renderable component
    for (pos, render) in (&positions, &renderables).join() {
      ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
    }
  }
}

fn main() -> rltk::BError {
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
      glyph: rltk::to_cp437('@'),
      fg: RGB::named(rltk::YELLOW),
      bg: RGB::named(rltk::BLACK),
    })
    .build();

  for i in 0..10 {
    game_state
      .ecs
      .create_entity()
      .with(LeftMover {})
      .with(Position { x: i * 7, y: 20 })
      .with(Renderable {
        glyph: rltk::to_cp437('☺'),
        fg: RGB::named(rltk::RED),
        bg: RGB::named(rltk::BLACK),
      })
      .build();
  }

  let context = rltk::RltkBuilder::simple80x50()
    .with_title("rsrl")
    .build()?;

  rltk::main_loop(context, game_state)
}
