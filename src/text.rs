//! Renders a [`Grid`] with a monospaced font.
//!
//! [`FontAtlas`] rasterizes each supported character into a cell-sized slot of a texture.
//! [`TextRenderer`] uploads the grid and draws it in one call.

use std::collections::HashMap;
use std::ops::{Range, RangeInclusive};

use bytemuck::{Pod, Zeroable};
use fontdue::{Font, FontSettings};

use crate::grid::Grid;

const FONT_BYTES: &[u8] = include_bytes!("../assets/fonts/LiberationMono-Regular.ttf");

/// Font size in logical pixels. The window scale factor multiplies this.
const FONT_SIZE: f32 = 16.0;

const ASCII: RangeInclusive<char> = ' '..='~';

/// Box drawing and block elements, which must join up with their neighboring cells.
const LINE_ART: RangeInclusive<char> = '\u{2500}'..='\u{259f}';

const CHARACTERS: [RangeInclusive<char>; 2] = [ASCII, LINE_ART];

/// Coverage at or above this value counts as solid ink rather than antialiasing.
const SOLID_INK: u8 = 128;

/// Drawn in place of any character the atlas does not contain.
const FALLBACK: char = '?';

const ATLAS_COLUMNS: u32 = 16;

/// A CPU-side, single-channel coverage texture with one cell-sized slot per character.
struct FontAtlas {
  cell_width: u32,
  cell_height: u32,
  width: u32,
  height: u32,
  pixels: Vec<u8>,
  slots: HashMap<char, u32>,
}

impl FontAtlas {
  fn new(font: &Font, px: f32) -> Self {
    let line = font
      .horizontal_line_metrics(px)
      .expect("the bundled font has horizontal metrics");
    let cell_width = font.metrics('M', px).advance_width.ceil() as u32;
    let cell_height = line.new_line_size.ceil() as u32;
    let baseline = line.ascent.round() as i32;

    let characters: Vec<char> = CHARACTERS
      .into_iter()
      .flatten()
      .filter(|ch| font.has_glyph(*ch))
      .collect();
    let width = ATLAS_COLUMNS * cell_width;
    let height = (characters.len() as u32).div_ceil(ATLAS_COLUMNS) * cell_height;

    let mut atlas = Self {
      cell_width,
      cell_height,
      width,
      height,
      pixels: vec![0; (width * height) as usize],
      slots: HashMap::with_capacity(characters.len()),
    };
    for (slot, ch) in (0..).zip(characters) {
      let (metrics, coverage) = font.rasterize(ch, px);
      let left = metrics.xmin;
      let top = baseline - metrics.ymin - metrics.height as i32;
      atlas.blit(slot, left, top, metrics.width, &coverage);
      if LINE_ART.contains(&ch) {
        atlas.extend_to_cell_edges(slot);
      }
      atlas.slots.insert(ch, slot);
    }
    atlas
  }

  fn pixel_index(&self, slot: u32, x: u32, y: u32) -> usize {
    let slot_x = slot % ATLAS_COLUMNS * self.cell_width;
    let slot_y = slot / ATLAS_COLUMNS * self.cell_height;
    ((slot_y + y) * self.width + slot_x + x) as usize
  }

  /// Closes the gap between line art and the cell edge.
  ///
  /// Cell sizes round up to whole pixels, so a line that spans the font's line height can stop
  /// a pixel or two short of the cell and leave gaps in walls and frames. Ink within a quarter
  /// of a cell of an edge is copied out to that edge; ink further away, such as the middle of
  /// a horizontal line seen from the top edge, is left alone.
  fn extend_to_cell_edges(&mut self, slot: u32) {
    let (width, height) = (self.cell_width, self.cell_height);
    let solid = |atlas: &Self, x, y| atlas.pixels[atlas.pixel_index(slot, x, y)] >= SOLID_INK;

    let solid_rows: Vec<u32> = (0..height)
      .filter(|y| (0..width).any(|x| solid(self, x, *y)))
      .collect();
    let (Some(&top), Some(&bottom)) = (solid_rows.first(), solid_rows.last()) else {
      return;
    };
    if top <= height / 4 {
      self.copy_row(slot, top, 0..top);
    }
    if height - 1 - bottom <= height / 4 {
      self.copy_row(slot, bottom, bottom + 1..height);
    }

    let solid_columns: Vec<u32> = (0..width)
      .filter(|x| (0..height).any(|y| solid(self, *x, y)))
      .collect();
    let (Some(&left), Some(&right)) = (solid_columns.first(), solid_columns.last()) else {
      return;
    };
    if left <= width / 4 {
      self.copy_column(slot, left, 0..left);
    }
    if width - 1 - right <= width / 4 {
      self.copy_column(slot, right, right + 1..width);
    }
  }

