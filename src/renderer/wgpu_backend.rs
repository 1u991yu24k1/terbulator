use crate::renderer::backend::{BackendType, Color, CursorInfo, RenderBackend};
use crate::terminal::Grid;
use crate::utils::{Result, TerbulatorError};
use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, SwashCache};
use std::sync::Arc;
use winit::window::Window;

pub struct WgpuBackend {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    font_system: FontSystem,
    swash_cache: SwashCache,
    font_size: f32,
    cell_width: f32,
    cell_height: f32,
    width: u32,
    height: u32,
}

impl WgpuBackend {
    async fn new_async(window: Arc<Window>, font_size: f32) -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| TerbulatorError::rendering(format!("Failed to create surface: {}", e)))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| TerbulatorError::rendering("Failed to find suitable adapter"))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Terminal Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .map_err(|e| TerbulatorError::rendering(format!("Failed to create device: {}", e)))?;

        let size = window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();

        // Measure cell dimensions
        let metrics = Metrics::new(font_size, font_size * 1.2);
        let mut buffer = Buffer::new(&mut font_system, metrics);
        buffer.set_text(&mut font_system, "M", Attrs::new(), cosmic_text::Shaping::Advanced);
        buffer.shape_until_scroll(&mut font_system, false);

        let cell_width = font_size * 0.6; // Approximate monospace width
        let cell_height = font_size * 1.2; // Line height

        Ok(Self {
            device,
            queue,
            surface,
            config,
            font_system,
            swash_cache,
            font_size,
            cell_width,
            cell_height,
            width: size.width,
            height: size.height,
        })
    }

    fn render_to_buffer(&mut self, grid: &Grid, cursor: CursorInfo) -> Vec<u8> {
        let width = self.width as usize;
        let height = self.height as usize;
        let mut buffer = vec![0u8; width * height * 4];

        // Fill background
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 4;
                buffer[idx] = 0;
                buffer[idx + 1] = 0;
                buffer[idx + 2] = 0;
                buffer[idx + 3] = 255;
            }
        }

        // Render cells
        for row in 0..grid.rows() {
            for col in 0..grid.cols() {
                if let Some(cell) = grid.get(col, row) {
                    let x = (col as f32 * self.cell_width) as usize;
                    let y = (row as f32 * self.cell_height) as usize;

                    // Draw background
                    let bg = if cell.attrs.inverse {
                        cell.fg
                    } else {
                        cell.bg
                    };

                    for dy in 0..(self.cell_height as usize) {
                        for dx in 0..(self.cell_width as usize) {
                            let px = x + dx;
                            let py = y + dy;
                            if px < width && py < height {
                                let idx = (py * width + px) * 4;
                                buffer[idx] = bg.r;
                                buffer[idx + 1] = bg.g;
                                buffer[idx + 2] = bg.b;
                                buffer[idx + 3] = bg.a;
                            }
                        }
                    }

                    // Render character (simplified - actual rendering would use swash)
                    if cell.ch != ' ' {
                        let fg = if cell.attrs.inverse {
                            cell.bg
                        } else {
                            cell.fg
                        };

                        // Simple glyph rendering placeholder
                        // In a real implementation, we'd use swash_cache to render glyphs
                        self.render_glyph_simple(&mut buffer, x, y, width, fg);
                    }
                }
            }
        }

        // Render cursor
        if cursor.visible {
            let x = (cursor.col as f32 * self.cell_width) as usize;
            let y = (cursor.row as f32 * self.cell_height) as usize;

            for dy in 0..(self.cell_height as usize) {
                for dx in 0..(self.cell_width as usize) {
                    let px = x + dx;
                    let py = y + dy;
                    if px < width && py < height {
                        let idx = (py * width + px) * 4;
                        buffer[idx] = 255;
                        buffer[idx + 1] = 255;
                        buffer[idx + 2] = 255;
                        buffer[idx + 3] = 128;
                    }
                }
            }
        }

        buffer
    }

    fn render_glyph_simple(&self, buffer: &mut [u8], x: usize, y: usize, width: usize, color: Color) {
        // Simplified glyph rendering - just draw a small rectangle
        let gw = (self.cell_width * 0.8) as usize;
        let gh = (self.cell_height * 0.8) as usize;

        for dy in 0..gh {
            for dx in 0..gw {
                let px = x + dx + 1;
                let py = y + dy + 1;
                if px < width && py < buffer.len() / width / 4 {
                    let idx = (py * width + px) * 4;
                    if idx + 3 < buffer.len() {
                        buffer[idx] = color.r;
                        buffer[idx + 1] = color.g;
                        buffer[idx + 2] = color.b;
                        buffer[idx + 3] = color.a;
                    }
                }
            }
        }
    }
}

