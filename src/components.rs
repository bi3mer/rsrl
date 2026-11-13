use raylib::prelude::Color;
use specs::prelude::*;
use specs_derive::Component;

use crate::point::Point;

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component)]
pub struct LeftMover;

#[derive(Component, Debug)]
pub struct Position {
  pub x: i32,
  pub y: i32,
}

#[derive(Component)]
pub struct Renderable {
  pub glyph: char,
  pub fg: Color,
  pub bg: Color,
}

#[derive(Component)]
pub struct Viewshed {
  pub visible_tiles: Vec<Point>,
  pub range: i32,
}
