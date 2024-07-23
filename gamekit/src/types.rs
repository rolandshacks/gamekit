//!
//! Types
//!

use std::ops::BitOr;

use ash::vk::{self, Handle};

use crate::api::Disposable;
use crate::error::Error;
use crate::device::Device;

pub struct Surface {
    pub handle: u64,
    pub obj: vk::SurfaceKHR
}

impl Disposable for Surface {
    fn dispose(&mut self) {
        self.handle = 0;
        if !self.obj.is_null() {
            let window = crate::globals::window();
            let surface_instance = &window.surface_instance;
            unsafe { surface_instance.destroy_surface(self.obj, None); }
            self.obj = ash::vk::SurfaceKHR::null();
        }
    }
}

pub struct Semaphore {
    pub obj: vk::Semaphore
}

impl Semaphore {
    pub fn new() -> Self {
        let device = crate::globals::device();
        Self::new_raw(&device.obj)
    }

    fn new_raw(device: &ash::Device) -> Self {
        let semaphore_create_info = vk::SemaphoreCreateInfo::default();
        let semaphore = unsafe { device.create_semaphore(&semaphore_create_info, None).unwrap() };
        Self { obj: semaphore }
    }
}

impl Disposable for Semaphore {

    fn dispose(&mut self) {
        if self.obj.is_null() { return; }
        let device = crate::globals::device();
        unsafe { device.obj.destroy_semaphore(self.obj, None) }
        self.obj = vk::Semaphore::null();
    }
}

pub struct Fence {
    pub obj: vk::Fence
}

impl Disposable for Fence {
    fn dispose(&mut self) {
        if self.obj.is_null() { return; }
        let device = crate::globals::device();
        unsafe { device.obj.destroy_fence(self.obj, None) }
        self.obj = vk::Fence::null();
    }
}

impl Fence {
    pub fn new(signaled: bool) -> Self {
        let device = crate::globals::device();
        Self::new_raw(&device.obj, signaled)
    }

    fn new_raw(device: &ash::Device, signaled: bool) -> Self {
        let flags = if signaled { vk::FenceCreateFlags::SIGNALED } else { vk::FenceCreateFlags::empty() };
        let fence_create_info = vk::FenceCreateInfo::default()
            .flags(flags);
        let fence = unsafe { device.create_fence(&fence_create_info, None).unwrap() };
        Self { obj: fence }
    }

    pub fn wait(&self, timeout: u64) -> bool {
        let device = crate::globals::device();
        let fences = [ self.obj ];
        let result = unsafe { device.obj.wait_for_fences(&fences, true, timeout) };

        match result {
            Ok(_) => true,
            Err(_) => false
        }
    }

    pub fn wait_and_reset(&self, timeout: u64) -> bool {
        match self.wait(timeout) {
            true => { self.reset() },
            false => false
        }
    }

    pub fn reset(&self) -> bool {
        let device = crate::globals::device();
        let fences = [ self.obj ];
        let result = unsafe { device.obj.reset_fences(&fences) };
        match result {
            Ok(_) => true,
            Err(_) => false
        }
    }

}

pub struct DeviceMemory {
    pub size: usize,
    //pub type_index: u32,
    //pub flags: u32,
    pub obj: vk::DeviceMemory
}

impl Disposable for DeviceMemory {
    fn dispose(&mut self) {
        if self.obj.is_null() { return; }
        let device = crate::globals::device();
        unsafe { device.obj.free_memory(self.obj, None) };
        self.obj = vk::DeviceMemory::null();
    }
}

impl DeviceMemory {
    pub const DEVICE_LOCAL: u32 = 0x1;
    pub const HOST_COHERENT: u32 = 0x2;
    pub const HOST_VISIBLE: u32 = 0x4;

    pub fn none() -> Self {
        Self {
            size: 0,
            //type_index: 0,
            //flags: 0,
            obj: vk::DeviceMemory::null()
        }
    }

    pub fn new(requirements: vk::MemoryRequirements, flags: u32) -> Result<Self, Error> {

        let instance = crate::globals::instance();
        let device = crate::globals::device();

        let mut property_flags = vk::MemoryPropertyFlags::default();
        if 0x0 != (flags & Self::DEVICE_LOCAL) { property_flags = property_flags.bitor(vk::MemoryPropertyFlags::DEVICE_LOCAL); }
        if 0x0 != (flags & Self::HOST_COHERENT) { property_flags = property_flags.bitor(vk::MemoryPropertyFlags::HOST_COHERENT); }
        if 0x0 != (flags & Self::HOST_VISIBLE) { property_flags = property_flags.bitor(vk::MemoryPropertyFlags::HOST_VISIBLE); }

        let mem_properties = unsafe { instance.obj.get_physical_device_memory_properties(device.physical_device) };

        let mut type_index = 0u32;

        let mut found = false;

        for (i, memory_type) in mem_properties.memory_types.iter().enumerate() {
            if (requirements.memory_type_bits & (1 << i)) != 0x0 && memory_type.property_flags.contains(property_flags) {
                type_index = i as u32;
                found = true;
                break;
            }
        }

        if !found {
            return Err(Error::from("failed to find suitable memory type"));
        }

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size as vk::DeviceSize)
            .memory_type_index(type_index);

