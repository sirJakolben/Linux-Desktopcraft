use std::path::Path;
use std::sync::{Arc, mpsc};

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3};
use winit::event_loop::OwnedDisplayHandle;
use winit::window::Window;

use crate::camera::Camera;
use crate::compositor::{ScreenContent, SurfaceLayout, Target};
use crate::window::WindowCompositor;
use smithay::reexports::wayland_server::backend::ObjectId;
use std::collections::HashMap;

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
    /// Alpha channel is padding (XRGB-style formats).
    pub opaque: bool,
    pub data: &'a [u8],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct QuadUniform {
    model: [[f32; 4]; 4],
    uv_scale: [f32; 2],
    /// x: decode sRGB in the shader, y: flip vertically.
    flags: [f32; 2],
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
    /// Test pattern or floor texture; kept while a window is shown.
    texture: wgpu::Texture,
    flags: [f32; 2],
    uniform: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

/// A pop-up drawn as its own quad just in front of its screen.
struct PopupQuad {
    slot: usize,
    /// Position in the window's pixels.
    pos: (i32, i32),
    size: (u32, u32),
    depth: u32,
    uniform: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

/// Distance of each pop-up level in front of the wall, in blocks.
const POPUP_LIFT: f32 = 0.002;

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
    /// Window size in pixels per screen, for placing pop-ups.
    screen_px: Vec<(u32, u32)>,
    popups: HashMap<ObjectId, PopupQuad>,
    windows: WindowCompositor,
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
            .request_device(&wgpu::DeviceDescriptor {
                required_features: adapter.features()
                    & wgpu::Features::VULKAN_EXTERNAL_MEMORY_DMA_BUF,
                ..Default::default()
            })
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
        let windows = WindowCompositor::new(&device, &quad_layout, smooth_sampler.clone());

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
            screen_px: vec![(1, 1); 2],
            popups: HashMap::new(),
            windows,
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
            opaque: true,
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
                opaque: true,
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

    /// Shows the window made of `surfaces` on screen `slot` (0-based),
    /// resizing the quad to the window's aspect ratio.
    pub fn update_window(
        &mut self,
        slot: usize,
        size: (u32, u32),
        surfaces: &[SurfaceLayout],
        content: impl FnMut(&SurfaceLayout, &mut dyn FnMut(ScreenContent)),
    ) {
        let target = self.windows.update(
            &self.device,
            &self.queue,
            &Target::Screen(slot),
            size,
            surfaces,
            content,
        );
        let view = target.view.clone();
        let (width, height) = (target.texture.width(), target.texture.height());
        self.screen_px[slot] = (width, height);
        let index = slot + 1;
        let bind_group =
            self.create_bind_group(&view, SamplerKind::Smooth, &self.quads[index].uniform);
        let quad = &mut self.quads[index];
        quad.bind_group = bind_group;
        quad.size = screen_size(width, height);
    }

    /// Shows pop-up `id` of screen `slot` at `pos` (window pixels).
    #[allow(clippy::too_many_arguments)]
    pub fn update_popup(
        &mut self,
        id: ObjectId,
        slot: usize,
        pos: (i32, i32),
        depth: u32,
        size: (u32, u32),
        surfaces: &[SurfaceLayout],
        content: impl FnMut(&SurfaceLayout, &mut dyn FnMut(ScreenContent)),
    ) {
        let target = self.windows.update(
            &self.device,
            &self.queue,
            &Target::Popup(id.clone()),
            size,
            surfaces,
            content,
        );
        let view = target.view.clone();
        let size = (target.texture.width(), target.texture.height());
        let uniform = match self.popups.remove(&id) {
            Some(old) => old.uniform,
            None => self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("popup"),
                size: size_of::<QuadUniform>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        };
        let bind_group = self.create_bind_group(&view, SamplerKind::Smooth, &uniform);
        self.popups.insert(
            id,
            PopupQuad {
                slot,
                pos,
                size,
                depth,
                uniform,
                bind_group,
            },
        );
    }

    pub fn remove_popup(&mut self, id: &ObjectId) {
        self.popups.remove(id);
        self.windows.remove(&Target::Popup(id.clone()));
    }

