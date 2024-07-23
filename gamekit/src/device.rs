//!
//! Device
//!

use ash::vk::{ColorSpaceKHR, CommandPool, Format, Handle, PhysicalDevice, PresentModeKHR, Queue, QueueFlags};
use ash::{ext, vk};

use log::{*};

use crate::api::Disposable;
use crate::error::Error;
use crate::instance::Instance;
use crate::window::Window;
use crate::constants::Constants;

pub fn required_device_extension_names() -> Vec<*const i8> {

    let mut ext = vec! [
        ash::khr::swapchain::NAME.as_ptr()
    ];

    if Constants::REQUIRE_EXTENDED_DYNAMIC_STATE {
        ext.push(ext::extended_dynamic_state::NAME.as_ptr());
    }

    if Constants::REQUIRE_EXTENDED_DYNAMIC_STATE3 {
        ext.push(ext::extended_dynamic_state3::NAME.as_ptr());
    }

    ext
}

pub struct DeviceFeatures {
    dynamic_state: bool,
    dynamic_state_3: bool
}

impl Default for DeviceFeatures {
    fn default() -> Self {
        Self {
            dynamic_state: false,
            dynamic_state_3: false
        }
    }
}

impl DeviceFeatures {
    pub fn has_dynamic_state(&self) -> bool {
        self.dynamic_state
    }

    pub fn has_dynamic_state_3(&self) -> bool {
        self.dynamic_state_3
    }
}

pub struct PhysicalDeviceInfo {
    pub obj: PhysicalDevice,
    pub graphics_family_index: u32,
    pub present_family_index: u32,
    pub mail_box_mode_support: bool,
    pub surface_format: ash::vk::SurfaceFormatKHR,
    pub uniform_buffer_alignment: usize
}

pub struct LogicalDeviceInfo {
    pub obj: ash::Device,
    pub dynamic_state_device: Option<ext::extended_dynamic_state3::Device>,
    pub graphics_queue: Queue,
    pub present_queue: Queue,
    pub device_features: DeviceFeatures
}

pub struct CommandPoolInfo {
    pub obj: CommandPool
}

pub struct Limits {
    pub uniform_buffer_alignment: usize
}

pub struct Device {
    pub physical_device: ash::vk::PhysicalDevice,
    pub graphics_family_index: u32,
    pub present_family_index: u32,
    pub mailbox_mode_support: bool,
    pub surface_format: ash::vk::SurfaceFormatKHR,
    pub obj: ash::Device,
    pub dynamic_state_device: Option<ext::extended_dynamic_state3::Device>,
    pub graphics_queue: ash::vk::Queue,
    pub present_queue: ash::vk::Queue,
    pub command_pool: ash::vk::CommandPool,
    pub limits: Limits,
    pub features: DeviceFeatures
}

impl Disposable for Device {
    fn dispose(&mut self) {
        trace!("Device::dispose");

        if !self.command_pool.is_null() {
            unsafe { self.obj.destroy_command_pool(self.command_pool, None); }
            self.command_pool = vk::CommandPool::null();
        }

        self.graphics_queue = ash::vk::Queue::null();
        self.present_queue = ash::vk::Queue::null();

        unsafe { self.obj.destroy_device(None); }

        self.physical_device = vk::PhysicalDevice::null();
    }
}

impl Device {

    pub fn new() -> Result<Self, Error> {

        let instance = crate::globals::instance();
        let window = crate::globals::window();

        let physical_device_info = Device::create_physical_device(&instance, window)?;
        let logical_device_info = Device::create_logical_device(&instance, &physical_device_info)?;
        let command_pool_info = Device::create_command_pool(&logical_device_info.obj, physical_device_info.graphics_family_index)?;

        let limits = Limits {
            uniform_buffer_alignment: physical_device_info.uniform_buffer_alignment
        };

        Ok(Self {
            physical_device: physical_device_info.obj,
            graphics_family_index: physical_device_info.graphics_family_index,
            present_family_index: physical_device_info.present_family_index,
            mailbox_mode_support: physical_device_info.mail_box_mode_support,
            surface_format: physical_device_info.surface_format,
            obj: logical_device_info.obj,
            dynamic_state_device: logical_device_info.dynamic_state_device,
            graphics_queue: logical_device_info.graphics_queue,
            present_queue: logical_device_info.present_queue,
            command_pool: command_pool_info.obj,
            limits,
            features: logical_device_info.device_features
        })

    }

