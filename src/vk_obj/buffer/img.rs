#![allow(unused)]
use ash::vk;
use crate::vk_obj::device::{self, ReplacingDevice, queues::DeviceQueueCategory};
use image;
use super::raw::Buffer;
#[derive(Default)]
pub struct ImageTexture {
    image: vk::Image,
    view: vk::ImageView,
    sampler: vk::Sampler,
    memory: vk::DeviceMemory,
}
impl ImageTexture {
    pub fn new(device: std::sync::Arc<ReplacingDevice>, filepath: &str) -> Self {
        let image = image::open(filepath).unwrap();
        let rgba8 = image.into_rgba8();
        let vector = rgba8.clone().into_vec();
        let size =( rgba8.dimensions().0 * rgba8.dimensions().1 * 4) as usize;

        let mut temp = Buffer::new(
            device.clone(), size, 
            vk::BufferUsageFlags::TRANSFER_SRC, 
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
        );
        temp.mapping(device.clone(), size, 0);
        temp.append(&vector);
        temp.unmapping(device.clone());

        // TODO lazy with all the formats so ill give up for now.
        let format = vk::Format::R8G8B8A8_SRGB;
        let info = vk::ImageCreateInfo {
            image_type: vk::ImageType::TYPE_2D,
            extent: vk::Extent3D {
                width: rgba8.width(),
                height: rgba8.height(),
                depth: 1
            },
            mip_levels: 1,
            format: format,
            tiling: vk::ImageTiling::OPTIMAL,
            initial_layout: vk::ImageLayout::UNDEFINED,
            usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            array_layers: 1,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };
        let (image, memory) = device.create_image(&info);
        Self::transition(device.clone(), &image, format, vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
        temp.to_image(device.clone(), &image, rgba8.width(), rgba8.height());
        Self::transition(device.clone(), &image, format, vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        
        let view_info = vk::ImageViewCreateInfo {
            image: image,
            view_type: vk::ImageViewType::TYPE_2D,
            format: format,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..Default::default()
        };
        let view = unsafe { device.device.create_image_view(&view_info, None).unwrap() };
        let sampler = Self::create_texture_sampler(device.clone());
        Self { image, view, sampler, memory }
    }
    fn transition(device: std::sync::Arc<ReplacingDevice>, image: &vk::Image, format: vk::Format, old_layout: vk::ImageLayout, new_layout: vk::ImageLayout) {
        let cmd_buffer = device.single_time_commands(DeviceQueueCategory::Graphics);

        

        let (src_access_mask, dst_access_mask, source, destination) 
        = if old_layout == vk::ImageLayout::UNDEFINED && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL {
            
            (vk::AccessFlags::NONE, vk::AccessFlags::TRANSFER_WRITE, vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER)

        } else if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL {
            
            (vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::SHADER_READ, vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::FRAGMENT_SHADER)

        } else {
            panic!("this layout transition is not supported");
        };
        
        let barrier = vk::ImageMemoryBarrier {
            old_layout,
            new_layout,
            src_queue_family_index: std::u32::MAX, // VK_QUEUE_FAMILY_IGNORED
            dst_queue_family_index: std::u32::MAX, // VK_QUEUE_FAMILY_IGNORED
            image: *image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            src_access_mask,
            dst_access_mask,
            ..Default::default()
        };
        unsafe { device.device.cmd_pipeline_barrier(
                    cmd_buffer, 
                    source, 
                    destination, 
                    vk::DependencyFlags::empty(), 
                    &[], 
                    &[], 
                    &[barrier]) };
        device.end_single_time_commands(cmd_buffer, DeviceQueueCategory::Graphics);
    }
    fn create_texture_sampler(device: std::sync::Arc<ReplacingDevice>) -> vk::Sampler {
        // let properties = unsafe { device.instance.instance.get_physical_device_properties(self.physical_device) };

        let info = vk::SamplerCreateInfo {
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            address_mode_u: vk::SamplerAddressMode::REPEAT,
            address_mode_v: vk::SamplerAddressMode::REPEAT,
            address_mode_w: vk::SamplerAddressMode::REPEAT,
            anisotropy_enable: 0, // VK_FALSE
            // max_anisotropy: properties.limits.max_sampler_anisotropy,
            max_anisotropy: 1.0,
            border_color: vk::BorderColor::INT_OPAQUE_BLACK,
            unnormalized_coordinates: 0, // VK_FALSE,
            compare_enable: 0, // VK_FALSE
            compare_op: vk::CompareOp::ALWAYS,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            mip_lod_bias: 0.0,
            min_lod: 0.0,
            max_lod: 0.0,
            
            ..Default::default()
        };
        
        unsafe { device.device.create_sampler(&info, None).unwrap() }
    }
    pub fn get_info(&self, layout: vk::ImageLayout) -> vk::DescriptorImageInfo {
        vk::DescriptorImageInfo {
            image_layout: layout,
            image_view: self.view,
            sampler: self.sampler
        }
    }
}