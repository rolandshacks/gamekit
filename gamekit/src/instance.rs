//!
//! Instance
//!

use std::ffi::CStr;

use ash::{ext, vk};

use log::{*};

use crate::api::Disposable;
use crate::error::Error;

// required extension ------------------------------------------------------

pub fn required_sdl_instance_extension_names(sdl_window: &sdl2::video::Window) -> Vec<&'static str> {
    let vulkan_extensions = sdl_window.vulkan_instance_extensions().unwrap();
    vulkan_extensions
}

#[cfg(target_os = "macos")]
pub fn required_instance_os_extension_names() -> Vec<*const i8> {
    vec![
        ash::mvk::macos_surface::name().as_ptr()
    ]
}

#[cfg(all(windows))]
pub fn required_instance_os_extension_names() -> Vec<*const i8> {
    vec![
        ash::khr::win32_surface::NAME.as_ptr(),
    ]
}

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
pub fn required_instance_os_extension_names() -> Vec<*const i8> {
    vec![
        ash::khr::xlib_surface::NAME.as_ptr()
    ]
}

fn required_instance_extension_names(enable_validation_layer: bool) -> Vec<*const i8> {

    let mut v = vec![
        ash::khr::surface::NAME.as_ptr(),
        ash::khr::get_physical_device_properties2::NAME.as_ptr()
    ];

    let platform_ext = required_instance_os_extension_names();
    for ext in platform_ext {
        v.push(ext);
    }

    if enable_validation_layer {
        v.push(ash::ext::debug_utils::NAME.as_ptr())
    };

    v
}

pub const LAYER_KHRONOS_VALIDATION_NAME: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0") };
pub const LAYER_LUNAR_API_DUMP_NAME: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_LUNARG_api_dump\0") };

fn required_layer_names(enable_validation_layer: bool, enable_api_dump_layer: bool) -> Vec<*const i8> {

    let mut v: Vec<*const i8> = vec![];

    if enable_validation_layer {
        v.push(LAYER_KHRONOS_VALIDATION_NAME.as_ptr());
    }

    if enable_api_dump_layer {
        v.push(LAYER_LUNAR_API_DUMP_NAME.as_ptr());
    }

    v
}

pub struct Instance {
    pub obj: ash::Instance,
    pub debug_utils: ash::ext::debug_utils::Instance,
    pub debug_utils_messenger: vk::DebugUtilsMessengerEXT,
    pub debug_utils_enabled: bool,
    pub sdl: sdl2::Sdl
}

impl Disposable for Instance {
    fn dispose(&mut self) {
        trace!("Instance::dispose");

        let instance = crate::globals::instance();

        unsafe {
            if instance.debug_utils_enabled {
                instance.debug_utils_enabled = false;
                instance.debug_utils.destroy_debug_utils_messenger(instance.debug_utils_messenger, None);
            }
            instance.obj.destroy_instance(None);
        }
    }

}

impl Instance {

    pub fn new() -> Result<Self, Error> {
        trace!("create instance");

        let options = crate::globals::options();

        let entry = crate::globals::entry();

        let app_info = vk::ApplicationInfo {
            api_version: vk::make_api_version(0, 1, 3, 0),
            ..Default::default()
        };

        let validation_layer_enabled = options.enable_validation_layer; // Constants::ENABLE_VALIDATION_LAYER;
        let api_dump_layer_enabled = options.enable_api_dump_layer; // Constants::ENABLE_API_DUMP_LAYER;

        let required_instance_extensions = required_instance_extension_names(validation_layer_enabled);
        let required_layers = required_layer_names(validation_layer_enabled, api_dump_layer_enabled);

        let mut instance_create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&required_instance_extensions)
            .enabled_layer_names(&required_layers);

        let debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE |
                vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR)
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::GENERAL |
                vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION |
                vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE)
            .pfn_user_callback(Some(debug_callback));

        if validation_layer_enabled {

            let mut available_layers = unsafe { entry.enumerate_instance_layer_properties().unwrap() };

            for required_layer in &required_layers {

                let required_layer_name = unsafe { std::ffi::CStr::from_ptr(*required_layer) };
                let mut found = false;

                for available_layer in &mut available_layers {

                    let name = &available_layer.layer_name[..];
                    let available_layer_name = unsafe { std::ffi::CStr::from_ptr(name.as_ptr()) };

                    if required_layer_name == available_layer_name {
                        found = true;
                        break;
                    }
                }

                if !found {
                    return Err(Error::from("required validation layers not supported"));
                }

            }

            instance_create_info.p_next = &debug_create_info as *const _ as *const core::ffi::c_void;
        }

        trace!("create vulkan instance");
        let instance = unsafe {
            entry.create_instance(&instance_create_info, None).unwrap()
        };
        trace!("created vulkan instance");

        let debug_utils = ext::debug_utils::Instance::new(entry, &instance);

        let mut debug_utils_messenger = ash::vk::DebugUtilsMessengerEXT::null();

        if validation_layer_enabled {
            unsafe {
                debug_utils_messenger = debug_utils.create_debug_utils_messenger(&debug_create_info, None).unwrap();
            }
        }

        let sdl = sdl2::init().unwrap();

        Ok(Self {
            obj: instance,
            debug_utils,
            debug_utils_messenger,
            debug_utils_enabled: validation_layer_enabled,
            sdl
        })

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