    fn create_physical_device(
        instance: &Instance,
        window: &Window) -> Result<PhysicalDeviceInfo, Error> {

        let surface_loader = &window.surface_instance;
        let surface = &window.surface;

        let devices = match unsafe { instance.obj.enumerate_physical_devices() } {
            Ok(devices) => { devices },
            Err(_) => { return Err(Error::from("failed to enumerate physical devices")) }
        };

        let required_device_extensions = required_device_extension_names();

        'device_loop: for physical_device in devices {

            let properties: vk::PhysicalDeviceProperties = unsafe { instance.obj.get_physical_device_properties(physical_device) };

            if properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU &&
               properties.device_type != vk::PhysicalDeviceType::INTEGRATED_GPU {
                continue 'device_loop;
            }

            let device_extension_properties = unsafe {
                instance.obj.enumerate_device_extension_properties(physical_device).unwrap()
            };

            // check if physical device supports all required extensions
            for required_name in &required_device_extensions {
                let required_name_str = unsafe { std::ffi::CStr::from_ptr(*required_name) };

                let mut found = false;

                for device_extension in &device_extension_properties {
                    let extension_name_str = unsafe { std::ffi::CStr::from_ptr(device_extension.extension_name.as_ptr()) };

                    if required_name_str == extension_name_str {
                        found = true;
                    }
                }

                if !found {
                    continue 'device_loop;
                }
            }

            // check surface format
            let surface_formats = unsafe {
                surface_loader.get_physical_device_surface_formats(physical_device, surface.obj).unwrap()
            };

            let mut found_swap_space_surface_format: i32 = -1;

            for (i, surface_format) in surface_formats.iter().enumerate() {
                if surface_format.format == Format::B8G8R8A8_SRGB && surface_format.color_space == ColorSpaceKHR::SRGB_NONLINEAR {
                    found_swap_space_surface_format = i as i32;
                    break;
                }
            }

            if found_swap_space_surface_format < 0 {
                continue;
            }

            let surface_format = surface_formats[found_swap_space_surface_format as usize].clone();

            // check present mode for mailbox support
            let device_present_modes = unsafe {
                surface_loader.get_physical_device_surface_present_modes(physical_device, surface.obj).unwrap()
            };

            let mut mail_box_mode_support = false;

            for mode in device_present_modes {
                if mode == PresentModeKHR::MAILBOX {
                    mail_box_mode_support = true;
                    break;
                }
            }

            // check for graphics and presentation queue family support

            let queue_families = unsafe { instance.obj.get_physical_device_queue_family_properties(physical_device) };

            let mut graphics_family_index: i32 = -1;
            let mut present_family_index: i32 = -1;

            for (i, queue_family) in queue_families.iter().enumerate() {

                if -1 == graphics_family_index {
                    if queue_family.queue_flags.contains(QueueFlags::GRAPHICS) {
                        graphics_family_index = i as i32;
                    }
                }

                if -1 == present_family_index {
                    let present_support = unsafe { surface_loader.get_physical_device_surface_support(physical_device, i as u32, surface.obj).unwrap() };
                    if present_support {
                        present_family_index = i as i32;
                    }
                }

                if -1 != graphics_family_index && -1 != present_family_index {
                    break;
                }

            }

            if -1 == graphics_family_index || -1 == present_family_index {
                continue;
            }

            let physical_device_info = PhysicalDeviceInfo {
                obj: physical_device,
                graphics_family_index: graphics_family_index as u32,
                present_family_index: present_family_index as u32,
                mail_box_mode_support,
                surface_format,
                uniform_buffer_alignment: properties.limits.min_uniform_buffer_offset_alignment as usize
            };

            let physical_device_name = String::from( unsafe { std::ffi::CStr::from_ptr(properties.device_name.as_ptr()) }.to_str().unwrap());
            trace!("using physical device {}", physical_device_name);

            // return found device info
            return Ok(physical_device_info);

        }

