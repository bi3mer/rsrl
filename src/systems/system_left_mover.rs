use specs::prelude::*;

use crate::components::*;
use crate::COLS;

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
