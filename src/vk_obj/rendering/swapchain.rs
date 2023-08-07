#![allow(unused)]
// use insomniac::linear::fvec2::FVec2;
use ash::{vk::{self, SurfaceFormatKHR, PresentModeKHR, Extent2D, SharingMode, CompositeAlphaFlagsKHR, SwapchainKHR, ImageSubresourceRange, ImageViewType, SampleCountFlags, AttachmentLoadOp, AccessFlags, Extent3D, FenceCreateFlags}, extensions::khr, Instance};

use crate::vk_obj::device::{self, ReplacingDevice};
#[derive(Default, Clone, Copy)]
pub struct ImageResource {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub memory: vk::DeviceMemory,
}
pub struct Swapchain {
    pub device: std::sync::Arc<ReplacingDevice>,
    swapchain_funcs: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub old: vk::SwapchainKHR,
    pub format: vk::SurfaceFormatKHR,
    pub extent: Extent2D,
    pub image_count: u32,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub renderpass: vk::RenderPass,
    pub depth_format: vk::Format,
    depth_resources: Vec<ImageResource>,
    pub frambuffers: Vec<vk::Framebuffer>,
    image_available: Vec<vk::Semaphore>, 
    rendering_done: Vec<vk::Semaphore>, 
    in_flight_fence: Vec<vk::Fence>, 
    in_flight_images: Vec<Option<vk::Fence>>,
    pub current_frame: usize,
}
pub const MAX_FRAMES: usize = 2;

