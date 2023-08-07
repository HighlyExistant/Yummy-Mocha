#![allow(unused)]
use ash::vk;

use crate::vk_obj::device::{self, ReplacingDevice};

pub struct DescriptorWriter {
    writers: Vec<vk::WriteDescriptorSet>,
}

impl DescriptorWriter {
    pub fn new() -> Self {
        Self { writers: vec![] }
    }
    pub fn add_storage_buffer(mut self, set: vk::DescriptorSet, count: u32, binding: u32, array_element: u32, info: &vk::DescriptorBufferInfo) -> Self {
        let writer = vk::WriteDescriptorSet {
            dst_set: set,
            descriptor_count: count,
            descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
            dst_binding: binding,
            dst_array_element: array_element,
            p_buffer_info: info,
            ..Default::default()
        };
        self.writers.push(writer);
        self
    }
    pub fn add_image_buffer(mut self, set: vk::DescriptorSet, count: u32, binding: u32, array_element: u32, info: &vk::DescriptorImageInfo) -> Self {
        let writer = vk::WriteDescriptorSet {
            dst_set: set,
            descriptor_count: count,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            dst_binding: binding,
            dst_array_element: array_element,
            p_image_info: info,
            ..Default::default()
        };
        self.writers.push(writer);
        self
    }
    pub fn write(&self, device: std::sync::Arc<ReplacingDevice>) {
        unsafe { device.device.update_descriptor_sets(self.writers.as_slice(), &[]) };
    }
}