    /// Model matrix of a pop-up: same plane and pixel scale as its screen.
    fn popup_model(&self, popup: &PopupQuad) -> Mat4 {
        let screen = &self.quads[popup.slot + 1];
        let (window_w, _) = self.screen_px[popup.slot];
        let k = screen.size.x / window_w as f32;
        let (w, h) = (popup.size.0 as f32 * k, popup.size.1 as f32 * k);
        let x = popup.pos.0 as f32 * k + w / 2.0 - screen.size.x / 2.0;
        let y = screen.size.y / 2.0 - (popup.pos.1 as f32 * k + h / 2.0);
        screen.placement
            * Mat4::from_translation(Vec3::new(x, y, POPUP_LIFT * popup.depth as f32))
            * Mat4::from_scale(Vec3::new(w, h, 1.0))
    }

    /// Puts the test pattern back on screen `slot`.
    pub fn reset_screen(&mut self, slot: usize) {
        self.windows.remove(&Target::Screen(slot));
        let index = slot + 1;
        let view = self.quads[index].texture.create_view(&Default::default());
        let bind_group =
            self.create_bind_group(&view, SamplerKind::Smooth, &self.quads[index].uniform);
        let (w, h, _) = &self.test_pattern;
        let size = screen_size(*w, *h);
        let quad = &mut self.quads[index];
        quad.bind_group = bind_group;
        quad.size = size;
    }

    /// Screen hit by the ray through `cursor` (physical window pixels):
    /// slot and UV with the origin top-left. The nearest screen wins.
    pub fn pick(&self, camera: &Camera, cursor: (f64, f64)) -> Option<(Target, Vec2)> {
        let (w, h) = (self.config.width as f32, self.config.height as f32);
        let ndc = Vec2::new(
            cursor.0 as f32 / w * 2.0 - 1.0,
            1.0 - cursor.1 as f32 / h * 2.0,
        );
        let inverse = camera.view_proj(w / h).inverse();
        let near = inverse.project_point3(ndc.extend(0.0));
        let far = inverse.project_point3(ndc.extend(1.0));
        let dir = far - near;

        // Pop-ups first: they win over their window even where they overlap.
        let mut models: Vec<(Target, Mat4, u32)> = self
            .popups
            .iter()
            .map(|(id, p)| (Target::Popup(id.clone()), self.popup_model(p), p.depth))
            .collect();
        models.sort_by_key(|(_, _, depth)| std::cmp::Reverse(*depth));
        let screens = self.quads.iter().enumerate().skip(1).map(|(index, quad)| {
            (
                Target::Screen(index - 1),
                quad.placement * Mat4::from_scale(quad.size.extend(1.0)),
                0,
            )
        });
        let popup_count = models.len();
        models.extend(screens);

        let mut best: Option<(f32, Target, Vec2)> = None;
        for (i, (target, model, _)) in models.into_iter().enumerate() {
            if i == popup_count && best.is_some() {
                break;
            }
            let to_local = model.inverse();
            let origin = to_local.transform_point3(near);
            let d = to_local.transform_vector3(dir);
            // Ray (almost) parallel to the quad: no stable hit, skip instead of
            // dividing by ~0.
            if d.z.abs() < 1e-6 {
                continue;
            }
            let t = -origin.z / d.z;
            let p = origin + d * t;
            if t < 0.0 || p.x.abs() > 0.5 || p.y.abs() > 0.5 {
                continue;
            }
            if i < popup_count && best.is_some() {
                // Deeper pop-ups were tested first.
                continue;
            }
            if best.as_ref().is_none_or(|(best_t, _, _)| t < *best_t) {
                best = Some((t, target, Vec2::new(p.x + 0.5, 0.5 - p.y)));
            }
        }
        best.map(|(_, target, uv)| (target, uv))
    }

    /// Physical window pixel where UV `uv` of screen `slot` appears.
    pub fn screen_point(&self, camera: &Camera, slot: usize, uv: Vec2) -> (f64, f64) {
        let (w, h) = (self.config.width as f32, self.config.height as f32);
        let quad = &self.quads[slot + 1];
        let model = quad.placement * Mat4::from_scale(quad.size.extend(1.0));
        let world = model.transform_point3(Vec3::new(uv.x - 0.5, 0.5 - uv.y, 0.0));
        let ndc = camera.view_proj(w / h).project_point3(world);
        (
            f64::from((ndc.x + 1.0) / 2.0 * w),
            f64::from((1.0 - ndc.y) / 2.0 * h),
        )
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(&self.device, &self.config);
        self.depth = create_depth(&self.device, self.config.width, self.config.height);
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Draws a frame. `keep_alive` is dropped once the GPU finished it, which
    /// releases client buffers only after they were read.
    pub fn render<K: Send + 'static>(&mut self, camera: &Camera, keep_alive: K) {
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
        self.submit_scene(&view, camera, |_| {});
        self.queue.on_submitted_work_done(move || drop(keep_alive));
        self.window.pre_present_notify();
        self.queue.present(frame);
    }

