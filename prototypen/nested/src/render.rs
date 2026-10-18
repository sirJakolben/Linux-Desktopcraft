use std::path::Path;
use std::sync::{Arc, mpsc};

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3};
use winit::event_loop::OwnedDisplayHandle;
use winit::window::Window;

use crate::camera::Camera;

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
const SKY: wgpu::Color = wgpu::Color {
    r: 0.30,
    g: 0.50,
    b: 0.90,
    a: 1.0,
};
const SCREEN_WIDTH: f32 = 2.0;

/// CPU-side pixels to upload into a quad texture.
pub struct Image<'a> {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: wgpu::TextureFormat,
    pub data: &'a [u8],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct QuadUniform {
    model: [[f32; 4]; 4],
    uv_scale: [f32; 2],
    _pad: [f32; 2],
}

#[derive(Clone, Copy)]
enum SamplerKind {
    /// Nearest + repeat, for block-like textures.
    Pixel,
    /// Linear + clamp, for window contents.
    Smooth,
}

struct Quad {
    /// Placement (rotation and translation) of the quad center.
    placement: Mat4,
    size: Vec2,
    uv_scale: Vec2,
    sampler: SamplerKind,
    texture: wgpu::Texture,
    uniform: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

pub struct Renderer {
    window: Arc<Window>,
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    view_format: wgpu::TextureFormat,
    depth: wgpu::TextureView,
    pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    quad_layout: wgpu::BindGroupLayout,
    pixel_sampler: wgpu::Sampler,
    smooth_sampler: wgpu::Sampler,
    /// Index 0 is the floor, the rest are screens.
    quads: Vec<Quad>,
    test_pattern: (u32, u32, Vec<u8>),
}

impl Renderer {
    pub async fn new(display: OwnedDisplayHandle, window: Arc<Window>) -> Self {
        let mut desc = wgpu::InstanceDescriptor::new_with_display_handle(Box::new(display));
        desc.backends = wgpu::Backends::VULKAN;
        let instance = wgpu::Instance::new(desc);
        let surface = instance
            .create_surface(window.clone())
            .expect("Surface erstellen");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("kein Vulkan-Adapter");
        let info = adapter.get_info();
        tracing::info!(
            "GPU: {} ({:?}), Treiber {} {}",
            info.name,
            info.backend,
            info.driver,
            info.driver_info
        );
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("Gerät anfordern");

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];
        let view_format = format.add_srgb_suffix();
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            view_formats: vec![view_format],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: size.width.max(1),
            height: size.height.max(1),
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        surface.configure(&device, &config);
        tracing::info!("Surface-Format {format:?}, View-Format {view_format:?}");

