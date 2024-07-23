//!
//! Swapchain
//!

use ash::vk::Handle;
use ash::{khr, vk};

use log::{*};

use crate::api::Disposable;
use crate::error::Error;

pub struct SwapChain {
    pub device: khr::swapchain::Device,
    pub obj: vk::SwapchainKHR,
    pub extent: vk::Extent2D,
    pub format: vk::SurfaceFormatKHR,
    pub image_count: usize
}

impl Disposable for SwapChain {
    fn dispose(&mut self) {
        trace!("SwapChain::dispose");
        self.destroy_swapchain();
    }
}

impl SwapChain {

    pub fn new() -> Result<Self, Error> {

        let options = crate::globals::options();
        let instance = crate::globals::instance();
        let window = crate::globals::window();
        let device = crate::globals::device();

        let surface_instance = &window.surface_instance;
        let surface = &window.surface;

        let surface_capabilities = unsafe {
            surface_instance.get_physical_device_surface_capabilities(
                device.physical_device, surface.obj
            ).unwrap()
        };

        let extent = {
            if surface_capabilities.current_extent.width != std::u32::MAX {
                vk::Extent2D { width: surface_capabilities.current_extent.width, height: surface_capabilities.current_extent.height }
            } else {
                let min_extent = surface_capabilities.min_image_extent;
                let max_extent = surface_capabilities.max_image_extent;
                vk::Extent2D { width: options.window_width.clamp(min_extent.width, max_extent.width), height: options.window_height.clamp(min_extent.height, max_extent.height) }
            }
        };

        let format = device.surface_format.clone();

        // set the number of swap chain image buffers
        let mut image_count = surface_capabilities.min_image_count + 1;
        if surface_capabilities.max_image_count > 0 && image_count > surface_capabilities.max_image_count {
            image_count = surface_capabilities.max_image_count;
        }

        // swap buffer mode (mailbox: triple-buffer, fifo: v-sync, immediate: no v-sync, fifo relaxed: no v-sync if late)
        let present_mode = if device.mailbox_mode_support { vk::PresentModeKHR::MAILBOX } else { vk::PresentModeKHR::FIFO };

        // create swap chain
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface.obj)
            .min_image_count(image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let queue_family_indices;
        if device.graphics_family_index != device.present_family_index {
            queue_family_indices = vec![device.graphics_family_index, device.present_family_index];
            let _ = swapchain_create_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&queue_family_indices);
        } else {
            let _ = swapchain_create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE);
        }

        let swapchain_device = khr::swapchain::Device::new(&instance.obj, &device.obj);
        let swapchain = unsafe { swapchain_device.create_swapchain(&swapchain_create_info, None).unwrap() };

        // update global metrics with swapchain extent
        let metrics = crate::globals::metrics_mut();
        metrics.set_window_size(extent.width, extent.height);

        Ok(Self {
            device: swapchain_device,
            obj: swapchain,
            extent,
            format,
            image_count: image_count as usize
        })

    }

    fn create_swapchain(&mut self) -> Result<(), Error> {
        let new_swapchain = SwapChain::new()?;

        self.device = new_swapchain.device;
        self.obj = new_swapchain.obj;
        self.extent = new_swapchain.extent;
        self.format = new_swapchain.format;

        Ok(())
    }

    fn destroy_swapchain(&mut self) {
        if !self.obj.is_null() {
            unsafe { self.device.destroy_swapchain(self.obj, None); }
            self.obj = vk::SwapchainKHR::null();
        }
    }

}
