//! Opens a window and draws a grid of monospaced text with wgpu.

mod game;
mod grid;
mod text;

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use game::Game;
use grid::Grid;
use text::TextRenderer;

struct Gpu {
  window: Arc<Window>,
  surface: wgpu::Surface<'static>,
  device: wgpu::Device,
  queue: wgpu::Queue,
  config: wgpu::SurfaceConfiguration,
  text: TextRenderer,
  grid: Grid,
}

impl Gpu {
  async fn new(window: Arc<Window>) -> Self {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let size = window.inner_size();
    let surface = instance.create_surface(window.clone()).unwrap();
    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
      })
      .await
      .unwrap();
    let (device, queue) = adapter.request_device(&Default::default()).await.unwrap();
    let mut config = surface
      .get_default_config(&adapter, size.width.max(1), size.height.max(1))
      .unwrap();
    // Grid colors are sRGB bytes, so the shader writes them to the surface unchanged.
    let plain_format = config.format.remove_srgb_suffix();
    if surface
      .get_capabilities(&adapter)
      .formats
      .contains(&plain_format)
    {
      config.format = plain_format;
    }
    surface.configure(&device, &config);
    let text = TextRenderer::new(&device, &queue, config.format, window.scale_factor() as f32);
    let (cols, rows) = text.grid_size(config.width, config.height);
    Self {
      window,
      surface,
      device,
      queue,
      config,
      text,
      grid: Grid::new(cols, rows),
    }
  }

  fn resize(&mut self, size: PhysicalSize<u32>) {
    if size.width > 0 && size.height > 0 {
      self.config.width = size.width;
      self.config.height = size.height;
      self.surface.configure(&self.device, &self.config);
      self.fit_grid();
    }
  }

  fn set_scale(&mut self, scale: f64) {
    self.text.set_scale(&self.device, &self.queue, scale as f32);
    self.fit_grid();
  }

  /// Resizes the grid to the cells that fit the surface, then schedules a redraw.
  fn fit_grid(&mut self) {
    let (cols, rows) = self.text.grid_size(self.config.width, self.config.height);
    if (cols, rows) != (self.grid.cols(), self.grid.rows()) {
      self.grid = Grid::new(cols, rows);
    }
    self.window.request_redraw();
  }

  fn render(&mut self, game: &Game) {
    let frame = match self.surface.get_current_texture() {
      wgpu::CurrentSurfaceTexture::Success(frame)
      | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
      wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => return,
      _ => {
        self.surface.configure(&self.device, &self.config);
        return;
      }
    };
    game.draw(&mut self.grid);
    self.text.prepare(&self.device, &self.queue, &self.grid);

    let view = frame.texture.create_view(&Default::default());
    let mut encoder = self.device.create_command_encoder(&Default::default());
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: &view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
          store: wgpu::StoreOp::Store,
        },
      })],
      ..Default::default()
    });
    self.text.draw(&mut pass);
    drop(pass);
    self.queue.submit([encoder.finish()]);
    self.queue.present(frame);
  }
}

#[derive(Default)]
struct App {
  gpu: Option<Gpu>,
  game: Game,
}

impl ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.gpu.is_none() {
      let attributes = Window::default_attributes()
        .with_title("rsrl")
        .with_inner_size(LogicalSize::new(640, 400));
      let window = Arc::new(event_loop.create_window(attributes).unwrap());
      self.gpu = Some(pollster::block_on(Gpu::new(window)));
    }
  }

  fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
    let Some(gpu) = self.gpu.as_mut() else { return };
    match event {
      WindowEvent::CloseRequested => event_loop.exit(),
      WindowEvent::Resized(size) => gpu.resize(size),
      WindowEvent::ScaleFactorChanged { scale_factor, .. } => gpu.set_scale(scale_factor),
      WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
        if self.game.handle_key(&event.logical_key) {
          gpu.window.request_redraw();
        }
      }
      WindowEvent::RedrawRequested => gpu.render(&self.game),
      _ => {}
    }
  }
}

fn main() {
  let event_loop = EventLoop::new().unwrap();
  event_loop.run_app(&mut App::default()).unwrap();
}
