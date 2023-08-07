#![allow(unused)]
use ash::{vk::{self, Extent2D, QueueFamilyProperties}, Entry};
pub mod queues;
mod instance;
use ash_window;
use raw_window_handle::{ HasRawDisplayHandle, HasRawWindowHandle};
use std::{sync::Arc, collections::HashSet};

use crate::vk_obj::device::queues::QueueInfo;

use self::queues::{DeviceQueues, DeviceQueueCategory};
// use self::{replacedevice::LogicalDevice, queues::DeviceQueues};
#[derive(Clone)]
pub enum WindowOption {
    Winit(Arc<winit::window::Window>),
}
impl WindowOption {
    pub fn get_extent2d(&self) -> ash::vk::Extent2D {
        match self {
            WindowOption::Winit(b) => {
                let inner = b.inner_size();
                return Extent2D {
                    width: inner.width,
                    height: inner.height,
                };
            }
        };
    }
}
#[derive(Default)]
pub struct QueueFamilyIndices {
    pub graphics: Option<u32>,
    pub surface: Option<u32>,
}

#[derive(Default)]
pub struct SwapchainSupport {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}// This file will replace the device code just not yet

pub struct LogicalDeviceQueueIndices {
    pub graphics: Option<u32>,
    pub compute_only: Option<u32>,
    pub transfer_only: Option<u32>,
}
pub struct LogicalDeviceBuilder {
    window: Option<std::sync::Arc<winit::window::Window>>,
    queue_support: Vec<vk::QueueFlags>,
    create_info: Vec<vk::DeviceQueueCreateInfo>,
    priorities: Vec<f32>,
    extensions: Vec<*const i8>
}
pub struct LogicalDevice {
    pub instance: instance::VulkanInstance,
    pub surface: Option<vk::SurfaceKHR>,
    pub physical_device: vk::PhysicalDevice,
    pub device: ash::Device,
    pub queues: DeviceQueues,
    // this field is used so that we can drop the surface
    pub surface_functions: ash::extensions::khr::Surface
}

impl LogicalDeviceBuilder {
    pub fn new() -> Self {
        Self { 
            window: None,
            queue_support: vec![],
            create_info: vec![],
            priorities: vec![],
            extensions: vec![],
        }
    }
    /// by using this function you are telling Vulkan
    /// you want to use surface extensions for your application.
    pub fn set_window(mut self, window: std::sync::Arc<winit::window::Window>) -> Self {
        self.window = Some(window);
        self
    }
    pub fn add_swapchain_extension(mut self) -> Self {
        self.extensions.push(ash::extensions::khr::Swapchain::name().as_ptr());
        self
    }
    pub fn add_queue(mut self, queue_count: u32, queue_family_index: u32) {
        self.priorities.push(1.0);
        let ci = vk::DeviceQueueCreateInfo {
            p_queue_priorities: self.priorities.last().unwrap(),
            queue_count,
            queue_family_index,
            ..Default::default()
        };
    }
    pub fn check_queue_support(mut self, flag: vk::QueueFlags) -> Self {
        self.queue_support.push(flag);
        self
    }
    pub fn build<F>(mut self, entry: &Entry, f: F) -> LogicalDevice
        where F: Fn(&Vec<QueueFamilyProperties>, &vk::PhysicalDevice, &Option<ash::vk::SurfaceKHR>, &ash::extensions::khr::Surface) -> Vec<(u32, u32, vk::CommandPoolCreateFlags)> {
        // Instance and Surface Creation
        let mut surface_extensions = false;
        let mut instancebuilder = instance::VulkanInstance::builder()
            .set_version(instance::ApiVersion::Type1_0)
            .enable_debugging();
        
        let (instance, surface) = if let Some(window) = &self.window {
            instancebuilder = instancebuilder.enable_window_extensions((*window).raw_display_handle());
            surface_extensions = true;
            
            let vkinstace = instancebuilder.build();
            let surface: vk::SurfaceKHR = Self::create_surface_winit(&entry, &vkinstace.instance, &window.clone());
            (vkinstace, Some(surface))
        } else {
            let vkinstace = instancebuilder.build();
            (vkinstace, None)
        };
        // Choosing a Physical Device
        let (physical_device, surface_functions) =  self.choose_physical_device(&entry, &instance.instance, &surface);
        
        // getting queue create info
        let physical_device_features = vk::PhysicalDeviceFeatures2 {
            ..Default::default()
        };

        let properties = unsafe { instance.instance.get_physical_device_queue_family_properties(physical_device) };
        let indices = HashSet::<u32>::new();
        let queues = f(&properties, &physical_device, &surface, &surface_functions);
        let info: Vec<vk::DeviceQueueCreateInfo> = queues.iter().map(|prop|{
            vk::DeviceQueueCreateInfo {
                p_queue_priorities: &1.0,
                queue_family_index: prop.0,
                queue_count: prop.1,
                ..Default::default()
            }
        }).collect();
        println!("{:?}", info);
        let queueinfo: Vec<QueueInfo> = queues.iter().map(|prop|{
            QueueInfo {
                queue_index: prop.0,
                queue_count: prop.1,
                pool_flags: prop.2,
                flags: properties[prop.0 as usize].queue_flags,
            }
        }).collect();
        let create_info = vk::DeviceCreateInfo {
            pp_enabled_extension_names: self.extensions.as_ptr(),
            enabled_extension_count: self.extensions.len() as u32,
            p_queue_create_infos: info.as_ptr(),
            queue_create_info_count: info.len() as u32,
            ..Default::default()
        };
        let device = unsafe { instance.instance.create_device(physical_device, &create_info, None).unwrap() };
        let queues = DeviceQueues::new(&queueinfo, &device);
        LogicalDevice { instance, surface, physical_device, device, queues, surface_functions }
    }

