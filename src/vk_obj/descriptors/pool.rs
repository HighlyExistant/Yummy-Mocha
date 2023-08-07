#![allow(unused)]
use ash::vk;
use std::sync::Arc;

use crate::vk_obj::device::ReplacingDevice;

pub struct DescriptorPool {
    device: Arc<ReplacingDevice>,
    pub pool: vk::DescriptorPool
}
impl DescriptorPool {
    pub fn allocate(&self, device: std::sync::Arc<ReplacingDevice>, layouts: *const vk::DescriptorSetLayout, count: u32) -> Vec<vk::DescriptorSet> {
        let info = vk::DescriptorSetAllocateInfo {
            p_set_layouts: layouts,
            descriptor_pool: self.pool,
            descriptor_set_count: count,
            ..Default::default()
        };
        unsafe { device.device.allocate_descriptor_sets(&info).unwrap() }
    }
}
pub struct DescriptorPoolBuilder {
    device: std::sync::Arc<ReplacingDevice>,
    pools: Vec<vk::DescriptorPoolSize>,
    flag: vk::DescriptorPoolCreateFlags,
    max_sets: u32
}
impl DescriptorPoolBuilder {
    pub fn new(device: std::sync::Arc<ReplacingDevice>) -> Self {
        Self { device, pools: vec![], flag: vk::DescriptorPoolCreateFlags::empty(), max_sets: 0 }
    }
    pub fn add_pool_size(mut self, ty: vk::DescriptorType, descriptor_count: u32) -> Self {
        let pool = vk::DescriptorPoolSize {
            descriptor_count: descriptor_count,
            ty,
        };
        self.pools.push(pool);
        self
    }
    pub fn set_flag(mut self, flag: vk::DescriptorPoolCreateFlags) -> Self {
        self.flag = flag;
        self
    }
    pub fn set_max_sets(mut self, max: u32) -> Self {
        self.max_sets = max;
        self
    }
    pub fn build(self) -> DescriptorPool {
        let create_info = vk::DescriptorPoolCreateInfo {
           pool_size_count: self.pools.len() as u32,
           p_pool_sizes: self.pools.as_ptr(),
           flags: self.flag,
           max_sets: self.max_sets,
            ..Default::default()
        };
        let pool = unsafe { self.device.device.create_descriptor_pool(&create_info, None).unwrap() };
        DescriptorPool { pool, device: self.device.clone() }
    }
}
impl Drop for DescriptorPool {
    fn drop(&mut self) {
        unsafe {
            self.device.device.destroy_descriptor_pool(self.pool, None);
        }
    }
}