  fn copy_row(&mut self, slot: u32, source: u32, destinations: Range<u32>) {
    for y in destinations {
      for x in 0..self.cell_width {
        let (to, from) = (
          self.pixel_index(slot, x, y),
          self.pixel_index(slot, x, source),
        );
        self.pixels[to] = self.pixels[from];
      }
    }
  }

  fn copy_column(&mut self, slot: u32, source: u32, destinations: Range<u32>) {
    for x in destinations {
      for y in 0..self.cell_height {
        let (to, from) = (
          self.pixel_index(slot, x, y),
          self.pixel_index(slot, source, y),
        );
        self.pixels[to] = self.pixels[from];
      }
    }
  }

  /// Copies a glyph bitmap into a slot at `(left, top)`, clipping anything outside the cell.
  fn blit(&mut self, slot: u32, left: i32, top: i32, glyph_width: usize, coverage: &[u8]) {
    if glyph_width == 0 {
      return;
    }
    for (row, line) in (0..).zip(coverage.chunks_exact(glyph_width)) {
      for (column, value) in (0..).zip(line) {
        let x = left + column;
        let y = top + row;
        if (0..self.cell_width as i32).contains(&x) && (0..self.cell_height as i32).contains(&y) {
          let index = self.pixel_index(slot, x as u32, y as u32);
          self.pixels[index] = *value;
        }
      }
    }
  }

  fn slot(&self, ch: char) -> u32 {
    self
      .slots
      .get(&ch)
      .copied()
      .unwrap_or(self.slots[&FALLBACK])
  }
}

/// Matches `Params` in `text.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuParams {
  grid: [u32; 2],
  cell: [u32; 2],
  atlas_columns: u32,
  padding: u32,
}

/// Matches `Cell` in `text.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuCell {
  glyph: u32,
  fg: u32,
  bg: u32,
}

pub struct TextRenderer {
  font: Font,
  atlas: FontAtlas,
  pipeline: wgpu::RenderPipeline,
  params: wgpu::Buffer,
  cells: wgpu::Buffer,
  atlas_view: wgpu::TextureView,
  bind_group: wgpu::BindGroup,
}