        let shader = device.create_shader_module(wgpu::include_wgsl!("quad.wgsl"));

        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera"),
            entries: &[uniform_entry(0)],
        });
        let quad_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("quad"),
            entries: &[
                uniform_entry(0),
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("quad"),
            bind_group_layouts: &[Some(&camera_layout), Some(&quad_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("quad"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: view_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera"),
            size: size_of::<[[f32; 4]; 4]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera"),
            layout: &camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let pixel_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("pixel"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            ..Default::default()
        });
        let smooth_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("smooth"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let depth = create_depth(&device, config.width, config.height);

        let mut renderer = Self {
            window,
            instance,
            surface,
            device,
            queue,
            config,
            view_format,
            depth,
            pipeline,
            camera_buffer,
            camera_bind_group,
            quad_layout,
            pixel_sampler,
            smooth_sampler,
            quads: Vec::new(),
            test_pattern: test_pattern(),
        };
        renderer.build_scene();
        renderer
    }

    fn build_scene(&mut self) {
        let checker = [
            90u8, 140, 70, 255, 70, 115, 55, 255, //
            70, 115, 55, 255, 90, 140, 70, 255,
        ];
        let checker = Image {
            width: 2,
            height: 2,
            stride: 8,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            data: &checker,
        };
        let floor = self.create_quad(
            Mat4::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            Vec2::splat(40.0),
            Vec2::splat(20.0),
            SamplerKind::Pixel,
            &checker,
        );
        self.quads.push(floor);

        let placements = [
            Mat4::from_translation(Vec3::new(-1.3, 1.4, -0.3))
                * Mat4::from_rotation_y(35f32.to_radians()),
            Mat4::from_translation(Vec3::new(1.3, 1.4, -0.3))
                * Mat4::from_rotation_y(-35f32.to_radians())
                * Mat4::from_rotation_x(-15f32.to_radians()),
        ];
        for placement in placements {
            let (w, h, data) = &self.test_pattern;
            let image = Image {
                width: *w,
                height: *h,
                stride: w * 4,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                data,
            };
            let size = screen_size(*w, *h);
            let quad = self.create_quad(placement, size, Vec2::ONE, SamplerKind::Smooth, &image);
            self.quads.push(quad);
        }
    }

    pub fn screen_count(&self) -> usize {
        self.quads.len() - 1
    }

    /// Shows `image` on screen `slot` (0-based), resizing the quad to its aspect ratio.
    pub fn set_screen_image(&mut self, slot: usize, image: &Image) {
        let index = slot + 1;
        let quad = &self.quads[index];
        let same = quad.texture.width() == image.width
            && quad.texture.height() == image.height
            && quad.texture.format() == image.format;
        if same {
            write_image(&self.queue, &quad.texture, image);
        } else {
            let (texture, bind_group) =
                self.create_texture(image, quad.sampler, &self.quads[index].uniform);
            let quad = &mut self.quads[index];
            quad.texture = texture;
            quad.bind_group = bind_group;
            quad.size = screen_size(image.width, image.height);
        }
    }

    /// Puts the test pattern back on screen `slot`.
    pub fn reset_screen(&mut self, slot: usize) {
        let (w, h, data) = std::mem::take(&mut self.test_pattern);
        let image = Image {
            width: w,
            height: h,
            stride: w * 4,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            data: &data,
        };
        self.set_screen_image(slot, &image);
        self.test_pattern = (w, h, data);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(&self.device, &self.config);
        self.depth = create_depth(&self.device, self.config.width, self.config.height);
    }

    pub fn render(&mut self, camera: &Camera) {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Occluded | wgpu::CurrentSurfaceTexture::Timeout => {
                return;
            }
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                drop(frame);
                self.surface.configure(&self.device, &self.config);
                return;
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                tracing::error!("Validierungsfehler der Surface");
                return;
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                self.surface = self
                    .instance
                    .create_surface(self.window.clone())
                    .expect("Surface neu erstellen");
                self.surface.configure(&self.device, &self.config);
                return;
            }
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.view_format),
            ..Default::default()
        });
        let mut encoder = self.device.create_command_encoder(&Default::default());
        self.draw(&mut encoder, &view, camera);
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        self.queue.present(frame);
    }

    /// Renders one frame offscreen and writes it as PNG.
    pub fn screenshot(&mut self, camera: &Camera, path: &Path) -> Result<(), String> {
        let (width, height) = (self.config.width, self.config.height);
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("screenshot"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.view_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let row = (width * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("screenshot"),
            size: u64::from(row) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self.device.create_command_encoder(&Default::default());
        self.draw(&mut encoder, &view, camera);
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(row),
                    rows_per_image: Some(height),
                },
            },
            texture.size(),
        );
        self.queue.submit([encoder.finish()]);

        let (tx, rx) = mpsc::channel();
        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |r| drop(tx.send(r)));
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| e.to_string())?;
        rx.recv()
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;

        let bgra = matches!(
            self.view_format,
            wgpu::TextureFormat::Bgra8UnormSrgb | wgpu::TextureFormat::Bgra8Unorm
        );
        let mapped = buffer
            .slice(..)
            .get_mapped_range()
            .map_err(|e| e.to_string())?;
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            let start = (y * row) as usize;
            for px in mapped[start..start + (width * 4) as usize]
                .as_chunks::<4>()
                .0
            {
                if bgra {
                    pixels.extend_from_slice(&[px[2], px[1], px[0], 255]);
                } else {
                    pixels.extend_from_slice(&[px[0], px[1], px[2], 255]);
                }
            }
        }
        drop(mapped);

        let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
        let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer
            .write_image_data(&pixels)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn draw(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, camera: &Camera) {
        let aspect = self.config.width as f32 / self.config.height as f32;
        let view_proj = camera.view_proj(aspect).to_cols_array_2d();
        self.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&view_proj));
        for quad in &self.quads {
            let model = quad.placement * Mat4::from_scale(quad.size.extend(1.0));
            let uniform = QuadUniform {
                model: model.to_cols_array_2d(),
                uv_scale: quad.uv_scale.to_array(),
                _pad: [0.0; 2],
            };
            self.queue
                .write_buffer(&quad.uniform, 0, bytemuck::bytes_of(&uniform));
        }

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("scene"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(SKY),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.camera_bind_group, &[]);
        for quad in &self.quads {
            pass.set_bind_group(1, &quad.bind_group, &[]);
            pass.draw(0..6, 0..1);
        }
    }

    fn create_quad(
        &self,
        placement: Mat4,
        size: Vec2,
        uv_scale: Vec2,
        sampler: SamplerKind,
        image: &Image,
    ) -> Quad {
        let uniform = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("quad"),
            size: size_of::<QuadUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let (texture, bind_group) = self.create_texture(image, sampler, &uniform);
        Quad {
            placement,
            size,
            uv_scale,
            sampler,
            texture,
            uniform,
            bind_group,
        }
    }

    fn create_texture(
        &self,
        image: &Image,
        sampler: SamplerKind,
        uniform: &wgpu::Buffer,
    ) -> (wgpu::Texture, wgpu::BindGroup) {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("quad"),
            size: wgpu::Extent3d {
                width: image.width,
                height: image.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: image.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        write_image(&self.queue, &texture, image);
        let view = texture.create_view(&Default::default());
        let sampler = match sampler {
            SamplerKind::Pixel => &self.pixel_sampler,
            SamplerKind::Smooth => &self.smooth_sampler,
        };
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("quad"),
            layout: &self.quad_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });
        (texture, bind_group)
    }
}

