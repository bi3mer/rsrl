//! Opens a window and clears it to red with wgpu.

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

const RED: wgpu::Color = wgpu::Color {
  r: 1.0,
  g: 0.0,
  b: 0.0,
  a: 1.0,
};

struct Gpu {
  window: Arc<Window>,
  surface: wgpu::Surface<'static>,
  device: wgpu::Device,
  queue: wgpu::Queue,
  config: wgpu::SurfaceConfiguration,
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
    let config = surface
      .get_default_config(&adapter, size.width.max(1), size.height.max(1))
      .unwrap();
    surface.configure(&device, &config);
    Self {
      window,
      surface,
      device,
      queue,
      config,
    }
  }

  fn resize(&mut self, size: PhysicalSize<u32>) {
    if size.width > 0 && size.height > 0 {
      self.config.width = size.width;
      self.config.height = size.height;
      self.surface.configure(&self.device, &self.config);
      self.window.request_redraw();
    }
  }

  fn render(&mut self) {
    let frame = match self.surface.get_current_texture() {
      wgpu::CurrentSurfaceTexture::Success(frame)
      | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
      wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => return,
      _ => {
        self.surface.configure(&self.device, &self.config);
        return;
      }
    };
    let view = frame.texture.create_view(&Default::default());
    let mut encoder = self.device.create_command_encoder(&Default::default());
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: &view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Clear(RED),
          store: wgpu::StoreOp::Store,
        },
      })],
      ..Default::default()
    });
    self.queue.submit([encoder.finish()]);
    self.queue.present(frame);
  }
}

#[derive(Default)]
struct App {
  gpu: Option<Gpu>,
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
      WindowEvent::RedrawRequested => gpu.render(),
      _ => {}
    }
  }
}

fn main() {
  let event_loop = EventLoop::new().unwrap();
  event_loop.run_app(&mut App::default()).unwrap();
}
