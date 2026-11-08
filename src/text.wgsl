// Draws the whole character grid with one fullscreen triangle.
//
// Each atlas slot is exactly one cell in size, so every screen pixel maps to one atlas texel.
// That makes textureLoad exact: no sampler and no filtering.

struct Params {
  grid: vec2u,
  cell: vec2u,
  atlas_columns: u32,
}

struct Cell {
  glyph: u32,
  fg: u32,
  bg: u32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> cells: array<Cell>;
@group(0) @binding(2) var atlas: texture_2d<f32>;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
  let corner = vec2f(f32((index << 1u) & 2u), f32(index & 2u));
  return vec4f(corner * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4f) -> @location(0) vec4f {
  let pixel = vec2u(position.xy);
  let cell_position = pixel / params.cell;

  // The window size is rarely a whole number of cells, which leaves a margin.
  if any(cell_position >= params.grid) {
    return vec4f(0.0, 0.0, 0.0, 1.0);
  }

  let cell = cells[cell_position.y * params.grid.x + cell_position.x];
  let slot = vec2u(cell.glyph % params.atlas_columns, cell.glyph / params.atlas_columns);
  let coverage = textureLoad(atlas, slot * params.cell + pixel % params.cell, 0).r;
  return mix(unpack4x8unorm(cell.bg), unpack4x8unorm(cell.fg), coverage);
}