impl TextRenderer {
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    format: wgpu::TextureFormat,
    scale: f32,
  ) -> Self {
    let font = Font::from_bytes(FONT_BYTES, FontSettings::default()).unwrap();
    let atlas = FontAtlas::new(&font, FONT_SIZE * scale);
    let atlas_view = upload_atlas(device, queue, &atlas);

    let shader = device.create_shader_module(wgpu::include_wgsl!("text.wgsl"));
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("text"),
      layout: None,
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: None,
        compilation_options: Default::default(),
        buffers: &[],
      },
      primitive: Default::default(),
      depth_stencil: None,
      multisample: Default::default(),
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: None,
        compilation_options: Default::default(),
        targets: &[Some(format.into())],
      }),
      multiview_mask: None,
      cache: None,
    });

    let params = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("text params"),
      size: size_of::<GpuParams>() as u64,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    let cells = create_cell_buffer(device, 1);
    let bind_group = create_bind_group(device, &pipeline, &params, &cells, &atlas_view);

    Self {
      font,
      atlas,
      pipeline,
      params,
      cells,
      atlas_view,
      bind_group,
    }
  }

  /// How many whole cells fit in a surface of the given pixel size, at least one in each axis.
  pub fn grid_size(&self, width: u32, height: u32) -> (u32, u32) {
    (
      (width / self.atlas.cell_width).max(1),
      (height / self.atlas.cell_height).max(1),
    )
  }

  /// Rasterizes the font again for a new window scale factor.
  pub fn set_scale(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, scale: f32) {
    self.atlas = FontAtlas::new(&self.font, FONT_SIZE * scale);
    self.atlas_view = upload_atlas(device, queue, &self.atlas);
    self.rebuild_bind_group(device);
  }

  /// Uploads the grid. Call this once per frame, before [`Self::draw`].
  pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, grid: &Grid) {
    let cells: Vec<GpuCell> = grid
      .cells()
      .iter()
      .map(|cell| GpuCell {
        glyph: self.atlas.slot(cell.ch),
        fg: cell.fg.0,
        bg: cell.bg.0,
      })
      .collect();

    let bytes: &[u8] = bytemuck::cast_slice(&cells);
    if bytes.len() as u64 > self.cells.size() {
      self.cells = create_cell_buffer(device, cells.len());
      self.rebuild_bind_group(device);
    }
    queue.write_buffer(&self.cells, 0, bytes);

    let params = GpuParams {
      grid: [grid.cols(), grid.rows()],
      cell: [self.atlas.cell_width, self.atlas.cell_height],
      atlas_columns: ATLAS_COLUMNS,
      padding: 0,
    };
    queue.write_buffer(&self.params, 0, bytemuck::bytes_of(&params));
  }

  pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
    pass.set_pipeline(&self.pipeline);
    pass.set_bind_group(0, &self.bind_group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn rebuild_bind_group(&mut self, device: &wgpu::Device) {
    self.bind_group = create_bind_group(
      device,
      &self.pipeline,
      &self.params,
      &self.cells,
      &self.atlas_view,
    );
  }
}

fn create_cell_buffer(device: &wgpu::Device, len: usize) -> wgpu::Buffer {
  device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("text cells"),
    size: (len * size_of::<GpuCell>()) as u64,
    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    mapped_at_creation: false,
  })
}

fn create_bind_group(
  device: &wgpu::Device,
  pipeline: &wgpu::RenderPipeline,
  params: &wgpu::Buffer,
  cells: &wgpu::Buffer,
  atlas_view: &wgpu::TextureView,
) -> wgpu::BindGroup {
  device.create_bind_group(&wgpu::BindGroupDescriptor {
    label: Some("text"),
    layout: &pipeline.get_bind_group_layout(0),
    entries: &[
      wgpu::BindGroupEntry {
        binding: 0,
        resource: params.as_entire_binding(),
      },
      wgpu::BindGroupEntry {
        binding: 1,
        resource: cells.as_entire_binding(),
      },
      wgpu::BindGroupEntry {
        binding: 2,
        resource: wgpu::BindingResource::TextureView(atlas_view),
      },
    ],
  })
}

