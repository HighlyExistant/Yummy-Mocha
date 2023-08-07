#![allow(unused)]
use ash::vk;
use std::sync::Arc;
use crate::vk_obj::device::{self, ReplacingDevice};
pub struct DescriptorLayout {
    device: Arc<ReplacingDevice>,
    pub layout: vk::DescriptorSetLayout,
}
pub struct DescriptorLayoutBuilder {
    device: Arc<ReplacingDevice>,
    flags: vk::DescriptorSetLayoutCreateFlags,
    bindings: Vec<vk::DescriptorSetLayoutBinding>,
    bindflags: Vec<Vec<vk::DescriptorBindingFlags>>,
    bindflags_ci: Vec<vk::DescriptorSetLayoutBindingFlagsCreateInfo>,
}
impl DescriptorLayoutBuilder {
    pub fn new(device: Arc<ReplacingDevice>) -> Self {
        Self { device, flags: vk::DescriptorSetLayoutCreateFlags::empty(), bindings: vec![], bindflags: vec![], bindflags_ci: vec![] }
    }
    pub fn add_binding(mut self, binding: u32, ty: vk::DescriptorType, descriptor_count: u32, flags: vk::ShaderStageFlags) -> Self {
        let binding = vk::DescriptorSetLayoutBinding {
            binding: binding,
            descriptor_count: descriptor_count,
            descriptor_type: ty,
            stage_flags: flags,
            ..Default::default()
        };
        self.bindings.push(binding);
        self
    }
    pub fn set_flag(mut self, flags: vk::DescriptorSetLayoutCreateFlags) -> Self {
        self.flags = flags;
        self
    }
    pub fn add_binding_flag(mut self, flags: Vec<vk::DescriptorBindingFlags>) -> Self {
        self.bindflags.push(flags);
        let last = self.bindflags.last().unwrap();
        self.bindflags_ci.push(vk::DescriptorSetLayoutBindingFlagsCreateInfo {
            binding_count: last.len() as u32,
            p_binding_flags: last.as_ptr(),
            ..Default::default()
        });
        self
    }
    pub fn build(self) -> DescriptorLayout{
        // let flags = [
        //     vk::DescriptorBindingFlags::PARTIALLY_BOUND
        // ];
        // let bindind_ext = [vk::DescriptorSetLayoutBindingFlagsCreateInfo {
        //     binding_count: flags.len() as u32,
        //     p_binding_flags: flags.as_ptr(),
        //     ..Default::default()
        // }];
        let bindings = if self.bindflags_ci.is_empty() {
            std::ptr::null()
        } else {
            self.bindflags_ci.as_ptr()
        };
        let create_info = vk::DescriptorSetLayoutCreateInfo {
            p_bindings: self.bindings.as_ptr(),
            binding_count: self.bindings.len() as u32,
            flags: self.flags,
            p_next: bindings as *const _ as _,
            ..Default::default()
        };
        let layout = unsafe { self.device.device.create_descriptor_set_layout(&create_info, None).unwrap() };
        DescriptorLayout { layout, device: self.device.clone() }
    }
}

impl Drop for DescriptorLayout {
    fn drop(&mut self) {
        unsafe {
            self.device.device.destroy_descriptor_set_layout(self.layout, None);
        }
    }
}