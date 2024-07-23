//!
//! Buffer
//!

use ash::vk::{self, Handle};

use crate::api::{Disposable, LockRef};
use crate::error::Error;
use crate::types::{DeviceMemory, Frame};

pub type IndexBufferElementType = u32;

const INDEX_DATA_TYPE: vk::IndexType = vk::IndexType::UINT32;

/// Buffer Type
pub struct BufferType {}

impl BufferType {
    pub const UNKNOWN: u32 = 0x0;
    pub const VERTEX: u32 = 0x1;
    pub const INDEX: u32 = 0x2;
    pub const UNIFORM: u32 = 0x3;
    pub const SHADER_STORAGE: u32 = 0x4;
    pub const STAGING: u32 = 0x5;
    pub const DYNAMIC_UNIFORM: u32 = 0x6;
}

/// Buffer Object
pub struct BufferObject {
    pub buffer_type: u32,
    pub size: usize,
    pub obj: vk::Buffer,
    pub memory: DeviceMemory
}

type BufferObjectRef = std::sync::Arc<BufferObject>;

impl Disposable for BufferObject {
    fn dispose(&mut self) {
        if self.obj.is_null() { return; }
        let device = crate::globals::device();
        unsafe { device.obj.destroy_buffer(self.obj, None) };
        self.obj = vk::Buffer::null();
        self.memory.dispose();
        self.size = 0;
    }
}

impl BufferObject {

    pub fn new(buffer_type: u32, size: usize, buffer_usage: vk::BufferUsageFlags, memory_usage: u32) -> Self {

        let device = crate::globals::device();

        let buffer_create_info = vk::BufferCreateInfo::default()
            .size(size as vk::DeviceSize)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .usage(buffer_usage);

        let buffer = unsafe { device.obj.create_buffer(&buffer_create_info, None).unwrap() };

        let memory = if memory_usage != 0x0 {
            let mem_requirements = unsafe { device.obj.get_buffer_memory_requirements( buffer ) };
            let memory = DeviceMemory::new(mem_requirements, memory_usage).unwrap();

            unsafe { let _ = device.obj.bind_buffer_memory(buffer, memory.obj, 0); }

            memory

        } else {
            DeviceMemory::none()
        };

        Self {
            buffer_type,
            size,
            obj: buffer,
            memory
        }
    }

    pub fn bind(&self, frame: &Frame) -> Result<(), Error> {

        let device = crate::globals::device();
        let command_buffer = &frame.command_buffer;

        match self.buffer_type {
            BufferType::VERTEX => {
                let offsets = [0u64];
                let buffers = [self.obj];
                let _ = unsafe { device.obj.cmd_bind_vertex_buffers(
                    command_buffer.obj, 0, &buffers, &offsets
                )};
            },
            BufferType::INDEX => {
                let _ = unsafe { device.obj.cmd_bind_index_buffer(
                    // index type - from primitives::VertexIndexType
                    command_buffer.obj, self.obj, 0, INDEX_DATA_TYPE
                )};
            },
            _ => {}
        }

        Ok(())

    }

    pub fn map(&self) -> Result<*mut std::ffi::c_void, Error> {
        self.memory.map(0, self.size)
    }

    pub fn map_region(&self, ofs: usize, len: usize) -> Result<*mut std::ffi::c_void, Error> {
        self.memory.map(ofs, len)
    }

    pub fn unmap(&self) -> Result<(), Error> {
        self.memory.unmap();
        Ok(())
    }

    pub fn copy_raw(&self, source_ptr: *const std::ffi::c_void) -> Result<(), Error> {
        self.copy_region_raw(source_ptr, 0, 0, self.size)
    }

    pub fn copy_region_raw(&self, source_ptr: *const std::ffi::c_void, source_ofs: usize, dest_ofs: usize, len: usize) -> Result<(), Error> {
        let ofs_ptr = unsafe { source_ptr.offset(source_ofs as isize) };
        let dest_ptr = self.map_region(dest_ofs, len)?;
        unsafe { std::ptr::copy_nonoverlapping(ofs_ptr, dest_ptr, len); }
        self.unmap()
    }

    pub fn copy(&self, src: &Self) -> Result<(), Error> {
        self.copy_region(src, 0, 0, src.size)
    }

