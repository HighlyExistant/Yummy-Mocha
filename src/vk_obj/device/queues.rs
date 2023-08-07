use std::{default, collections::{HashMap, HashSet}, ops::Add};

use ash::{vk, Device};

pub struct DeviceQueues {
    pub device: ash::Device,
    pub queues: HashMap<u32, (vk::CommandPool, Vec<vk::Queue>)>,
    /// Contains a queue with Graphics, Surface, Compute and Transfer support. This queue is typically
    /// slower than a queue specialized in these operations.
    pub general_idx: Option<u32>,
    /// Contains a queue with Graphics Support
    pub graphics_idx: Option<u32>,
    /// Contains a queue with Surface Support
    pub surface_idx: Option<u32>,
    /// Contains a queue with Compute Support
    pub compute_idx: Option<u32>,
    /// Contains a queue with Transfer Support
    pub transfer_idx: Option<u32>,
}
pub enum DeviceQueueCategory {
    General,
    Graphics,
    Surface,
    Compute,
    Transfer,
}
pub struct QueueInfo {
    pub queue_index: u32,
    pub queue_count: u32,
    pub pool_flags: vk::CommandPoolCreateFlags,
    pub flags: vk::QueueFlags
}
impl DeviceQueues {
    pub fn new(queueinfo: &Vec<QueueInfo>, device: &ash::Device) -> Self {
        let mut queues: HashMap<u32, (vk::CommandPool, Vec<vk::Queue>)> = HashMap::new();
        
        let mut general_idx: Option<u32> = None;
        /// The following queues are made for specific tasks in mind, if the queues
        /// made for these tasks are not found these will default into a general queue,
        /// if the general queue is not found then these will remain None.
        let mut graphics_idx: Option<u32> = None;
        let mut surface_idx: Option<u32> = None;
        let mut compute_idx: Option<u32> = None;
        let mut transfer_idx: Option<u32> = None;

        for info in queueinfo {
            let create_info = vk::CommandPoolCreateInfo {
                queue_family_index: info.queue_index,
                flags: info.pool_flags,
                ..Default::default()
            };

            let pool = unsafe { device.create_command_pool(&create_info, None).unwrap() };
            queues.insert(info.queue_index, (pool, Vec::new()));
            if let Some((pool, queuevec)) = queues.get_mut(&info.queue_index) {
                for i in 0..info.queue_count {
                    let queue = unsafe { device.get_device_queue(info.queue_index, i) };
                    queuevec.push(queue);
                }
            }
            if info.flags.contains(vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE | vk::QueueFlags::TRANSFER) {
                general_idx = Some(info.queue_index);
            } else if info.flags.contains(vk::QueueFlags::GRAPHICS) && !info.flags.contains(vk::QueueFlags::COMPUTE) {
                graphics_idx = Some(info.queue_index);
                surface_idx  = Some(info.queue_index);
            } else if info.flags.contains(vk::QueueFlags::COMPUTE) && !info.flags.contains(vk::QueueFlags::GRAPHICS) {
                compute_idx = Some(info.queue_index);
            } else if info.flags.contains(vk::QueueFlags::TRANSFER) {
                transfer_idx = Some(info.queue_index);
            }
        }
        if let Some(idx) = general_idx {
            if graphics_idx.is_none() {
                surface_idx  = Some(idx);
                graphics_idx = Some(idx);
            }
            if compute_idx.is_none() {
                graphics_idx = Some(idx);
            }
            if transfer_idx.is_none() {
                graphics_idx = Some(idx);
            }
        }
        Self { device: device.clone(), queues, general_idx, graphics_idx, surface_idx, compute_idx, transfer_idx }
    }
    pub fn get_graphics(&self, idx: usize) -> Option<vk::Queue> {
        if let Some(i) = self.graphics_idx {
            Some(self.queues.get(&i).unwrap().1[idx])
        } else if let Some(i) = self.general_idx {
            Some(self.queues.get(&i).unwrap().1[idx])
        } else {
            None
        }
    }
    pub fn get_present(&self, idx: usize) -> Option<vk::Queue> {
        if let Some(i) = self.surface_idx {
            Some(self.queues.get(&i).unwrap().1[idx])
        } else if let Some(i) = self.general_idx {
            Some(self.queues.get(&i).unwrap().1[idx])
        } else {
            None
        }
    }
    pub fn create_command_buffers(&self, device: &Device, level: vk::CommandBufferLevel, category: DeviceQueueCategory, count: u32) -> Vec<vk::CommandBuffer> {
        let pool = self.get_pool(&category);
        let info = vk::CommandBufferAllocateInfo {
            level,
            command_buffer_count: count,
            command_pool: pool,
            ..Default::default()
        };
        unsafe { device.allocate_command_buffers(&info).unwrap() }
    }
    pub fn get_queue(&self, category: &DeviceQueueCategory, idx: usize) -> vk::Queue {
        match category {
            DeviceQueueCategory::Compute => {
                if let Some(idx_) = self.compute_idx {
                    self.queues.get(&idx_).unwrap().1[idx]
                } else {
                    self.queues.get(&self.general_idx.unwrap()).unwrap().1[idx]
                }
            }
            DeviceQueueCategory::Graphics => {
                if let Some(idx_) = self.graphics_idx {
                    self.queues.get(&idx_).unwrap().1[idx]
                } else {
                    self.queues.get(&self.general_idx.unwrap()).unwrap().1[idx]
                }
            }
            DeviceQueueCategory::Surface => {
                self.queues.get(&self.general_idx.unwrap()).unwrap().1[idx]
            }
            DeviceQueueCategory::Transfer => {
                if let Some(idx_) = self.transfer_idx {
                    self.queues.get(&idx_).unwrap().1[idx]
                } else {
                    self.queues.get(&self.general_idx.unwrap()).unwrap().1[idx]
                }
            }
            _ => {
                self.queues.get(&self.general_idx.unwrap()).unwrap().1[idx]
            }
        }
    }
    pub fn get_pool(&self, category: &DeviceQueueCategory) -> vk::CommandPool {
        match category {
            DeviceQueueCategory::Compute => {
                if let Some(idx) = self.compute_idx {
                    self.queues.get(&idx).unwrap().0
                } else {
                    self.queues.get(&self.general_idx.unwrap()).unwrap().0
                }
            }
            DeviceQueueCategory::Graphics => {
                if let Some(idx) = self.graphics_idx {
                    self.queues.get(&idx).unwrap().0
                } else {
                    self.queues.get(&self.general_idx.unwrap()).unwrap().0
                }
            }
            DeviceQueueCategory::Surface => {
                self.queues.get(&self.general_idx.unwrap()).unwrap().0
            }
            DeviceQueueCategory::Transfer => {
                if let Some(idx) = self.transfer_idx {
                    self.queues.get(&idx).unwrap().0
                } else {
                    self.queues.get(&self.general_idx.unwrap()).unwrap().0
                }
            }
            _ => {
                self.queues.get(&self.general_idx.unwrap()).unwrap().0
            }
        }
    }
    pub fn single_time_commands(&self, device: &Device, category: DeviceQueueCategory, idx: usize) -> vk::CommandBuffer {
        let cmd = self.create_command_buffers(device, vk::CommandBufferLevel::PRIMARY, category, 1)[idx];

        let begin_info = vk::CommandBufferBeginInfo {
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            ..Default::default()
        };
        unsafe { device.begin_command_buffer(cmd, &begin_info).unwrap() };
        cmd
    }
    pub fn end_single_time_commands(&self, device: &Device, category: DeviceQueueCategory, idx: usize, command_buffer: vk::CommandBuffer) {
        unsafe { device.end_command_buffer(command_buffer).unwrap() };
        let info = vk::SubmitInfo {
            command_buffer_count: 1,
            p_command_buffers: &command_buffer,
            ..Default::default()
        };
        unsafe { 
            let queue = self.get_queue(&category, idx);
            let pool = self.get_pool(&category);
            device.queue_submit(queue, &[info], vk::Fence::null()).unwrap();
            device.queue_wait_idle(queue).unwrap();
            device.free_command_buffers(pool, &[command_buffer]);
        }
    }
}

impl Drop for DeviceQueues {
    fn drop(&mut self) {
        for (idx, (pool, queues)) in self.queues.iter() {
            unsafe {
                self.device.destroy_command_pool(*pool, None);
            }
        }
    }
}