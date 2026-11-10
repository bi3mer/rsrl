//! Helpers that depend on raylib, kept apart from the game logic in main.rs.

use raylib::prelude::*;

/// True on the frame a key goes down and on each OS key-repeat while held.
pub fn pressed(rl: &RaylibHandle, key: KeyboardKey) -> bool {
  rl.is_key_pressed(key) || rl.is_key_pressed_repeat(key)
}

/// The size of one grid cell in pixels, measured from one glyph of `font` at the
/// size it was loaded. Assumes a monospace font, so every glyph has the same
/// width. Rounds up so neighboring glyphs never overlap.
pub fn cell_size(font: &Font) -> (i32, i32) {
  let size = font.measure_text("M", font.base_size() as f32, 0.0);
  (size.x.ceil() as i32, size.y.ceil() as i32)
}

/// Moves a window of the given size to the center of the monitor it is on.
/// Resizing keeps the top-left corner fixed, so a window that grows needs this
/// to stay on screen.
pub fn center_window(rl: &mut RaylibHandle, width: i32, height: i32) {
  let monitor = raylib::core::window::get_current_monitor();
  let monitor_w = raylib::core::window::get_monitor_width(monitor);
  let monitor_h = raylib::core::window::get_monitor_height(monitor);
  rl.set_window_position((monitor_w - width) / 2, (monitor_h - height) / 2);
}