        Err(Error::from("failed to find compatible physical device"))

    }

    fn create_logical_device(instance: &Instance, physical_device_info: &PhysicalDeviceInfo) -> Result<LogicalDeviceInfo, Error>{

        let physical_device = physical_device_info.obj;

        let queue_priorities = [1.0f32];

        let mut device_features = DeviceFeatures::default();

        let queue_create_infos = {
            // Vulkan specs does not allow passing an array containing duplicated family indices.
            // And since the family for graphics and presentation could be the same we need to
            // deduplicate it.
            let mut indices = vec![physical_device_info.graphics_family_index, physical_device_info.present_family_index];
            indices.dedup();

            // Now we build an array of `DeviceQueueCreateInfo`.
            // One for each different family index.
            indices
                .iter()
                .map(|index| {
                    vk::DeviceQueueCreateInfo::default()
                        .queue_family_index(*index)
                        .queue_priorities(&queue_priorities)
                })
                .collect::<Vec<_>>()
        };

        let enabled_device_extension_names = required_device_extension_names();

        let mut device_feature_selector = vk::PhysicalDeviceFeatures2::default();
        let mut feature_info_dynamic_state = vk::PhysicalDeviceExtendedDynamicStateFeaturesEXT::default();
        let mut feature_info_dynamic_state3 = vk::PhysicalDeviceExtendedDynamicState3FeaturesEXT::default();

        device_feature_selector.p_next = &mut feature_info_dynamic_state as *mut _ as *mut core::ffi::c_void;

        if Constants::REQUIRE_EXTENDED_DYNAMIC_STATE3 {
            feature_info_dynamic_state.p_next = &mut feature_info_dynamic_state3 as *mut _ as *mut core::ffi::c_void;
        }

        unsafe { instance.obj.get_physical_device_features2(physical_device, &mut device_feature_selector) };

        if feature_info_dynamic_state.extended_dynamic_state == vk::TRUE {
            device_features.dynamic_state = true;
        } else {
            return Err(Error::from("feature 'dynamic state' not supported by device"));
        }

        if Constants::REQUIRE_EXTENDED_DYNAMIC_STATE3 {
            if feature_info_dynamic_state3.extended_dynamic_state3_color_blend_enable == vk::TRUE {
                device_features.dynamic_state_3 = true;
            } else {
                return Err(Error::from("feature 'dynamic state 3' not supported by device"));
            }
        }

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&enabled_device_extension_names)
            .push_next(&mut device_feature_selector);

        let logical_device = unsafe { instance.obj.create_device(physical_device, &device_create_info, None).unwrap() };
        let graphics_queue = unsafe { logical_device.get_device_queue(physical_device_info.graphics_family_index, 0) };
        let present_queue = unsafe { logical_device.get_device_queue(physical_device_info.present_family_index, 0) };

        let dynamic_state_device = if device_features.has_dynamic_state_3() { Some(ext::extended_dynamic_state3::Device::new(&instance.obj, &logical_device)) } else { None };

        Ok(LogicalDeviceInfo{
            obj: logical_device,
            dynamic_state_device,
            graphics_queue,
            present_queue,
            device_features
        })
    }

    fn create_command_pool(device: &ash::Device, graphics_queue_family_index: u32) -> Result<CommandPoolInfo, Error> {

        let command_pool_create_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(graphics_queue_family_index);

        let command_pool = unsafe { device.create_command_pool(&command_pool_create_info, None).unwrap() };

        Ok(CommandPoolInfo{
            obj: command_pool
        })
    }

    pub fn begin_command() -> vk::CommandBuffer {
        let device = crate::globals::device();

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(device.command_pool)
            .command_buffer_count(1);

        let command_buffers = unsafe {
            device.obj.allocate_command_buffers(&command_buffer_allocate_info).unwrap()
        };

        let command_buffer = command_buffers[0];

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe { let _ = device.obj.begin_command_buffer(command_buffer, &command_buffer_begin_info); }

        command_buffer
    }

    pub fn end_command(command_buffer: vk::CommandBuffer) {
        let device = crate::globals::device();

        let command_buffers = [ command_buffer ];

        let submit_info = vk::SubmitInfo::default()
            .command_buffers(&command_buffers);

        let submit_infos = [ submit_info ];

        unsafe {
            let _ = device.obj.end_command_buffer(command_buffer);
            let _ = device.obj.queue_submit(device.graphics_queue, &submit_infos, vk::Fence::null());
            let _ = device.obj.queue_wait_idle(device.graphics_queue);
            device.obj.free_command_buffers(device.command_pool, &command_buffers);
        }
    }


}

unsafe extern "system" fn debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut core::ffi::c_void
) -> vk::Bool32 {

    let types = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "(general) ",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "(performance) ",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "(validation) ",
        _ => "",
    };

    let message = std::ffi::CStr::from_ptr((*p_callback_data).p_message);

    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => { trace!("{}{:?}", types, message); },
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => { warn!("{}{:?}", types, message); },
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => { error!("{}{:?}", types, message); },
        _ => { info!("{}{:?}", types, message); },
    }

    vk::FALSE
}
