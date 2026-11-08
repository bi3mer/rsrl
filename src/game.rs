//! Game state, and how input changes it. Rendering only ever reads this.

use winit::keyboard::{Key, NamedKey};

use crate::grid::{Color, Grid, BLACK, WHITE};

const AMBER: Color = Color::rgb(255, 191, 0);
const GRAY: Color = Color::rgb(128, 128, 128);
const NAVY: Color = Color::rgb(0, 0, 96);

/// The map size in tiles. This is game state and does not depend on the window.
const MAP_WIDTH: u8 = 8;
const MAP_HEIGHT: u8 = 2;

/// Where map tile `(0, 0)` appears on the screen, in grid cells.
const MAP_ORIGIN: (u32, u32) = (3, 7);

pub struct Game {
  /// Position in map tiles, not screen cells.
  player: (u8, u8),
}

impl Default for Game {
  fn default() -> Self {
    Self { player: (2, 1) }
  }
}

impl Game {
  /// Applies a key press and returns whether the screen needs a new frame.
  pub fn handle_key(&mut self, key: &Key) -> bool {
    let (dx, dy): (i8, i8) = match key {
      Key::Named(NamedKey::ArrowLeft) => (-1, 0),
      Key::Named(NamedKey::ArrowRight) => (1, 0),
      Key::Named(NamedKey::ArrowUp) => (0, -1),
      Key::Named(NamedKey::ArrowDown) => (0, 1),
      _ => return false,
    };
    let x = self.player.0.saturating_add_signed(dx);
    let y = self.player.1.saturating_add_signed(dy);
    let moved_to = (x.min(MAP_WIDTH - 1), y.min(MAP_HEIGHT - 1));
    let changed = moved_to != self.player;
    self.player = moved_to;
    changed
  }

  /// Draws the current state. The room and sample text are placeholders.
  pub fn draw(&self, grid: &mut Grid) {
    grid.clear();
    grid.print(2, 1, "rsrl", AMBER, BLACK);
    grid.print(
      2,
      3,
      "The quick brown fox jumps over the lazy dog.",
      WHITE,
      BLACK,
    );
    grid.print(
      2,
      4,
      "0123456789 !\"#$%&'()*+,-./:;<=>?[]^_{|}~",
      GRAY,
      BLACK,
    );
    grid.print(2, 6, "┌────────┐", GRAY, BLACK);
    grid.print(2, 7, "│.....#..│", GRAY, BLACK);
    grid.print(2, 8, "│.....#..│", GRAY, BLACK);
    grid.print(2, 9, "└────────┘", GRAY, BLACK);
    grid.print(15, 8, " colored background ", WHITE, NAVY);
    let x = MAP_ORIGIN.0 + u32::from(self.player.0);
    let y = MAP_ORIGIN.1 + u32::from(self.player.1);
    grid.put(x, y, '@', AMBER, BLACK);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn arrow(key: NamedKey) -> Key {
    Key::Named(key)
  }

  #[test]
  fn arrow_keys_move_the_player_one_tile() {
    let mut game = Game { player: (3, 0) };
    assert!(game.handle_key(&arrow(NamedKey::ArrowLeft)));
    assert_eq!(game.player, (2, 0));
    assert!(game.handle_key(&arrow(NamedKey::ArrowRight)));
    assert_eq!(game.player, (3, 0));
    assert!(game.handle_key(&arrow(NamedKey::ArrowDown)));
    assert_eq!(game.player, (3, 1));
    assert!(game.handle_key(&arrow(NamedKey::ArrowUp)));
    assert_eq!(game.player, (3, 0));
  }

  #[test]
  fn unbound_keys_need_no_frame() {
    let mut game = Game { player: (3, 1) };
    assert!(!game.handle_key(&Key::Character("a".into())));
    assert!(!game.handle_key(&arrow(NamedKey::Enter)));
    assert_eq!(game.player, (3, 1));
  }

  #[test]
  fn walking_into_a_map_edge_needs_no_frame() {
    let mut game = Game { player: (0, 0) };
    assert!(!game.handle_key(&arrow(NamedKey::ArrowLeft)));
    assert!(!game.handle_key(&arrow(NamedKey::ArrowUp)));

    let last = (MAP_WIDTH - 1, MAP_HEIGHT - 1);
    let mut game = Game { player: last };
    assert!(!game.handle_key(&arrow(NamedKey::ArrowRight)));
    assert!(!game.handle_key(&arrow(NamedKey::ArrowDown)));
    assert_eq!(game.player, last);
  }

  #[test]
  fn draw_converts_map_tiles_to_screen_cells() {
    let game = Game { player: (2, 1) };
    let mut grid = Grid::new(10, 12);
    game.draw(&mut grid);
    let (x, y) = (MAP_ORIGIN.0 + 2, MAP_ORIGIN.1 + 1);
    assert_eq!(grid.cells()[(y * 10 + x) as usize].ch, '@');
  }
}