fn upload_atlas(
  device: &wgpu::Device,
  queue: &wgpu::Queue,
  atlas: &FontAtlas,
) -> wgpu::TextureView {
  let size = wgpu::Extent3d {
    width: atlas.width,
    height: atlas.height,
    depth_or_array_layers: 1,
  };
  let texture = device.create_texture(&wgpu::TextureDescriptor {
    label: Some("text atlas"),
    size,
    mip_level_count: 1,
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    format: wgpu::TextureFormat::R8Unorm,
    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    view_formats: &[],
  });
  queue.write_texture(
    texture.as_image_copy(),
    &atlas.pixels,
    wgpu::TexelCopyBufferLayout {
      offset: 0,
      bytes_per_row: Some(atlas.width),
      rows_per_image: None,
    },
    size,
  );
  texture.create_view(&Default::default())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn atlas() -> FontAtlas {
    let font = Font::from_bytes(FONT_BYTES, FontSettings::default()).unwrap();
    FontAtlas::new(&font, FONT_SIZE)
  }

  fn slot_coverage(atlas: &FontAtlas, ch: char) -> u32 {
    let slot = atlas.slot(ch);
    let left = slot % ATLAS_COLUMNS * atlas.cell_width;
    let top = slot / ATLAS_COLUMNS * atlas.cell_height;
    (top..top + atlas.cell_height)
      .flat_map(|y| (left..left + atlas.cell_width).map(move |x| (x, y)))
      .map(|(x, y)| u32::from(atlas.pixels[(y * atlas.width + x) as usize]))
      .sum()
  }

  #[test]
  fn atlas_dimensions_are_whole_cells() {
    let atlas = atlas();
    assert_eq!(atlas.width, ATLAS_COLUMNS * atlas.cell_width);
    assert_eq!(atlas.height % atlas.cell_height, 0);
    assert_eq!(atlas.pixels.len(), (atlas.width * atlas.height) as usize);
  }

  #[test]
  fn every_ascii_character_has_its_own_slot() {
    let atlas = atlas();
    assert!((' '..='~').all(|ch| atlas.slots.contains_key(&ch)));
  }

  #[test]
  fn printable_glyphs_have_coverage_and_space_has_none() {
    let atlas = atlas();
    assert!(slot_coverage(&atlas, '@') > 0);
    assert!(slot_coverage(&atlas, '\u{2500}') > 0);
    assert_eq!(slot_coverage(&atlas, ' '), 0);
  }

  /// Whether `ch` has solid ink at any of the given cell-relative pixels.
  fn inked(atlas: &FontAtlas, ch: char, at: impl Iterator<Item = (u32, u32)>) -> bool {
    let slot = atlas.slot(ch);
    let left = slot % ATLAS_COLUMNS * atlas.cell_width;
    let top = slot / ATLAS_COLUMNS * atlas.cell_height;
    at.map(|(x, y)| atlas.pixels[((top + y) * atlas.width + left + x) as usize])
      .any(|value| value >= SOLID_INK)
  }

  fn atlases_at_several_sizes() -> impl Iterator<Item = FontAtlas> {
    let font = Font::from_bytes(FONT_BYTES, FontSettings::default()).unwrap();
    [12.0, 13.5, 16.0, 20.0, 24.0, 32.0]
      .into_iter()
      .map(move |px| FontAtlas::new(&font, px))
  }

  #[test]
  fn line_glyphs_reach_the_cell_edges_they_join() {
    for atlas in atlases_at_several_sizes() {
      let (w, h) = (atlas.cell_width, atlas.cell_height);
      assert!(inked(&atlas, '\u{2502}', (0..w).map(|x| (x, 0))));
      assert!(inked(&atlas, '\u{2502}', (0..w).map(|x| (x, h - 1))));
      assert!(inked(&atlas, '\u{2500}', (0..h).map(|y| (0, y))));
      assert!(inked(&atlas, '\u{2500}', (0..h).map(|y| (w - 1, y))));
    }
  }

  #[test]
  fn line_glyphs_stay_off_the_cell_edges_they_do_not_join() {
    for atlas in atlases_at_several_sizes() {
      let (w, h) = (atlas.cell_width, atlas.cell_height);
      assert!(!inked(&atlas, '\u{2502}', (0..h).map(|y| (0, y))));
      assert!(!inked(&atlas, '\u{2502}', (0..h).map(|y| (w - 1, y))));
      assert!(!inked(&atlas, '\u{2500}', (0..w).map(|x| (x, 0))));
      assert!(!inked(&atlas, '\u{2500}', (0..w).map(|x| (x, h - 1))));
    }
  }

  #[test]
  fn missing_characters_use_the_fallback_slot() {
    let atlas = atlas();
    assert_eq!(atlas.slot('\u{1f600}'), atlas.slot(FALLBACK));
  }

  #[test]
  fn gpu_structs_match_the_shader_layout() {
    assert_eq!(size_of::<GpuParams>(), 24);
    assert_eq!(size_of::<GpuCell>(), 12);
  }
}
