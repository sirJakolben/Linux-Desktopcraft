//! Zero-copy import of client dmabufs into wgpu textures (Vulkan only).
//!
//! All `unsafe` for dmabuf handling lives in this module.

use ash::vk;
use smithay::backend::allocator::dmabuf::{Dmabuf, WeakDmabuf};
use smithay::backend::allocator::{Buffer, Format, Fourcc, Modifier};
use wgpu::hal;
use wgpu::hal::api::Vulkan;

/// DRM formats we offer, all single-plane 8-bit RGB.
const FOURCCS: [Fourcc; 4] = [
    Fourcc::Argb8888,
    Fourcc::Xrgb8888,
    Fourcc::Abgr8888,
    Fourcc::Xbgr8888,
];

/// DRM fourcc → Vulkan/wgpu format with the same memory layout.
/// UNORM on purpose: the shader decodes sRGB, so no sRGB modifier support is needed.
fn formats_for(code: Fourcc) -> Option<(vk::Format, wgpu::TextureFormat)> {
    match code {
        Fourcc::Argb8888 | Fourcc::Xrgb8888 => {
            Some((vk::Format::B8G8R8A8_UNORM, wgpu::TextureFormat::Bgra8Unorm))
        }
        Fourcc::Abgr8888 | Fourcc::Xbgr8888 => {
            Some((vk::Format::R8G8B8A8_UNORM, wgpu::TextureFormat::Rgba8Unorm))
        }
        _ => None,
    }
}

pub struct DmabufImporter {
    device: wgpu::Device,
    /// `dev_t` of the render node of the GPU wgpu runs on.
    pub render_node: libc::dev_t,
    /// Format + modifier pairs that can be imported and sampled.
    pub formats: Vec<Format>,
    cache: Vec<(WeakDmabuf, wgpu::Texture)>,
}

impl DmabufImporter {
    pub fn new(device: &wgpu::Device) -> Result<Self, String> {
        if !device
            .features()
            .contains(wgpu::Features::VULKAN_EXTERNAL_MEMORY_DMA_BUF)
        {
            return Err("VULKAN_EXTERNAL_MEMORY_DMA_BUF fehlt".into());
        }
        // SAFETY: the hal device is only used for queries; no handle is destroyed.
        let hal_device =
            unsafe { device.as_hal::<Vulkan>() }.ok_or("wgpu läuft nicht auf Vulkan")?;
        let instance = hal_device.shared_instance().raw_instance();
        let physical = hal_device.raw_physical_device();

        let mut drm = vk::PhysicalDeviceDrmPropertiesEXT::default();
        let mut props = vk::PhysicalDeviceProperties2::default().push_next(&mut drm);
        // SAFETY: valid instance and physical device; output structs are default-initialized.
        unsafe { instance.get_physical_device_properties2(physical, &mut props) };
        if drm.has_render == vk::FALSE {
            return Err("GPU meldet keinen Render-Node".into());
        }
        let render_node = libc::makedev(drm.render_major as u32, drm.render_minor as u32);

        let mut formats = Vec::new();
        for code in FOURCCS {
            let (vk_format, _) = formats_for(code).expect("listed fourcc");
            for modifier in importable_modifiers(instance, physical, vk_format) {
                formats.push(Format {
                    code,
                    modifier: Modifier::from(modifier),
                });
            }
        }
        drop(hal_device);

        Ok(Self {
            device: device.clone(),
            render_node,
            formats,
            cache: Vec::new(),
        })
    }