    pub fn copy_region(&self, src: &Self, src_ofs: usize, dest_ofs: usize, len: usize) -> Result<(), Error> {

        let src_buffer = src.obj;
        let dest_buffer = self.obj;

        let device = crate::globals::device();

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(device.command_pool)
            .command_buffer_count(1);

        let command_buffers = unsafe {
            device.obj.allocate_command_buffers(&command_buffer_allocate_info).unwrap()
        };

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        let copy_regions = [
            vk::BufferCopy::default()
                .src_offset(src_ofs as vk::DeviceSize)
                .dst_offset(dest_ofs as vk::DeviceSize)
                .size(len as vk::DeviceSize)
        ];

        let submit_infos = [
            vk::SubmitInfo::default()
                .command_buffers(&command_buffers)
        ];

        unsafe {
            let command_buffer = command_buffers[0];

            let _ = device.obj.begin_command_buffer(command_buffer, &command_buffer_begin_info);

            device.obj.cmd_copy_buffer(command_buffer, src_buffer, dest_buffer, &copy_regions);

            let _ = device.obj.end_command_buffer(command_buffer);
            let _ = device.obj.queue_submit(device.graphics_queue, &submit_infos, vk::Fence::null());
            let _ = device.obj.queue_wait_idle(device.graphics_queue);

            device.obj.free_command_buffers(device.command_pool, &command_buffers);
        }

        Ok(())

    }


}


struct BufferObjects {
    buffer_objects: Vec<BufferObject>,
    size: usize,
    buffer_type: u32,
    binding: u32
}

impl Disposable for BufferObjects {
    fn dispose(&mut self) {
        for buffer_object in &mut self.buffer_objects {
            buffer_object.dispose();
        }
        self.buffer_objects.clear();
        self.size = 0;
        self.buffer_type = 0;
        self.binding = 0;
    }
}

impl BufferObjects {

    pub fn new(binding: u32, buffer_type: u32, size: usize) -> Self {

        let mut buffer_objects = Self {
            buffer_objects: Vec::new(),
            size,
            buffer_type,
            binding
        };

        match buffer_type {
            BufferType::VERTEX => {
                buffer_objects.add_vertex_buffer();
                buffer_objects.add_staging_buffer();
            },
            BufferType::INDEX => {
                buffer_objects.add_index_buffer();
                buffer_objects.add_staging_buffer();
            },
            BufferType::UNIFORM | BufferType::DYNAMIC_UNIFORM => {

                let pipeline = crate::globals::pipeline();

                // create buffer object per frame
                let num_frames = pipeline.frame_count();
                for _ in 0..num_frames {
                    buffer_objects.add_uniform_buffer();
                }
            }
            _ => {}
        }

        buffer_objects
    }

    pub fn add(&mut self, buffer_object: BufferObject) {
        self.buffer_objects.push(buffer_object);
    }

    pub fn get(&self, index: usize) -> &BufferObject {
        &self.buffer_objects[index]
    }

    pub fn add_vertex_buffer(&mut self) -> &mut Self {
        self.add(BufferObject::new(
            self.buffer_type,
            self.size,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            DeviceMemory::DEVICE_LOCAL));

        self
    }

    pub fn add_staging_buffer(&mut self) -> &mut Self {
        self.add(BufferObject::new(
            BufferType::STAGING,
            self.size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            DeviceMemory::HOST_VISIBLE | DeviceMemory::HOST_COHERENT));

        self
    }

    pub fn add_index_buffer(&mut self) -> &mut Self {
        self.add(BufferObject::new(
            self.buffer_type,
            self.size,
            vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            DeviceMemory::DEVICE_LOCAL));

        self
    }

    pub fn add_uniform_buffer(&mut self) -> &mut Self {

        self.add(BufferObject::new(
            self.buffer_type,
            self.size,
            vk::BufferUsageFlags::UNIFORM_BUFFER | vk::BufferUsageFlags::TRANSFER_SRC,
            DeviceMemory::HOST_VISIBLE | DeviceMemory::HOST_COHERENT));

        self
    }

    pub fn add_shader_storage_buffer(&mut self) -> &mut Self {

        self.add(BufferObject::new(
            self.buffer_type,
            self.size,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_SRC,
            DeviceMemory::HOST_VISIBLE | DeviceMemory::HOST_COHERENT));

        self
    }

}
pub struct VertexBuffer {
    buffer_objects: BufferObjects
}

impl Disposable for VertexBuffer {
    fn dispose(&mut self) {
        self.buffer_objects.dispose();
    }
}

impl VertexBuffer {
    pub fn new(size: usize) -> Self {

        let buffer_objects = BufferObjects::new(
            0,
            BufferType::VERTEX,
            size
        );

        Self {
            buffer_objects
        }
    }

    pub fn copy(&self, source_ptr: *const std::ffi::c_void) -> Result<(), Error> {
        self.copy_region(source_ptr, 0, self.buffer_objects.size)
    }

    pub fn copy_region(&self, source_ptr: *const std::ffi::c_void, ofs: usize, size: usize) -> Result<(), Error> {
        self.buffer_objects.buffer_objects[1].copy_region_raw(source_ptr, ofs, ofs, size)?;
        self.buffer_objects.buffer_objects[0].copy_region(&self.buffer_objects.buffer_objects[1], ofs, ofs, size)?;
        Ok(())
    }

