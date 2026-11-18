use core::Psp;
use core::display::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

pub struct Emulator {
    psp: Psp,
    frontend: Option<Frontend>,
}

impl Emulator {
    pub fn new(executable: &[u8]) -> Self {
        Self {
            psp: Psp::new(executable),
            frontend: None,
        }
    }
}

impl ApplicationHandler for Emulator {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.frontend.is_some() {
            return;
        }

        let attributes = Window::default_attributes()
            .with_title("peespee")
            .with_inner_size(LogicalSize::new(DISPLAY_WIDTH * 2, DISPLAY_HEIGHT * 2));
        let window = event_loop
            .create_window(attributes)
            .expect("Failed to create window");
        self.frontend = Some(Frontend::new(Arc::new(window)));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(frontend) = &mut self.frontend else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => frontend.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                self.psp.run_frame(&mut frontend.framebuffer);
                frontend.render();
                frontend.window.request_redraw();
            }
            _ => {}
        }
    }
}

struct Frontend {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    framebuffer: Vec<u8>,
    framebuffer_texture: wgpu::Texture,
    framebuffer_texture_view: wgpu::TextureView,
    framebuffer_blitter: wgpu::util::TextureBlitter,
}

impl Frontend {
    fn new(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .expect("Failed to find a GPU adapter");
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("Failed to create GPU device");

        let size = window.inner_size();
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .expect("Surface is not supported by the adapter");
        config.present_mode = wgpu::PresentMode::AutoNoVsync;
        surface.configure(&device, &config);

        let framebuffer_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: DISPLAY_WIDTH,
                height: DISPLAY_HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let framebuffer_texture_view =
            framebuffer_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let framebuffer_blitter = wgpu::util::TextureBlitter::new(&device, config.format);

        Self {
            window,
            surface,
            device,
            queue,
            config,

            framebuffer: vec![0; (DISPLAY_WIDTH * DISPLAY_HEIGHT * 4) as usize],
            framebuffer_texture,
            framebuffer_texture_view,
            framebuffer_blitter,
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    fn render(&mut self) {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            _ => return,
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.queue.write_texture(
            self.framebuffer_texture.as_image_copy(),
            &self.framebuffer,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(DISPLAY_WIDTH * 4),
                rows_per_image: Some(DISPLAY_HEIGHT),
            },
            self.framebuffer_texture.size(),
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        self.framebuffer_blitter.copy(
            &self.device,
            &mut encoder,
            &self.framebuffer_texture_view,
            &view,
        );

        self.queue.submit([encoder.finish()]);
        self.queue.present(frame);
    }
}