impl RenderBackend for WgpuBackend {
    fn new(_window: &Window, _font_size: f32) -> Result<Self> {
        // GPU backend is not yet fully implemented
        // For now, we return an error and fall back to CPU backend
        Err(TerbulatorError::backend_init(
            "WgpuBackend not yet fully implemented, use CPU backend instead"
        ))
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        if width > 0 && height > 0 {
            self.width = width;
            self.height = height;
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
        Ok(())
    }

    fn clear(&mut self) -> Result<()> {
        // GPU backend not yet implemented
        Ok(())
    }

    fn render_frame(&mut self, grid: &mut Grid, cursor: CursorInfo) -> Result<()> {
        // Clear dirty tracking (wgpu backend not yet fully implemented)
        grid.clear_dirty();

        let frame = self
            .surface
            .get_current_texture()
            .map_err(|e| TerbulatorError::rendering(format!("Failed to get surface texture: {}", e)))?;

        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let buffer_data = self.render_to_buffer(grid, cursor);

        // Create texture from buffer
        let texture_size = wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Frame Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &buffer_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.width),
                rows_per_image: Some(self.height),
            },
            texture_size,
        );

        // For now, just clear the screen
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }

    fn render_pane(
        &mut self,
        grid: &mut Grid,
        _cursor: CursorInfo,
        _offset_x: i32,
        _offset_y: i32,
        _width: u32,
        _height: u32,
    ) -> Result<()> {
        // Clear dirty tracking (wgpu backend not yet fully implemented)
        grid.clear_dirty();
        // GPU backend not yet implemented
        Err(TerbulatorError::rendering("WgpuBackend multi-pane rendering not implemented"))
    }

    fn draw_border(&mut self, _x: i32, _y: i32, _width: i32, _height: i32) -> Result<()> {
        // GPU backend not yet implemented
        Ok(())
    }

    fn present(&mut self) -> Result<()> {
        // Present is handled by the surface in wgpu
        Ok(())
    }

    fn backend_type(&self) -> BackendType {
        BackendType::Gpu
    }

    fn cell_dimensions(&self) -> (f32, f32) {
        (self.cell_width, self.cell_height)
    }

    fn render_help_overlay(&mut self, _help_text: &[&str]) -> Result<()> {
        // GPU backend help overlay not yet implemented
        // For now, return Ok to allow compilation
        log::warn!("Help overlay not implemented for GPU backend");
        Ok(())
    }

    fn font_size(&self) -> f32 {
        self.font_size
    }

    fn set_font_size(&mut self, size: f32) -> Result<()> {
        self.font_size = size;
        self.cell_width = size * 0.6;
        self.cell_height = size * 1.3;
        log::info!("Font size changed to {} (GPU backend)", size);
        Ok(())
    }

    fn draw_selection_highlight(&mut self, _col: usize, _row: usize, _cell_width: f32, _cell_height: f32, _offset_x: i32, _offset_y: i32) -> Result<()> {
        // GPU backend not yet implemented
        Ok(())
    }

    fn draw_image(&mut self, _image: &image::DynamicImage, _x: i32, _y: i32, _width: u32, _height: u32) -> Result<()> {
        // GPU backend not yet implemented
        Ok(())
    }
}
