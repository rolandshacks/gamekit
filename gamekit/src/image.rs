//!
//! Image
//!

use ash::vk::{self, Handle};

use crate::api::{Disposable, LockRef};
use crate::bitmap::Bitmap;
use crate::buffer::{BufferObject, BufferType};
use crate::compiler::StaticBitmapDescriptor;
use crate::device::Device;
use crate::{error::Error, types::DeviceMemory};

pub struct Image {
    pub image_type: u32,
    pub width: u32,
    pub height: u32,
    pub channels: u32,
    pub size: usize,
    pub format: vk::Format,
    pub obj: vk::Image,
    pub memory: DeviceMemory
}

impl Disposable for Image {
    fn dispose(&mut self) {
        if !self.obj.is_null() {
            let device = crate::globals::device();
            unsafe { device.obj.destroy_image(self.obj, None); }
            self.obj = vk::Image::null();
        }

        self.memory.dispose();
        self.image_type = 0;
        self.width = 0;
        self.height = 0;
        self.channels = 0;
        self.size = 0;
    }
}

pub type ImageRef = std::sync::Arc<Image>;
pub type ImageLockRef = LockRef<Image>;

impl Image {

    pub const PIXEL_BUFFER: u32 = 0x1;
    pub const DEPTH_BUFFER: u32 = 0x2;

    pub fn new(image_type: u32, width: u32, height: u32, size: usize, format: vk::Format) -> Result<Self, Error> {
        Self::create(
            image_type,
            width,
            height,
            size,
            format
        )
    }

    pub fn detach(&mut self) {
        self.obj = vk::Image::null();
    }

    pub fn from_file(filename: &str) -> Result<Self, Error> {
        let bitmap = Bitmap::from_file(filename)?;
        Self::from_bitmap(bitmap)
    }

    pub fn from_resource(descriptor: &StaticBitmapDescriptor) -> Result<Self, Error> {
        let bitmap = Bitmap::from_resource(descriptor)?;
        Self::from_bitmap(bitmap)
    }

    pub fn from_memory(data: &[u8], format: &str) -> Result<Self, Error> {
        let bitmap = Bitmap::from_memory(data, format)?;
        Self::from_bitmap(bitmap)
    }

    pub fn from_bitmap(bitmap: Bitmap) -> Result<Self, Error> {

        let mut staging_buffer = BufferObject::new(
            BufferType::STAGING,
            bitmap.size(),
            vk::BufferUsageFlags::TRANSFER_SRC,
            DeviceMemory::HOST_VISIBLE | DeviceMemory::HOST_COHERENT
        );

        let pixels_ptr = bitmap.as_raw();

        staging_buffer.copy_raw(pixels_ptr)?;

        let image_format = match bitmap.bits_per_pixel() {
            8 =>  vk::Format::R8_UNORM,
            16 => vk::Format::R16_UNORM,
            24 => vk::Format::R8G8B8_SRGB,
            _ => vk::Format::R8G8B8A8_SRGB
        };

        let image = Self::create(Image::PIXEL_BUFFER, bitmap.width(), bitmap.height(), bitmap.size(), image_format)?;

        image.transition_image_layout(vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
        image.copy_buffer_to_image(staging_buffer.obj, bitmap.width(), bitmap.height())?;
        image.transition_image_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

        staging_buffer.dispose();

        Ok(image)
    }

    pub fn attach(image: vk::Image, image_type: u32, format: vk::Format) -> Result<Self, Error> {
        Ok(Self {
            image_type,
            width: 0,
            height: 0,
            channels: 0,
            size: 0,
            format: format,
            obj: image,
            memory: DeviceMemory::none()
        })
    }

    pub fn create(image_type: u32, width: u32, height: u32, size: usize, format: vk::Format) -> Result<Self, Error> {
        let device = crate::globals::device();

        let channels = 4u32;

        let usage_flags = match image_type {
            Self::DEPTH_BUFFER => vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            Self::PIXEL_BUFFER => vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            _ => { return Err(Error::from("unknown image_type")); }
        };

        let image_create_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D { width, height, depth: 1 } )
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(usage_flags)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1)
            .flags(vk::ImageCreateFlags::empty());

