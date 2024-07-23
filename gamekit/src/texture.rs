//!
//! Texture
//!

use ash::vk::{self, Handle};

use crate::{api::{Disposable, LockRef}, bitmap::Bitmap, error::Error, image::{Image, ImageView}, manifest::StaticTextureDescriptor};

pub struct Texture {
    filename: String,
    image: Image,
    image_view: ImageView,
    pub width: u32,
    pub height: u32
}

pub type TextureRef = std::sync::Arc<Texture>;
pub type TextureLockRef = LockRef<Texture>;

impl Disposable for Texture {
    fn dispose(&mut self) {
        self.image_view.dispose();
        self.image.dispose();
    }
}

impl Texture {

    fn new(image: Image, filename: &str) -> Result<Self, Error> {

        let image_view = ImageView::new(&image);
        let width = image.width;
        let height = image.height;

        Ok(Self {
            filename: filename.to_string(),
            image,
            image_view,
            width,
            height
        })
    }

    pub fn from_resource(descriptor: &StaticTextureDescriptor) -> Result<Self, Error> {
        let image = Image::from_memory(descriptor.data, descriptor.format)?;
        Self::new(image,descriptor.name)
    }

    pub fn from_file(filename: &str) -> Result<Self, Error> {
        let image = Image::from_file(filename)?;
        Self::from_image(image)
    }

    pub fn from_image(image: Image) -> Result<Self, Error> {
        Self::new(image, "")
    }

    pub fn from_bitmap(bitmap: Bitmap) -> Result<Self, Error> {
        let image = Image::from_bitmap(bitmap)?;
        Self::from_image(image)
    }

    pub fn from_memory(data: &[u8], format: &str) -> Result<Self, Error> {
        let image = Image::from_memory(data, format)?;
        Self::from_image(image)
    }

    pub fn get_binding(texture_ref: &TextureLockRef, binding: u32, filtering: bool) -> TextureBinding {

        let t = texture_ref.clone();

        let texture = t.lock().unwrap();
        let image_view = &texture.image_view;
        let sampler = Sampler::new(filtering).unwrap();

        let descriptor = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(image_view.obj)
            .sampler(sampler.obj);

        TextureBinding {
            texture: texture_ref.clone(),
            sampler,
            descriptor,
            binding
        }
    }

}

pub struct TextureBinding {
    pub texture: TextureLockRef,
    sampler: Sampler,
    pub descriptor: vk::DescriptorImageInfo,
    binding: u32
}

impl Disposable for TextureBinding {
    fn dispose(&mut self) {
        self.sampler.dispose();
    }
}

impl TextureBinding {
    pub fn binding(&self) -> u32 {
        self.binding
    }
}

struct Sampler {
    obj: vk::Sampler
}

impl Disposable for Sampler {
    fn dispose(&mut self) {
        if self.obj.is_null() { return; }
        let device = crate::globals::device();
        unsafe { device.obj.destroy_sampler(self.obj, None) };
        self.obj = vk::Sampler::null();
    }
}

impl Sampler {

    pub fn new(filtering: bool) -> Result<Self, Error> {

        let instance = crate::globals::instance();
        let device = crate::globals::device();

        let properties = unsafe { instance.obj.get_physical_device_properties(device.physical_device) };

        let filter_mode = if filtering { vk::Filter::LINEAR } else { vk::Filter::NEAREST };

        let sampler_create_info = vk::SamplerCreateInfo::default()
            .mag_filter(filter_mode)
            .min_filter(filter_mode)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(true)
            .max_anisotropy(properties.limits.max_sampler_anisotropy)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR);

        let obj = unsafe { device.obj.create_sampler(&sampler_create_info, None).unwrap() };

        Ok(Self {
            obj
        })

    }

}