    pub fn bind(&self, frame: &Frame) -> Result<(), Error> {
        self.buffer_objects.buffer_objects[0].bind(frame)
    }

}


pub struct IndexBuffer {
    buffer_objects: BufferObjects
}

impl Disposable for IndexBuffer {
    fn dispose(&mut self) {
        self.buffer_objects.dispose();
    }
}

impl IndexBuffer {
    pub fn new(size: usize) -> Self {

        let buffer_objects = BufferObjects::new(
            0,
            BufferType::INDEX,
            size
        );

        Self {
            buffer_objects
        }
    }

    pub fn copy(&self, source_ptr: *const std::ffi::c_void) -> Result<(), Error> {
        self.copy_region(source_ptr, 0, self.buffer_objects.size)
    }

    pub fn copy_region(&self, source_ptr: *const std::ffi::c_void, ofs: usize, size: usize) -> Result<(), Error> {
        self.buffer_objects.buffer_objects[1].copy_region_raw(source_ptr, ofs, ofs, size)?;
        self.buffer_objects.buffer_objects[0].copy_region(&self.buffer_objects.buffer_objects[1], ofs, ofs, size)?;
        Ok(())
    }

    pub fn bind(&self, frame: &Frame) -> Result<(), Error> {
        self.buffer_objects.buffer_objects[0].bind(frame)
    }

}

pub struct ShaderStorageBuffer {
    buffer_objects: BufferObjects
}

impl Disposable for ShaderStorageBuffer {
    fn dispose(&mut self) {
        self.buffer_objects.dispose();
    }
}

impl ShaderStorageBuffer {
    pub fn new(size: usize) -> Self {

        let buffer_objects = BufferObjects::new(
            0,
            BufferType::SHADER_STORAGE,
            size
        );

        Self {
            buffer_objects
        }
    }

    pub fn alloc_frame_buffer(&mut self) {
        self.buffer_objects.add_shader_storage_buffer();
    }

    pub fn copy(&self, frame: &Frame, source_ptr: *const std::ffi::c_void) -> Result<(), Error> {
        let buffer_object = &self.buffer_objects.buffer_objects[frame.index as usize];
        buffer_object.copy_raw(source_ptr)
    }

    pub fn bind(&self, frame: &Frame) -> Result<(), Error> {
        let buffer_object = &self.buffer_objects.buffer_objects[frame.index as usize];
        buffer_object.bind(frame)
    }


}

pub struct PushConstants<T> {
    data: T,
    data_size: usize

    //pub raw_ptr: *mut std::ffi::c_void,
    //pub material: Option<MaterialRef>
}

pub type PushConstantsRef<T> = std::sync::Arc<PushConstants<T>>;

impl <T: Default> Disposable for PushConstants<T> {
    fn dispose(&mut self) {
        self.data_size = 0;
        //self.raw_ptr = std::ptr::null_mut();
    }
}

impl <T: Default> PushConstants<T> {
    pub fn new() -> Result<Self, Error> {

        let data = T::default();
        let data_size = core::mem::size_of::<T>();

        Ok(Self {
            data,
            data_size
        })
    }

    pub fn size(&self) -> usize {
        self.data_size
    }

    pub fn update(&self) -> Result<(), Error> {
        let pipeline = crate::globals::pipeline();
        let frame = pipeline.current_frame();
        self.update_frame(frame)
    }

    pub fn update_all(&self) -> Result<(), Error> {
        let pipeline = crate::globals::pipeline();

        for frame in &pipeline.frames {
            self.update_frame(frame)?
        }

        Ok(())
    }

    fn update_frame(&self, frame: &Frame) -> Result<(), Error> {

        let data_ptr = unsafe { std::slice::from_raw_parts(
            &self.data as *const T as *const u8,
            std::mem::size_of::<T>()
        ) };

        let material_ref = crate::globals::renderer().material();
        let material = material_ref.lock().unwrap();
        let pipeline_layout = material.pipeline_layout;

        let device = crate::globals::device();

        //let frame = pipeline.current_frame();
        let command_buffer = frame.command_buffer.obj;

        unsafe {
            device.obj.cmd_push_constants(
                command_buffer,
                pipeline_layout,
                vk::ShaderStageFlags::ALL_GRAPHICS,
                0,
                &data_ptr
            );
        }

        Ok(())
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }


}

pub struct UniformBuffer {
    buffer_objects: BufferObjects,
    pub dynamic: bool,
    pub offset: usize,
    pub size: usize,
    pub dynamic_size: usize
}

pub type UniformBufferRef = std::sync::Arc<UniformBuffer>;
pub type UniformBufferLockRef = LockRef<UniformBuffer>;

