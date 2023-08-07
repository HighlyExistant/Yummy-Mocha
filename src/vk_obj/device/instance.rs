// This file will replace the instance code just not yet

use ash::{vk::{InstanceCreateInfo, ApplicationInfo}, Entry, extensions::ext::DebugUtils};
use ash_window;
use raw_window_handle::{self};

pub enum ApiVersion {
    Type1_0 = 4194304,
    Type1_1 = 4198400,
    Type1_2 = 4202496,
    Type1_3 = 4206592
}

pub struct VulkanInstance {
    pub instance: ash::Instance,
}
pub struct VulkanInstanceBuilder {
    entry: Entry,
    api_version: u32,
    extensions: Vec<*const i8>,
    layers: Vec<*const i8>,
}
impl VulkanInstanceBuilder {
    pub fn new() -> Self {
        Self { entry: Entry::linked(), api_version: 0, extensions: vec![], layers: vec![] }
    }
    pub fn set_version(mut self, version: ApiVersion) -> Self {
        self.api_version = version as u32;
        self
    }
    pub fn enable_window_extensions(mut self, display: raw_window_handle::RawDisplayHandle) -> Self {
        let mut window_required_extensions = ash_window::enumerate_required_extensions(
            display
        )
        .unwrap().to_vec();
        self.extensions.append(&mut window_required_extensions);
        self
    }
    pub fn enable_debugging(mut self) -> Self {
        self.extensions.push(DebugUtils::name().as_ptr());
        let validation_layers = b"VK_LAYER_KHRONOS_validation\0";
        self.layers.push(validation_layers.as_ptr() as *const i8);
        self
    }
    // Requires Vulkan Version 1.2
    pub fn enable_descriptor_indexing(mut self) -> Self {
        let indexing_layers = b"SPV_EXT_descriptor_indexing\0";
        self.layers.push(indexing_layers.as_ptr() as *const i8);
        self
    }
    pub fn build(mut self) -> VulkanInstance {
        
        let app_info = ApplicationInfo {
            api_version: self.api_version,
            ..Default::default()
        };

        let create_info = InstanceCreateInfo {
            enabled_extension_count: self.extensions.len() as u32,
            pp_enabled_extension_names: self.extensions.as_ptr(),
            enabled_layer_count: self.layers.len() as u32,
            pp_enabled_layer_names: self.layers.as_ptr(),
            p_application_info: &app_info,
            ..Default::default()
        };
        let instance = unsafe { self.entry.create_instance(&create_info, None).unwrap() }; 

        VulkanInstance { instance: instance }
    }
}

impl VulkanInstance {
    pub fn builder() -> VulkanInstanceBuilder {
        VulkanInstanceBuilder::new()
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}