fn uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn write_image(queue: &wgpu::Queue, texture: &wgpu::Texture, image: &Image) {
    queue.write_texture(
        texture.as_image_copy(),
        image.data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(image.stride),
            rows_per_image: Some(image.height),
        },
        texture.size(),
    );
}

fn create_depth(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("depth"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
        .create_view(&Default::default())
}

/// World size of a screen showing a `width` x `height` image.
fn screen_size(width: u32, height: u32) -> Vec2 {
    Vec2::new(SCREEN_WIDTH, SCREEN_WIDTH * height as f32 / width as f32)
}

/// RGBA test image: color bars on top, gray ramp below, orange marker in the
/// top-left corner so flips and channel swaps are visible at a glance.
fn test_pattern() -> (u32, u32, Vec<u8>) {
    const BARS: [[u8; 3]; 8] = [
        [255, 255, 255],
        [255, 255, 0],
        [0, 255, 255],
        [0, 255, 0],
        [255, 0, 255],
        [255, 0, 0],
        [0, 0, 255],
        [0, 0, 0],
    ];
    let (w, h) = (320u32, 200u32);
    let mut data = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let rgb = if x < 32 && y < 32 {
                [255, 128, 0]
            } else if y < h * 2 / 3 {
                BARS[(x * 8 / w) as usize]
            } else {
                let v = (x * 255 / (w - 1)) as u8;
                [v, v, v]
            };
            data.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
    }
    (w, h, data)
}