        let image = unsafe { device.obj.create_image(&image_create_info, None).unwrap() };

        let mem_requirements = unsafe { device.obj.get_image_memory_requirements(image) };

        let memory = DeviceMemory::new(mem_requirements, DeviceMemory::DEVICE_LOCAL)?;

        unsafe { device.obj.bind_image_memory(image, memory.as_handle(), 0).unwrap() };

        Ok(Self {
            image_type,
            width,
            height,
            channels,
            size,
            format,
            obj: image,
            memory
        })

    }

    pub fn copy_buffer_to_image(&self, buffer: vk::Buffer, width: u32, height: u32) -> Result<(), Error> {

        let device = crate::globals::device();

        let command_buffer = Device::begin_command();

        let copy_region = vk::BufferImageCopy::default()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .mip_level(0)
                    .base_array_layer(0)
                    .layer_count(1)
            )
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(vk::Extent3D { width, height, depth: 1 });

        let regions = [ copy_region ];

        unsafe { device.obj.cmd_copy_buffer_to_image(
            command_buffer,
            buffer,
            self.obj,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &regions) };

        Device::end_command(command_buffer);

        Ok(())
    }

    pub fn transition_image_layout(&self, old_layout: vk::ImageLayout, new_layout: vk::ImageLayout) {

        let src_access_mask;
        let dst_access_mask;
        let source_stage;
        let destination_stage;

        if old_layout == vk::ImageLayout::UNDEFINED && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL {
            src_access_mask = vk::AccessFlags::NONE;
            dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            source_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
            destination_stage = vk::PipelineStageFlags::TRANSFER;
        } else if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL {
            src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            dst_access_mask = vk::AccessFlags::SHADER_READ;
            source_stage = vk::PipelineStageFlags::TRANSFER;
            destination_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
        } else {
            panic!(
                "Unsupported layout transition({:?} => {:?}).",
                old_layout, new_layout
            )
        }

        let device = crate::globals::device();

        let command_buffer = Device::begin_command();

        let barrier = vk::ImageMemoryBarrier::default()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask)
            .image(self.obj)
            .subresource_range(vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1));

        let barriers = [ barrier ];

        unsafe {
            device.obj.cmd_pipeline_barrier(
                command_buffer,
                source_stage, destination_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &barriers
            );
        }

        Device::end_command(command_buffer);

    }

}

pub struct ImageView {
    pub obj: vk::ImageView,
    pub image: vk::Image,
    pub format: vk::Format
}

impl Disposable for ImageView {
    fn dispose(&mut self) {
        if self.obj.is_null() { return; }
        let device = crate::globals::device();
        unsafe { device.obj.destroy_image_view(self.obj, None); }
        self.obj = vk::ImageView::null();
        self.image = vk::Image::null();
    }
}

impl ImageView {
    pub fn new(image: &Image) -> Self {
        let device = crate::globals::device();
        Self::create_ex(&device.obj, image)
    }

    pub fn create_ex(device: &ash::Device, image: &Image) -> Self {

        let image_type = image.image_type;

        let aspect_mask = if image_type == Image::DEPTH_BUFFER { vk::ImageAspectFlags::DEPTH } else { vk::ImageAspectFlags::COLOR };

        let image_view_create_info = vk::ImageViewCreateInfo::default()
            .image(image.obj)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(image.format)
            .components(
                vk::ComponentMapping::default()
                    .r(vk::ComponentSwizzle::IDENTITY)
                    .g(vk::ComponentSwizzle::IDENTITY)
                    .b(vk::ComponentSwizzle::IDENTITY)
                    .a(vk::ComponentSwizzle::IDENTITY))
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(aspect_mask)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1));

        let image_view = unsafe { device.create_image_view(&image_view_create_info, None).unwrap() };

        Self {
            obj: image_view,
            image: image.obj,
            format: image.format
        }

    }

}