    /// Returns the texture for `dmabuf`, importing it on first use.
    pub fn import(&mut self, dmabuf: &Dmabuf) -> Result<wgpu::Texture, String> {
        self.cache.retain(|(weak, _)| !weak.is_gone());
        if let Some((_, texture)) = self
            .cache
            .iter()
            .find(|(weak, _)| weak.upgrade().as_ref() == Some(dmabuf))
        {
            return Ok(texture.clone());
        }

        let format = dmabuf.format();
        if dmabuf.num_planes() != 1 {
            return Err(format!("{} Ebenen, nur 1 unterstützt", dmabuf.num_planes()));
        }
        if !self.formats.contains(&format) {
            return Err(format!(
                "Format {:?} mit Modifier {:?} nicht unterstützt",
                format.code, format.modifier
            ));
        }
        let (_, texture_format) = formats_for(format.code).ok_or("Format ohne Abbildung")?;
        let fd = dmabuf
            .handles()
            .next()
            .ok_or("dmabuf ohne fd")?
            .try_clone_to_owned()
            .map_err(|e| e.to_string())?;
        let offset = dmabuf.offsets().next().unwrap_or(0);
        let stride = dmabuf.strides().next().unwrap_or(0);
        let size = wgpu::Extent3d {
            width: dmabuf.width(),
            height: dmabuf.height(),
            depth_or_array_layers: 1,
        };

        let hal_desc = hal::TextureDescriptor {
            label: Some("dmabuf"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: texture_format,
            usage: wgpu::TextureUses::RESOURCE,
            memory_flags: hal::MemoryFlags::empty(),
            view_formats: Vec::new(),
        };
        let hal_texture = {
            // SAFETY: only used to create the image below.
            let hal_device =
                unsafe { self.device.as_hal::<Vulkan>() }.ok_or("wgpu läuft nicht auf Vulkan")?;
            // SAFETY: `fd` is our own duplicate of a single-plane dmabuf whose
            // format/modifier is in the list the driver reported as importable;
            // modifier, stride and offset come from the client's buffer
            // parameters. Vulkan takes ownership of `fd`.
            unsafe {
                hal_device.texture_from_dmabuf_fd(
                    fd,
                    &hal_desc,
                    u64::from(format.modifier),
                    u64::from(stride),
                    u64::from(offset),
                )
            }
            .map_err(|e| format!("Vulkan-Import fehlgeschlagen: {e}"))?
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("dmabuf"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: texture_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        // SAFETY: the image was created on this device matching `desc`. Its
        // contents are defined by the client. wgpu tracks it as RESOURCE; our
        // own barriers (`ownership_barriers`) move it between the external
        // owner and RESOURCE around every use, so the tracked state holds.
        let texture = unsafe {
            self.device.create_texture_from_hal::<Vulkan>(
                hal_texture,
                &desc,
                wgpu::TextureUses::RESOURCE,
                true,
            )
        };
        tracing::info!(
            "dmabuf importiert ohne Kopie: {:?}, Modifier {:#x}, {}×{}",
            format.code,
            u64::from(format.modifier),
            size.width,
            size.height
        );
        self.cache.push((dmabuf.weak(), texture.clone()));
        Ok(texture)
    }
}

/// Command buffer that moves `textures` from the external owner (the
/// client's driver) to wgpu's queue family (`acquire`) or back.
///
/// Recorded with raw Vulkan: wgpu-hal's barrier API needs the hal textures
/// borrowed while the encoder is open, which wgpu's snatch lock forbids.
/// `VK_QUEUE_FAMILY_EXTERNAL` instead of `FOREIGN`: wgpu does not enable
/// `VK_EXT_queue_family_foreign`, and clients render on the same GPU.
pub fn ownership_barriers(
    device: &wgpu::Device,
    textures: &[wgpu::Texture],
    acquire: bool,
) -> wgpu::CommandBuffer {
    let images: Vec<vk::Image> = textures
        .iter()
        // SAFETY: the guard only lives for reading the handle.
        .filter_map(|t| unsafe { t.as_hal::<Vulkan>() }.map(|h| unsafe { h.raw_handle() }))
        .collect();
    let (raw_device, ours) = {
        // SAFETY: only clones the dispatch table and reads the family index.
        let hal_device = unsafe { device.as_hal::<Vulkan>() }.expect("Vulkan");
        (
            hal_device.raw_device().clone(),
            hal_device.queue_family_index(),
        )
    };
    let range = vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
    };
    // External images are kept in GENERAL layout while foreign-owned.
    let barriers: Vec<vk::ImageMemoryBarrier> = images
        .iter()
        .map(|&image| {
            let barrier = vk::ImageMemoryBarrier::default()
                .image(image)
                .subresource_range(range);
            if acquire {
                barrier
                    .dst_access_mask(vk::AccessFlags::SHADER_READ)
                    .old_layout(vk::ImageLayout::GENERAL)
                    .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .src_queue_family_index(vk::QUEUE_FAMILY_EXTERNAL)
                    .dst_queue_family_index(ours)
            } else {
                barrier
                    .src_access_mask(vk::AccessFlags::SHADER_READ)
                    .old_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .new_layout(vk::ImageLayout::GENERAL)
                    .src_queue_family_index(ours)
                    .dst_queue_family_index(vk::QUEUE_FAMILY_EXTERNAL)
            }
        })
        .collect();
    let (src_stage, dst_stage) = if acquire {
        (
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
        )
    } else {
        (
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
        )
    };
    let mut encoder = device.create_command_encoder(&Default::default());
    // SAFETY: the encoder is used only through the raw command buffer; the
    // images are imported dmabufs alive for this submission (held by the quads).
    unsafe {
        encoder.as_hal_mut::<Vulkan, _, _>(|hal_encoder| {
            if let Some(hal_encoder) = hal_encoder {
                raw_device.cmd_pipeline_barrier(
                    hal_encoder.raw_handle(),
                    src_stage,
                    dst_stage,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &barriers,
                );
            }
        });
    }
    encoder.finish()
}

/// Single-plane modifiers of `format` that can be imported from a dmabuf and sampled.
fn importable_modifiers(
    instance: &ash::Instance,
    physical: vk::PhysicalDevice,
    format: vk::Format,
) -> Vec<u64> {
    let mut list = vk::DrmFormatModifierPropertiesListEXT::default();
    let mut props = vk::FormatProperties2::default().push_next(&mut list);
    // SAFETY: first call only queries the count.
    unsafe { instance.get_physical_device_format_properties2(physical, format, &mut props) };
    let mut modifiers = vec![
        vk::DrmFormatModifierPropertiesEXT::default();
        list.drm_format_modifier_count as usize
    ];
    let mut list = vk::DrmFormatModifierPropertiesListEXT::default()
        .drm_format_modifier_properties(&mut modifiers);
    let mut props = vk::FormatProperties2::default().push_next(&mut list);
    // SAFETY: `modifiers` has room for the count reported above.
    unsafe { instance.get_physical_device_format_properties2(physical, format, &mut props) };

    modifiers
        .iter()
        .filter(|m| {
            m.drm_format_modifier_plane_count == 1
                && m.drm_format_modifier_tiling_features
                    .contains(vk::FormatFeatureFlags::SAMPLED_IMAGE)
                && can_import(instance, physical, format, m.drm_format_modifier)
        })
        .map(|m| m.drm_format_modifier)
        .collect()
}

fn can_import(
    instance: &ash::Instance,
    physical: vk::PhysicalDevice,
    format: vk::Format,
    modifier: u64,
) -> bool {
    let mut external_info = vk::PhysicalDeviceExternalImageFormatInfo::default()
        .handle_type(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);
    let mut modifier_info = vk::PhysicalDeviceImageDrmFormatModifierInfoEXT::default()
        .drm_format_modifier(modifier)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    let info = vk::PhysicalDeviceImageFormatInfo2::default()
        .format(format)
        .ty(vk::ImageType::TYPE_2D)
        .tiling(vk::ImageTiling::DRM_FORMAT_MODIFIER_EXT)
        .usage(vk::ImageUsageFlags::SAMPLED)
        .push_next(&mut external_info)
        .push_next(&mut modifier_info);
    let mut external_props = vk::ExternalImageFormatProperties::default();
    let mut props = vk::ImageFormatProperties2::default().push_next(&mut external_props);
    // SAFETY: all structs are valid and correctly chained.
    let result = unsafe {
        instance.get_physical_device_image_format_properties2(physical, &info, &mut props)
    };
    result.is_ok()
        && external_props
            .external_memory_properties
            .external_memory_features
            .contains(vk::ExternalMemoryFeatureFlags::IMPORTABLE)
}