impl Disposable for UniformBuffer {
    fn dispose(&mut self) {
        self.buffer_objects.dispose();
        self.dynamic = false;
        self.offset = 0;
        self.size = 0;
        self.dynamic_size = 0;
    }
}

impl UniformBuffer {
    pub fn new(index: u32, size: usize, dynamic_size: usize) -> Self {

        let dynamic = dynamic_size > 0;
        let buffer_size = size.max(dynamic_size);
        let buffer_type = if dynamic { BufferType::DYNAMIC_UNIFORM } else { BufferType::UNIFORM };

        let buffer_objects = BufferObjects::new(
            index,
            buffer_type,
            buffer_size
        );

        Self {
            buffer_objects,
            dynamic,
            offset: 0,
            size,
            dynamic_size
        }
    }

    pub fn binding(&self) -> u32 {
        self.buffer_objects.binding
    }

    pub fn is_dynamic(&self) ->  bool {
        self.dynamic
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn set_offset(&mut self, offset: usize) -> &mut Self {
        self.offset = offset;
        self
    }

    pub fn copy(&self, frame: &Frame, source_ptr: *const std::ffi::c_void) -> Result<(), Error> {
        self.copy_region(frame, source_ptr, 0, self.offset, self.size)
    }

    pub fn copy_region(&self, frame: &Frame, source_ptr: *const std::ffi::c_void, src_ofs: usize, dest_ofs: usize, len: usize) -> Result<(), Error> {
        let buffer_object = &self.buffer_objects.buffer_objects[frame.index as usize];
        buffer_object.copy_region_raw(source_ptr, src_ofs, dest_ofs, len)
    }

    pub fn bind(&self, frame: &Frame) -> Result<(), Error> {
        let buffer_object = &self.buffer_objects.buffer_objects[frame.index as usize];
        buffer_object.bind(frame)
    }

    pub fn get_buffer_info(&self, frame_index: usize) -> vk::DescriptorBufferInfo {
        let buffer_object = self.buffer_objects.get(frame_index);

        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(buffer_object.obj)
            .offset(self.offset as u64) // or self.offset as u64 (??)
            .range(self.size as u64);

        buffer_info
    }

}

pub struct Uniform<T> {
    data: T,
    buffer_ref: UniformBufferLockRef,
    offset: usize,
    data_size: usize,
    aligned_data_size: usize,
    dynamic_size: usize
}

pub type UniformRef<T> = std::sync::Arc<Uniform<T>>;

impl <T: Default> Disposable for Uniform<T> {
    fn dispose(&mut self) {
        self.buffer_ref.lock().unwrap().dispose();
        self.offset = 0;
        self.data_size = 0;
        self.aligned_data_size = 0;
        self.dynamic_size = 0;
    }
}

impl <T: Default> Uniform<T> {

    pub fn new(index: u32, dynamic_array_elements: usize) -> Result<Self, Error> {

        let device = crate::globals::device();
        let limits = &device.limits;

        let data = T::default();
        let data_size = core::mem::size_of::<T>();

        let alignment = limits.uniform_buffer_alignment.max(4);
        let aligned_data_size = (data_size + alignment)  - (data_size % alignment);

        let dynamic_size = if dynamic_array_elements > 0 { aligned_data_size * (1 + dynamic_array_elements) } else { 0 };

        let buffer = UniformBuffer::new(index, data_size, dynamic_size);
        let buffer_ref = std::sync::Arc::new(std::sync::Mutex::new(buffer));

        Ok(Self {
            data,
            buffer_ref,
            offset: 0,
            data_size,
            aligned_data_size,
            dynamic_size
        })
    }

    pub fn get_buffer_ref(&self) -> UniformBufferLockRef {
        self.buffer_ref.clone()
    }

    fn copy(&self, frame: &Frame) -> Result<(), Error> {
        let data_ptr: *const T = &self.data;
        let raw_ptr = data_ptr as *const std::ffi::c_void;
        self.buffer_ref.lock().unwrap().copy(frame, raw_ptr)?;

        Ok(())
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn set_array_index(&mut self, idx: usize) -> usize {
        let old_idx = self.offset / self.aligned_data_size;
        self.set_offset(idx * self.aligned_data_size);
        old_idx
    }

    pub fn set_offset(&mut self, ofs: usize) {
        self.offset = ofs;
        self.buffer_ref.lock().unwrap().set_offset(ofs);
    }

    pub fn update(&self) -> Result<(), Error> {
        let pipeline = crate::globals::pipeline();
        let frame = pipeline.current_frame();
        self.copy(frame)
    }

    pub fn update_all(&self) -> Result<(), Error> {
        let pipeline = crate::globals::pipeline();
        //let frame = pipeline.current_frame();
        for frame in &pipeline.frames {
            self.copy(frame)?;
        }
        Ok(())
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

}
