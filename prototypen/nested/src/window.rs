//! One offscreen texture per window: the whole surface tree (toplevel and
//! subsurfaces) composited in 2D, sampled by the screen quad in the scene.

use std::collections::HashMap;

use bytemuck::{Pod, Zeroable};
use smithay::backend::renderer::utils::CommitCounter;
use smithay::reexports::wayland_server::backend::ObjectId;

use crate::compositor::{ScreenContent, SurfaceLayout, Target};
use crate::render::{Image, write_image};

/// Linear blending, sRGB storage; the scene samples it like any sRGB texture.
pub const WINDOW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ItemUniform {
    /// Destination in NDC: left, top, right, bottom.
    rect: [f32; 4],
    /// Source in UV: left, top, right, bottom.
    uv: [f32; 4],
    /// x: decode sRGB, y: flip vertically, z: force opaque.
    flags: [f32; 4],
}

struct SurfaceTexture {
    texture: wgpu::Texture,
    /// Imported dmabuf (needs ownership barriers) rather than an owned copy.
    external: bool,
    commit: Option<CommitCounter>,
    uniform: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

pub struct WindowTarget {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    /// Surfaces in drawing order (bottom first).
    draws: Vec<ObjectId>,
    dirty: bool,
}

pub struct WindowCompositor {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    surfaces: HashMap<ObjectId, SurfaceTexture>,
    windows: HashMap<Target, WindowTarget>,
}

impl WindowCompositor {
    pub fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        sampler: wgpu::Sampler,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("window.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("window"),
            bind_group_layouts: &[Some(layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("window"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: WINDOW_FORMAT,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        Self {
            pipeline,
            layout: layout.clone(),
            sampler,
            surfaces: HashMap::new(),
            windows: HashMap::new(),
        }
    }

    /// Rebuilds `target` from `surfaces`; `content` supplies each
    /// surface's buffer. Returns the window texture.
    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &Target,
        size: (u32, u32),
        surfaces: &[SurfaceLayout],
        mut content: impl FnMut(&SurfaceLayout, &mut dyn FnMut(ScreenContent)),
    ) -> &WindowTarget {
        let (width, height) = (size.0.max(1), size.1.max(1));
        let resize = self
            .windows
            .get(target)
            .is_none_or(|w| w.texture.width() != width || w.texture.height() != height);
        if resize {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("window"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: WINDOW_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let view = texture.create_view(&Default::default());
            let old = self.windows.remove(target);
            self.windows.insert(
                target.clone(),
                WindowTarget {
                    texture,
                    view,
                    draws: old.map(|w| w.draws).unwrap_or_default(),
                    dirty: true,
                },
            );
        }

        let mut draws = Vec::with_capacity(surfaces.len());
        for layout in surfaces {
            let mut shown = false;
            content(layout, &mut |c| {
                shown = self.update_surface(device, queue, layout, (width, height), c);
            });
            if shown {
                draws.push(layout.id.clone());
            }
        }

        let window = self.windows.get_mut(target).expect("created above");
        for id in &window.draws {
            if !draws.contains(id) {
                self.surfaces.remove(id);
            }
        }
        window.draws = draws;
        window.dirty = true;
        window
    }

    /// Updates the texture and placement of one surface. Returns whether it is drawn.
    fn update_surface(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &SurfaceLayout,
        window_size: (u32, u32),
        content: ScreenContent,
    ) -> bool {
        let (flags, existing_ok) = match &content {
            ScreenContent::Empty => {
                self.surfaces.remove(&layout.id);
                return false;
            }
            ScreenContent::Shm(image) => {
                let ok = self.surfaces.get(&layout.id).is_some_and(|s| {
                    !s.external
                        && s.texture.width() == image.width
                        && s.texture.height() == image.height
                        && s.texture.format() == image.format
                });
                ([0.0, 0.0, opaque_flag(image.opaque), 0.0], ok)
            }
            ScreenContent::Gpu {
                texture,
                y_inverted,
                opaque,
            } => {
                let ok = self
                    .surfaces
                    .get(&layout.id)
                    .is_some_and(|s| s.external && &s.texture == texture);
                let flip = if *y_inverted { 1.0 } else { 0.0 };
                ([1.0, flip, opaque_flag(*opaque), 0.0], ok)
            }
        };

        if existing_ok {
            let surface = self.surfaces.get_mut(&layout.id).expect("checked");
            if let ScreenContent::Shm(image) = &content
                && surface.commit != Some(layout.commit)
            {
                write_image(queue, &surface.texture, image);
            }
            surface.commit = Some(layout.commit);
        } else {
            let (texture, external) = match &content {
                ScreenContent::Shm(image) => (create_owned(device, queue, image), false),
                ScreenContent::Gpu { texture, .. } => (texture.clone(), true),
                ScreenContent::Empty => unreachable!(),
            };
            let uniform = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("window item"),
                size: size_of::<ItemUniform>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let view = texture.create_view(&Default::default());
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("window item"),
                layout: &self.layout,
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
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });
            self.surfaces.insert(
                layout.id.clone(),
                SurfaceTexture {
                    texture,
                    external,
                    commit: Some(layout.commit),
                    uniform,
                    bind_group,
                },
            );
        }

        let (w, h) = (window_size.0 as f32, window_size.1 as f32);
        let [x, y, dw, dh] = layout.dst;
        let uniform = ItemUniform {
            rect: [
                x / w * 2.0 - 1.0,
                1.0 - y / h * 2.0,
                (x + dw) / w * 2.0 - 1.0,
                1.0 - (y + dh) / h * 2.0,
            ],
            uv: layout.uv,
            flags,
        };
        let surface = &self.surfaces[&layout.id];
        queue.write_buffer(&surface.uniform, 0, bytemuck::bytes_of(&uniform));
        true
    }

    pub fn texture(&self, target: &Target) -> Option<wgpu::Texture> {
        self.windows.get(target).map(|w| w.texture.clone())
    }

    pub fn remove(&mut self, target: &Target) {
        if let Some(window) = self.windows.remove(target) {
            for id in window.draws {
                self.surfaces.remove(&id);
            }
        }
    }

    /// Imported textures read by the next `draw`; they need ownership barriers.
    pub fn pending_external(&self) -> Vec<wgpu::Texture> {
        self.windows
            .values()
            .filter(|w| w.dirty)
            .flat_map(|w| &w.draws)
            .filter_map(|id| self.surfaces.get(id))
            .filter(|s| s.external)
            .map(|s| s.texture.clone())
            .collect()
    }

    /// Redraws every window whose contents changed.
    pub fn draw(&mut self, encoder: &mut wgpu::CommandEncoder) {
        for window in self.windows.values_mut().filter(|w| w.dirty) {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("window"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &window.view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            for id in &window.draws {
                if let Some(surface) = self.surfaces.get(id) {
                    pass.set_bind_group(0, &surface.bind_group, &[]);
                    pass.draw(0..6, 0..1);
                }
            }
            drop(pass);
            window.dirty = false;
        }
    }
}

fn opaque_flag(opaque: bool) -> f32 {
    if opaque { 1.0 } else { 0.0 }
}

fn create_owned(device: &wgpu::Device, queue: &wgpu::Queue, image: &Image) -> wgpu::Texture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("shm surface"),
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
    write_image(queue, &texture, image);
    texture
}
