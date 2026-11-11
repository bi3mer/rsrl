use specs::prelude::*;

use crate::components::*;
use crate::COLS;

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
