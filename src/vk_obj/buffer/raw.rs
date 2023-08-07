use ash::vk::{self, Extent3D, Offset3D, ImageSubresourceLayers};
use std::{sync::Arc, collections::btree_set::Iter};

use crate::vk_obj::device::{ReplacingDevice, queues::DeviceQueueCategory};


pub struct Buffer<T> {
    device: Arc<ReplacingDevice>,
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub capacity: vk::DeviceSize,
    length: usize,
    pub mapped: *mut T,
}

impl<T> Buffer<T> {
    /// Contstructs a new *buffer* using *Arc<<Device>>*
    /// # Examples
    /// ```
    /// use holly::buffer::allocator;
    /// use holly::buffer::raw;
    /// fn main() {
    ///     ...
    ///     let buffer = raw::Buffer::new(device.clone(), 4096, 
    ///         vk::BufferUsageFlags::TRANSFER_SRC, 
    ///         vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
    ///     );
    /// }
    /// ```
    pub fn new(device: Arc<ReplacingDevice>, size: usize, usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags) -> Self {
        let buffer = device.allocate_buffer(size, usage, properties);
        let requirements = unsafe { device.device.get_buffer_memory_requirements(buffer) };
        let memory_index = Self::get_memory_type_index(device.clone(), properties, requirements);

        let alloc_info = vk::MemoryAllocateInfo {
            allocation_size: requirements.size,
            memory_type_index: memory_index,
            ..Default::default()
        };
        
        let memory = unsafe { device.device.allocate_memory(&alloc_info, None).unwrap() };
        unsafe { device.device.bind_buffer_memory(buffer, memory, 0).unwrap() };
        
        Self { buffer, memory, capacity: size as u64, length: 0, mapped: [].as_mut_ptr() as *mut T, device: device.clone() }
    }
    fn get_memory_type_index(device: Arc<ReplacingDevice>, properties: vk::MemoryPropertyFlags, requirements: vk::MemoryRequirements) -> u32 {
        let memory_properties = unsafe { device.instance.instance.get_physical_device_memory_properties(device.physical_device) };
        let i = (0..memory_properties.memory_type_count).find_map(|i| {
            if requirements.memory_type_bits & (1 << i) == (1 << i) &&
				memory_properties.memory_types[i as usize].property_flags & properties == properties {
				Some(i)
			} else {
                None
            }
        }).unwrap();

        i
    }
    pub fn mapping(&mut self, device: Arc<ReplacingDevice>, size: usize, offset: vk::DeviceSize) {
        self.mapped = unsafe { device.device.map_memory(self.memory, offset, size as u64, vk::MemoryMapFlags::empty()).unwrap() } as *mut T;
    }
    pub fn append(&mut self, data: &Vec<T>) {
        let size = data.len() * std::mem::size_of::<T>();
        if !self.mapped.is_null() && size <= self.capacity as usize  {
            unsafe {
                let location = self.mapped.add(self.length * std::mem::size_of::<T>()) as *mut libc::c_void;
                libc::memcpy(
                location, 
                data.as_ptr() as *const libc::c_void, 
                size); 
            };
        }
        self.length += data.len();
        if cfg!(debug_assertions) {
            if (self.length * std::mem::size_of::<T>()) > self.capacity as usize {
                panic!("length of data should not exceed capacity that was specified at buffer allocation")
            }
        }
    }
    pub fn unmapping(&self, device: std::sync::Arc<ReplacingDevice>) {
        if !self.mapped.is_null() {
            unsafe { device.device.unmap_memory(self.memory) };
        }
    }
    pub fn from_iter(device: Arc<ReplacingDevice>, usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags, iter: Iter<T>) {
        let size = iter.len() * std::mem::size_of::<T>();
        let buffer = device.allocate_buffer(size, usage, properties);
        let requirements = unsafe { device.device.get_buffer_memory_requirements(buffer) };
        let memory_index = Self::get_memory_type_index(device.clone(), properties, requirements);

        let alloc_info = vk::MemoryAllocateInfo {
            allocation_size: requirements.size,
            memory_type_index: memory_index,
            ..Default::default()
        };
        
        let memory = unsafe { device.device.allocate_memory(&alloc_info, None).unwrap() };
        unsafe { device.device.bind_buffer_memory(buffer, memory, 0).unwrap() };
        let mut this = Self { buffer, memory, capacity: size as u64, length: iter.len(), mapped: [].as_mut_ptr() as *mut T, device: device.clone() };
        this.mapping(device.clone(), size, 0);
        let mut mapped = this.mapped;
        for val in iter {
            
            unsafe { libc::memcpy(mapped as *mut libc::c_void , val as *const _ as _, std::mem::size_of::<T>()); };
            mapped = unsafe { mapped.add(std::mem::size_of::<T>()) };
        }
    }
    pub fn from_vec(device: Arc<ReplacingDevice>, usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags, vec: &Vec<T>) -> Self {
        let size = vec.len() * std::mem::size_of::<T>();
        let buffer = device.allocate_buffer(size, usage, properties);
        let requirements = unsafe { device.device.get_buffer_memory_requirements(buffer) };
        
        let i = Self::get_memory_type_index(device.clone(), properties, requirements);

        let alloc_info = vk::MemoryAllocateInfo {
            allocation_size: requirements.size,
            memory_type_index: i as u32,
            ..Default::default()
        };
        
        let memory = unsafe { device.device.allocate_memory(&alloc_info, None).unwrap() };
        unsafe { device.device.bind_buffer_memory(buffer, memory, 0).unwrap() };
        
        let mut ret = Self { buffer, memory, capacity: size as u64, length: 0, mapped: [].as_mut_ptr() as *mut T, device: device.clone() };
        ret.mapping(device.clone(), size, 0);

        ret.append(vec);

        ret
    }
    pub fn to_image(&self, device: Arc<ReplacingDevice>, image: &vk::Image, width: u32, height: u32) {
        let command_buffer = device.single_time_commands(crate::vk_obj::device::queues::DeviceQueueCategory::Graphics);
        let copy = vk::BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_subresource: ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
                ..Default::default()
            },
            image_offset: Offset3D {
                x: 0,
                y: 0,
                z: 0,
            },
            image_extent: Extent3D {
                width,
                height,
                depth: 1,
            }
        };
        unsafe { device.device.cmd_copy_buffer_to_image(command_buffer, self.buffer, *image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[copy]) };
        device.end_single_time_commands(command_buffer, DeviceQueueCategory::Graphics);
    }
    pub fn len(&self) -> usize {
        self.length
    }
    pub fn capacity(&self) -> usize {
        (self.capacity as usize) / std::mem::size_of::<T>() 
    }
    pub fn capacity_in_bytes(&self)  -> usize {
        self.capacity as usize
    }
    pub fn read_memory(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.mapped, self.capacity as usize) }
    }
}

impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {
        unsafe { self.device.device.free_memory(self.memory, None) };
        unsafe { self.device.device.destroy_buffer(self.buffer, None) };
    }
}