    /// Records and submits the scene into `view`; `after` may append to the encoder.
    fn submit_scene(
        &mut self,
        view: &wgpu::TextureView,
        camera: &Camera,
        after: impl FnOnce(&mut wgpu::CommandEncoder),
    ) {
        self.submit(|renderer, encoder| {
            renderer.draw(encoder, view, camera);
            after(encoder);
        });
    }

    /// Redraws changed windows, then runs `record`; imported textures are
    /// wrapped in ownership transfers.
    fn submit(&mut self, record: impl FnOnce(&Self, &mut wgpu::CommandEncoder)) {
        let external = self.windows.pending_external();
        let mut encoder = self.device.create_command_encoder(&Default::default());
        self.windows.draw(&mut encoder);
        record(self, &mut encoder);
        if external.is_empty() {
            self.queue.submit([encoder.finish()]);
        } else {
            let acquire = crate::dmabuf::ownership_barriers(&self.device, &external, true);
            let release = crate::dmabuf::ownership_barriers(&self.device, &external, false);
            self.queue.submit([acquire, encoder.finish(), release]);
        }
    }

    /// Writes the window texture of screen `slot` as PNG, in window pixels.
    pub fn window_screenshot(&mut self, slot: usize, path: &Path) -> Result<(), String> {
        let texture = self
            .windows
            .texture(&Target::Screen(slot))
            .ok_or_else(|| format!("Screen {slot} zeigt kein Fenster"))?;
        let (width, height) = (texture.width(), texture.height());
        let (buffer, row) = self.readback_buffer(width, height);
        self.submit(|_, encoder| copy_to_buffer(encoder, &texture, &buffer, row));
        let pixels = self.read_pixels(&buffer, width, height, row, false)?;
        write_png(path, width, height, &pixels)
    }

    fn readback_buffer(&self, width: u32, height: u32) -> (wgpu::Buffer, u32) {
        let row = (width * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: u64::from(row) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        (buffer, row)
    }

    /// Waits for the GPU and returns tightly packed RGBA pixels.
    fn read_pixels(
        &self,
        buffer: &wgpu::Buffer,
        width: u32,
        height: u32,
        row: u32,
        bgra: bool,
    ) -> Result<Vec<u8>, String> {
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
        Ok(pixels)
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
        let (buffer, row) = self.readback_buffer(width, height);
        self.submit_scene(&view, camera, |encoder| {
            copy_to_buffer(encoder, &texture, &buffer, row);
        });
        let bgra = matches!(
            self.view_format,
            wgpu::TextureFormat::Bgra8UnormSrgb | wgpu::TextureFormat::Bgra8Unorm
        );
        let pixels = self.read_pixels(&buffer, width, height, row, bgra)?;
        write_png(path, width, height, &pixels)
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
                flags: quad.flags,
            };
            self.queue
                .write_buffer(&quad.uniform, 0, bytemuck::bytes_of(&uniform));
        }
        for popup in self.popups.values() {
            let uniform = QuadUniform {
                model: self.popup_model(popup).to_cols_array_2d(),
                uv_scale: [1.0, 1.0],
                flags: [0.0, 0.0],
            };
            self.queue
                .write_buffer(&popup.uniform, 0, bytemuck::bytes_of(&uniform));
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
        for popup in self.popups.values() {
            pass.set_bind_group(1, &popup.bind_group, &[]);
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
            texture,
            flags: [0.0, 0.0],
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
        let bind_group = self.create_bind_group(&view, sampler, uniform);
        (texture, bind_group)
    }

    fn create_bind_group(
        &self,
        view: &wgpu::TextureView,
        sampler: SamplerKind,
        uniform: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        let sampler = match sampler {
            SamplerKind::Pixel => &self.pixel_sampler,
            SamplerKind::Smooth => &self.smooth_sampler,
        };
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("quad"),
            layout: &self.quad_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        })
    }
}

fn copy_to_buffer(
    encoder: &mut wgpu::CommandEncoder,
    texture: &wgpu::Texture,
    buffer: &wgpu::Buffer,
    row: u32,
) {
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(row),
                rows_per_image: Some(texture.height()),
            },
        },
        texture.size(),
    );
}

fn write_png(path: &Path, width: u32, height: u32, pixels: &[u8]) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(pixels).map_err(|e| e.to_string())
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

pub fn write_image(queue: &wgpu::Queue, texture: &wgpu::Texture, image: &Image) {
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