impl Swapchain {
    pub fn new(device: std::sync::Arc<ReplacingDevice>, extent: Extent2D, old: SwapchainKHR) -> Self {
        let swapchain_funcs: khr::Swapchain = khr::Swapchain::new(&device.instance.instance, &device.device);
        let (swapchain, format, extent, image_count) = Self::create_swapchain(&swapchain_funcs, &device, extent, old);
        let images = unsafe { swapchain_funcs.get_swapchain_images(swapchain).unwrap() };
        let image_views = Self::create_image_views(image_count, &images, format.format, &device);
        let (renderpass, depth_format) = Self::create_renderpass(&device, format.format);
        let depth_resources = Self::create_depth_resources(&device, image_count, extent, depth_format);
        let frambuffers = Self::create_framebuffers(&device, image_count, extent, &image_views, &depth_resources, &renderpass);
        let (image_available, rendering_done, in_flight_fence, in_flight_images) = Self::create_sync_resources(&device, image_count);

        Self { 
            device, 
            swapchain_funcs, 
            swapchain, 
            old, 
            format, 
            extent, 
            image_count, 
            images, 
            image_views, 
            renderpass, 
            depth_format, 
            depth_resources, 
            frambuffers, 
            image_available, 
            rendering_done, 
            in_flight_fence, 
            in_flight_images,
            current_frame: 0,
        }
    }
    fn create_swapchain(swapchain_funcs: &ash::extensions::khr::Swapchain, device: &std::sync::Arc<ReplacingDevice>, extent: Extent2D, old: vk::SwapchainKHR) -> (vk::SwapchainKHR, SurfaceFormatKHR, Extent2D, u32) {
        let details = device.swapchain_support();

        let format = Self::choose_format(&details.formats);
        let present_mode = Self::choose_present_mode(&details.present_modes);
        let window_extent = Self::choose_extent(&details.capabilities, extent);

        let mut image_count = details.capabilities.min_image_count + 1;
        if (details.capabilities.max_image_count > 0 &&
			image_count > details.capabilities.max_image_count) {
            image_count = details.capabilities.max_image_count;
		}
        
        let (sharing, index_count, indices) = if device.queues.graphics_idx.is_some() && device.queues.surface_idx.is_some() {
            if device.queues.graphics_idx != device.queues.surface_idx {
                (vk::SharingMode::CONCURRENT, 2, vec![device.queues.graphics_idx.unwrap(), device.queues.surface_idx.unwrap()])
            } else {
                (vk::SharingMode::EXCLUSIVE, 0, vec![])
            }
            
        } else {
            (vk::SharingMode::EXCLUSIVE, 0, vec![])
        };

        let create_info = vk::SwapchainCreateInfoKHR {
            surface: device.surface.unwrap(),
            min_image_count: image_count,
            image_format: format.format,
            image_color_space: format.color_space,
            image_extent: window_extent,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            pre_transform: details.capabilities.current_transform,
            composite_alpha: CompositeAlphaFlagsKHR::OPAQUE,
            present_mode: present_mode,
            clipped: 1, // VK_TRUE
            old_swapchain: old,
            // The following entries depend on whether the indices for 
            // surface and graphics are the same and can change depending on device
            // TODO: Make it compatible for more devices by using multiple queue indices in Device and adding conditional for these values
            image_sharing_mode: sharing,
            queue_family_index_count: index_count,
            p_queue_family_indices: indices.as_ptr(),
            ..Default::default()
        };
        
        let swapchain = unsafe { swapchain_funcs.create_swapchain(&create_info, None).unwrap() };
        (swapchain, format, window_extent, image_count)
    }
    fn create_image_views(image_count: u32, images: &Vec<vk::Image>, format: vk::Format, device: &std::sync::Arc<ReplacingDevice>) -> Vec<vk::ImageView> {
        let views: Vec<vk::ImageView> = images.iter().map(|image| {
            let create_info = vk::ImageViewCreateInfo {
                image: *image,
                format: format,
                view_type: ImageViewType::TYPE_2D,
                subresource_range: ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1
                },
                ..Default::default()
            };

            let view = unsafe { device.device.create_image_view(&create_info, None).unwrap() };
            view
        }).collect();
        views
    }
    fn create_renderpass(device: &std::sync::Arc<ReplacingDevice>, format: vk::Format) -> (vk::RenderPass, vk::Format) {
        // FIND DEPTH FORMAT
        let depth_format = Self::find_depth_format(device.clone());

        // Depth Attachment
        // * THIS IS GOOD STOP FRIGGIN CHECKING
        let depth_attachment = vk::AttachmentDescription {
            format: depth_format,
            samples: SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::DONT_CARE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            ..Default::default()
        };

        // * THIS IS GOOD STOP FRIGGIN CHECKING
        let depth_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            ..Default::default()
        };

        // Color Attachment
        // * THIS IS GOOD STOP FRIGGIN CHECKING
        let color_attachment = vk::AttachmentDescription {
            format: format,
            samples: SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        };
        
        // * THIS IS GOOD STOP FRIGGIN CHECKING
        let color_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            ..Default::default()
        };
        // Subpass
        // * THIS IS GOOD STOP FRIGGIN CHECKING
        let subpass = vk::SubpassDescription {
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count: 1,
            p_color_attachments: &color_ref,
            p_depth_stencil_attachment: &depth_ref,
            ..Default::default()
        };

        // * THIS IS GOOD STOP FRIGGIN CHECKING
        let dependency = vk::SubpassDependency {
            dst_subpass: 0,
            dst_access_mask: AccessFlags::COLOR_ATTACHMENT_WRITE | AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            src_subpass: u32::MAX, // VK_SUBPASS_EXTERNAL
            src_access_mask: unsafe { std::mem::transmute::<u32, AccessFlags>(0) },
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            ..Default::default()
        };
        // * THIS IS GOOD STOP FRIGGIN CHECKING
        let attachments = [color_attachment, depth_attachment];
        let create_info = vk::RenderPassCreateInfo {
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            subpass_count: 1,
            p_subpasses: &subpass,
            dependency_count: 1,
            p_dependencies: &dependency,
            ..Default::default()
        };
        let renderpass = unsafe { device.device.create_render_pass(&create_info, None).unwrap() };
        (renderpass, depth_format)
    }
    fn create_depth_resources(device: &std::sync::Arc<ReplacingDevice>,image_count: u32, extent: Extent2D, depth_format: vk::Format) -> Vec<ImageResource> {
        let mut resources: Vec<ImageResource> = vec![ImageResource::default(); image_count as usize];
        for i in 0..resources.len() {
            // Create Image
            let image_info = vk::ImageCreateInfo {
                extent: Extent3D {
                    width: extent.width,
                    height: extent.height,
                    depth: 1,
                },
                mip_levels: 1,
                format: depth_format,
                tiling: vk::ImageTiling::OPTIMAL,
                image_type: vk::ImageType::TYPE_2D,
                usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                array_layers: 1,
                samples: SampleCountFlags::TYPE_1,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                ..Default::default()
            };
            (resources[i].image, resources[i].memory) = device.create_image(&image_info);
        
            // Create Image View
            let view_info = vk::ImageViewCreateInfo {
                image: resources[i].image,
                view_type: vk::ImageViewType::TYPE_2D,
                format: depth_format,
                subresource_range: ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                    ..Default::default()
                },
                ..Default::default()
            };
            resources[i].view = unsafe { device.device.create_image_view(&view_info, None).unwrap() };
        }
        resources
    }
    fn create_framebuffers(device: &std::sync::Arc<ReplacingDevice>, image_count: u32, extent: Extent2D, color: &Vec<vk::ImageView>, depth: &Vec<ImageResource>, renderpass: &vk::RenderPass) -> Vec<vk::Framebuffer> {
        let mut frambuffers = vec![vk::Framebuffer::default(); image_count as usize];

        // * THIS IS GOOD STOP FRIGGIN CHECKING
        for i in 0..image_count as usize {
            let attachments = [color[i], depth[i].view];
            let create_info = vk::FramebufferCreateInfo {
                render_pass: *renderpass,
                attachment_count: attachments.len() as u32,
                p_attachments: attachments.as_ptr(),
                width: extent.width,
                height: extent.height,
                layers: 1,
                ..Default::default()
            };
            frambuffers[i] = unsafe { device.device.create_framebuffer(&create_info, None).unwrap() };
        }
        frambuffers
    }
    fn create_sync_resources(device: &std::sync::Arc<ReplacingDevice>,image_count: u32) -> (Vec<vk::Semaphore>, Vec<vk::Semaphore>, Vec<vk::Fence>, Vec<Option<vk::Fence>>) {
        let mut image_available = vec![vk::Semaphore::default(); MAX_FRAMES];
        let mut rendering_done = vec![vk::Semaphore::default(); MAX_FRAMES];
        let mut in_flight_fence = vec![vk::Fence::default(); MAX_FRAMES];
        let in_flight_image = vec![None; image_count as usize];

        let semaphore_info = vk::SemaphoreCreateInfo {
            ..Default::default()
        };
        let fence_info = vk::FenceCreateInfo {
            flags: FenceCreateFlags::SIGNALED,
            ..Default::default()
        };
        for i in 0..MAX_FRAMES {
            image_available[i] = unsafe { device.device.create_semaphore(&semaphore_info, None).unwrap() };
            rendering_done[i] = unsafe { device.device.create_semaphore(&semaphore_info, None).unwrap() };
            in_flight_fence[i] = unsafe { device.device.create_fence(&fence_info, None).unwrap() };
        }
        (image_available, rendering_done, in_flight_fence, in_flight_image)
    }
    fn choose_format(formats: &Vec<SurfaceFormatKHR>) -> SurfaceFormatKHR {
        
		for format in formats {
			if (format.format == ash::vk::Format::B8G8R8A8_SRGB
				&& format.color_space == ash::vk::ColorSpaceKHR::SRGB_NONLINEAR)
			{
				return *format;
			}
        }
        formats[0]
    }
    fn choose_present_mode(present_modes: &Vec<PresentModeKHR>) -> PresentModeKHR {
        for mode in present_modes {
            if *mode == PresentModeKHR::MAILBOX {
                return *mode;
            }
        }
        return PresentModeKHR::FIFO;
    }
    fn choose_extent(capabilities: &vk::SurfaceCapabilitiesKHR, extent: Extent2D) -> Extent2D {
        if capabilities.current_extent.width != std::u32::MAX {
            return capabilities.current_extent;
        }
        
        let mut actual_extent = extent;
        actual_extent.width = std::cmp::max(
			capabilities.min_image_extent.width,
			std::cmp::min(capabilities.min_image_extent.width, actual_extent.width));
            actual_extent.height = std::cmp::max(
			capabilities.min_image_extent.height,
			std::cmp::min(capabilities.min_image_extent.height, actual_extent.height));

        actual_extent
    }
    pub fn next_image(&self) -> Result<(u32, bool), vk::Result> {
        unsafe { self.device.device.wait_for_fences(&[self.in_flight_fence[self.current_frame]], true, std::u64::MAX).unwrap() };
        let result = unsafe { self.swapchain_funcs.acquire_next_image(self.swapchain, std::u64::MAX, self.image_available[self.current_frame], vk::Fence::null()) };
        result
    }
    pub fn submit(&mut self, command_buffers: Vec<vk::CommandBuffer>, index: usize) -> (u32, Result<bool, vk::Result>) {
        // Queue the image for presentation
        if let Some(fence) = self.in_flight_images[index] {
            unsafe { self.device.device.wait_for_fences(&[fence], true, std::u64::MAX).unwrap() };
        }

        self.in_flight_images[index] = Some(self.in_flight_fence[self.current_frame]);

        let wait_semaphores = [self.image_available[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.rendering_done[self.current_frame]];

        let submit_info = vk::SubmitInfo {
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: command_buffers.len() as u32,
            p_command_buffers: command_buffers.as_ptr(),
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
            ..Default::default()
        };
        unsafe { self.device.device.reset_fences(&[self.in_flight_fence[self.current_frame]]).unwrap() };
        unsafe { self.device.device.queue_submit(self.device.queues.get_graphics(0).unwrap(), &[submit_info], self.in_flight_fence[self.current_frame]).unwrap() };
        
        let image_index = index as u32;
        let swapchains = [self.swapchain];
        let present_info = vk::PresentInfoKHR {
            wait_semaphore_count: signal_semaphores.len() as u32,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &image_index as *const u32,
            ..Default::default()
        };
        let suboptimal = unsafe { self.swapchain_funcs.queue_present(self.device.queues.get_present(0).unwrap(), &present_info) };
        self.current_frame = (self.current_frame + 1) % 2;
        return (image_index, suboptimal);
    }
    fn find_depth_format(device: std::sync::Arc<ReplacingDevice>) -> vk::Format {
        return unsafe { device.find_supported_format(
                    &vec![vk::Format::D32_SFLOAT, vk::Format::D32_SFLOAT_S8_UINT, vk::Format::D24_UNORM_S8_UINT],
                    vk::ImageTiling::OPTIMAL,
                    vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
                    ) };
      }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            for i in 0..self.image_count as usize {
                // self.device.device.destroy_image(self.images[i], None);
                self.device.device.destroy_image_view(self.image_views[i], None);

                self.device.device.destroy_image(self.depth_resources[i].image, None);
                self.device.device.destroy_image_view(self.depth_resources[i].view, None);
                self.device.device.free_memory(self.depth_resources[i].memory, None);
            }
            for i in 0..MAX_FRAMES {
                self.device.device.destroy_semaphore(self.rendering_done[i], None);
                self.device.device.destroy_semaphore(self.image_available[i], None);
                self.device.device.destroy_fence(self.in_flight_fence[i], None);
            }
            for framebuffer in &self.frambuffers {
                self.device.device.destroy_framebuffer(*framebuffer, None);
            }
            self.device.device.destroy_render_pass(self.renderpass, None);
            self.swapchain_funcs.destroy_swapchain(self.swapchain, None);
        }
    }
}