        let mem = unsafe { device.obj.allocate_memory(&alloc_info, None).unwrap() };

        Ok(Self {
            size: requirements.size as usize,
            //type_index,
            //flags,
            obj: mem
        })

    }

    pub fn map(&self, ofs: usize, len: usize) -> Result<*mut std::ffi::c_void, Error> {
        let device = crate::globals::device();
        let ptr = unsafe {
            match device.obj.map_memory(self.obj, ofs as vk::DeviceSize, len as vk::DeviceSize, vk::MemoryMapFlags::empty()) {
                Ok(ptr) => ptr,
                Err(_) => {
                    return Err(Error::from("map_memory failed"));
                }
            }
        };

        Ok(ptr)
    }

    pub fn unmap(&self) {
        let device = crate::globals::device();
        unsafe { device.obj.unmap_memory(self.obj) };
    }

    pub fn as_handle(&self) -> vk::DeviceMemory {
        return self.obj;
    }

    pub fn find_memory_type(requirements: vk::MemoryRequirements, mem_properties: vk::PhysicalDeviceMemoryProperties, required_properties: vk::MemoryPropertyFlags) -> u32 {
        for i in 0..mem_properties.memory_type_count {
            if requirements.memory_type_bits & (1 << i) != 0 &&
               mem_properties.memory_types[i as usize].property_flags.contains(required_properties) {
                return i;
            }
        }
        return 0; // not found
    }


}

pub struct Framebuffer {
    pub obj: vk::Framebuffer,
    pub render_pass: vk::RenderPass,
    pub image_view: vk::ImageView,
    pub width: u32,
    pub height: u32
}

impl Disposable for Framebuffer {
    fn dispose(&mut self) {
        if self.obj.is_null() { return; }
        let device = crate::globals::device();
        unsafe { device.obj.destroy_framebuffer(self.obj, None) };
        self.obj = vk::Framebuffer::null();
    }
}

impl Framebuffer {
    pub fn new (device: &ash::Device, render_pass: vk::RenderPass, image_view: vk::ImageView, depth_image_view: vk::ImageView, width: u32, height: u32) -> Result<Self, Error> {

        let attachments = [ image_view, depth_image_view ];

        let frame_buffer_create_info = vk::FramebufferCreateInfo::default()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(width)
            .height(height)
            .layers(1);

        let obj = unsafe { device.create_framebuffer(&frame_buffer_create_info, None).unwrap() };

        Ok(Self {
            obj,
            render_pass,
            image_view,
            width,
            height
        })
    }

}

pub struct CommandBuffer {
    pub obj: vk::CommandBuffer
}

impl Disposable for CommandBuffer {
    fn dispose(&mut self) {
        if self.obj.is_null() { return; }
        let device = crate::globals::device();
        let command_buffers = [self.obj];
        unsafe { device.obj.free_command_buffers(device.command_pool, &command_buffers) };
        self.obj = vk::CommandBuffer::null();
    }
}

impl CommandBuffer {

    pub fn new(device: &Device) -> Result<Self, Error> {

        let command_pool = device.command_pool;

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffers = unsafe { device.obj.allocate_command_buffers(&command_buffer_allocate_info).unwrap() };
        let command_buffer = command_buffers[0];

        Ok(Self {
            obj: command_buffer
        })
    }

    pub fn reset(&self) {
        let device = crate::globals::device();
        let _ = unsafe { device.obj.reset_command_buffer(self.obj, vk::CommandBufferResetFlags::empty()) };
    }

    pub fn begin(&self) {
        let device = crate::globals::device();
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::default();
        let _ = unsafe { device.obj.begin_command_buffer(self.obj, &command_buffer_begin_info) };
    }

    pub fn end(&self) {
        let device = crate::globals::device();
        let _ = unsafe { device.obj.end_command_buffer(self.obj) };
    }


}

pub struct Frame {
    pub index: u32,
    pub command_buffer: CommandBuffer,
    pub image_available: Semaphore,
    pub render_finished: Semaphore,
    pub command_buffers_completed: Fence
}

impl Disposable for Frame {
    fn dispose(&mut self) {
        self.index = 0;
        self.command_buffer.dispose();
        self.image_available.dispose();
        self.render_finished.dispose();
        self.command_buffers_completed.dispose();
    }
}

impl Frame {
    pub fn new(device: &Device, index: u32) -> Result<Frame, Error> {

        let command_buffer = CommandBuffer::new(device)?;

        Ok(Self {
            index,
            command_buffer,
            image_available: Semaphore::new_raw(&device.obj),
            render_finished: Semaphore::new_raw(&device.obj),
            command_buffers_completed: Fence::new_raw(&device.obj, true)
        })
    }

}