    fn create_surface_winit(entry: &Entry, instance: &ash::Instance, window: &winit::window::Window) -> vk::SurfaceKHR {
        let display = window.raw_display_handle();
        let window_hwnd = window.raw_window_handle();
        unsafe { ash_window::create_surface(entry, instance, display, window_hwnd, None).unwrap() }
    }
    fn choose_physical_device(&self, entry: &Entry, vkinstance: &ash::Instance, surface: &Option<vk::SurfaceKHR>) -> (vk::PhysicalDevice, ash::extensions::khr::Surface) {
        let physical_devices = unsafe { vkinstance.enumerate_physical_devices().unwrap() };
        let surface_funcs = ash::extensions::khr::Surface::new(entry, vkinstance);
        
        unsafe { 
            let physical_device = physical_devices.iter().find_map(|device| {
                vkinstance.get_physical_device_queue_family_properties(*device)
                .iter().enumerate()
                .find_map(|(i, info)| {
                    let mut support = true;
                    for condition in &self.queue_support {
                        support = support && info.queue_flags.contains(*condition);
                    }
                    if let Some(s) = surface {
                        support = support && surface_funcs.get_physical_device_surface_support(*device,i as u32,*s,).unwrap();
                    }
                    if support {
                        Some((*device))
                    } else {
                        None
                    }
                })
            }).unwrap();
            (physical_device, surface_funcs)
        }
    }
    fn create_commandpool(device: &ash::Device, queue_index: u32) -> vk::CommandPool {
        let create_info = vk::CommandPoolCreateInfo {
            queue_family_index: queue_index,
            flags: vk::CommandPoolCreateFlags::TRANSIENT | vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            ..Default::default()
        };
        unsafe { device.create_command_pool(&create_info, None).unwrap() }
    }
}

impl LogicalDevice {
    
    pub fn allocate_buffer(&self, size: usize, usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags) -> vk::Buffer {
        let create_info = vk::BufferCreateInfo {
            size: size as u64,
            usage: usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let buffer = unsafe { self.device.create_buffer(&create_info, None).unwrap() };
        buffer
    }
    pub fn create_command_buffers(&self, level: vk::CommandBufferLevel, category: DeviceQueueCategory, count: u32) -> Vec<vk::CommandBuffer> {
        self.queues.create_command_buffers(&self.device, level, category, count)
    }
    pub fn single_time_commands(&self, category: DeviceQueueCategory) -> vk::CommandBuffer {
        self.queues.single_time_commands(&self.device, category, 0)
    }
    pub fn end_single_time_commands(&self, command_buffer: vk::CommandBuffer, category: DeviceQueueCategory) {
        self.queues.end_single_time_commands(&self.device, category, 0, command_buffer)
    }
    
    fn query_swapchain_support(surface_funcs: &ash::extensions::khr::Surface, physical_device: &vk::PhysicalDevice, surface: &vk::SurfaceKHR) -> SwapchainSupport {
        let support = unsafe { 
            SwapchainSupport {
                capabilities: surface_funcs.get_physical_device_surface_capabilities(*physical_device, *surface).unwrap(),
                formats: surface_funcs.get_physical_device_surface_formats(*physical_device, *surface).unwrap(),
                present_modes: surface_funcs.get_physical_device_surface_present_modes(*physical_device, *surface).unwrap(),
            } 
        };
        support
    }
    pub fn swapchain_support(&self) -> SwapchainSupport {
        Self::query_swapchain_support(&self.surface_functions, &self.physical_device, &self.surface.unwrap())
    }
    pub fn create_image(
        &self,
        info: &vk::ImageCreateInfo
    ) -> (vk::Image, vk::DeviceMemory) {
        let image = unsafe { self.device.create_image(info, None).unwrap() };
        // Allocate Memory
        let requirements = unsafe { self.device.get_image_memory_requirements(image) };
        let memory_properties = unsafe { self.instance.instance.get_physical_device_memory_properties(self.physical_device) };
        let mut memory_type_index = 0;
        for i in 0..memory_properties.memory_type_count {
            if requirements.memory_type_bits & (1 << i) == (1 << i)
            && memory_properties.memory_types[i as usize].property_flags & vk::MemoryPropertyFlags::DEVICE_LOCAL == (vk::MemoryPropertyFlags::DEVICE_LOCAL) {
                memory_type_index = i;
                break;
            }
        }

        let alloc_info = vk::MemoryAllocateInfo {
            allocation_size: requirements.size,
            memory_type_index: memory_type_index,
            ..Default::default()
        };
        let memory = unsafe { self.device.allocate_memory(&alloc_info, None).unwrap() };
       
       unsafe { self.device.bind_image_memory(image, memory, 0).unwrap() };
        (image, memory)
    }
    pub unsafe fn find_supported_format(
        &self, candidates: &Vec<vk::Format> ,  tiling: vk::ImageTiling,  features: vk::FormatFeatureFlags) -> vk::Format {
      for format in candidates {
        let props = self.instance.instance.get_physical_device_format_properties(self.physical_device, *format);
    
        if tiling == vk::ImageTiling::LINEAR && (props.linear_tiling_features & features) == features {
          return *format;
        } else if 
            tiling == vk::ImageTiling::OPTIMAL && (props.optimal_tiling_features & features) == features {
          return *format;
        }
      }
      panic!("failed to find supported format!");
    }
}

impl Drop for LogicalDevice {
    fn drop(&mut self) {
        unsafe { 
            self.device.destroy_device(None);
            self.instance.instance.destroy_instance(None);
            if let Some(surface) = self.surface {
                self.surface_functions.destroy_surface(surface, None);
            }
        };
    }
}
pub type ReplacingDevice = LogicalDevice;