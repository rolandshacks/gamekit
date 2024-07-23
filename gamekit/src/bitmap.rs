//!
//! Bitmap
//!

use std::fs::File;
use std::io::{Cursor, Read};

use crate::api::Disposable;
use crate::error::Error;
use crate::manifest::StaticBitmapDescriptor;

pub struct Bitmap {
    width: u32,
    height: u32,
    bits_per_pixel: u32,
    bytes_per_line: u32,
    size: usize,
    pixels: Vec<u8>
}

pub type BitmapRef = std::sync::Arc<Bitmap>;
pub type BitmapLockRef = std::sync::Arc<std::sync::Mutex<Bitmap>>;

impl Default for Bitmap {
    fn default() -> Self {
        Self::new(0, 0, 0, 0)
    }
}

impl Disposable for Bitmap {
    fn dispose(&mut self) {
        self.width = 0;
        self.height = 0;
        self.bits_per_pixel = 0;
        self.bytes_per_line = 0;
        self.size = 0;
        self.pixels.clear();
    }
}

impl Bitmap {
    pub const FORMAT_DEFAULT: u32 = 0x0;
    pub const FORMAT_CHARSET: u32 = 0x1;

    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
    pub fn bits_per_pixel(&self) -> u32 { self.bits_per_pixel }
    pub fn bytes_per_line(&self) -> u32 { self.bytes_per_line }
    pub fn size(&self) -> usize { self.size }
    pub fn pixels(&self) -> &Vec<u8> { &self.pixels }
    pub fn pixels_mut(&mut self) -> &mut Vec<u8> { &mut self.pixels }
    pub fn as_raw(&self) -> *const std::ffi::c_void { self.pixels.as_ptr() as *const std::ffi::c_void }

    pub fn new(width: u32, height: u32, bits_per_pixel: u32, bytes_per_line: u32) -> Self {

        let bpl = if bytes_per_line > 0 { bytes_per_line } else { (width * bits_per_pixel) / 8 };
        let size = (height * bpl) as usize;
        let pixels = Vec::new();

        Self {
            width,
            height,
            bits_per_pixel,
            bytes_per_line: bpl,
            size,
            pixels
        }
    }

    pub fn alloc(width: u32, height: u32, bits_per_pixel: u32, bytes_per_line: u32) -> Self {
        let mut bitmap = Self::new(width, height, bits_per_pixel, bytes_per_line);
        bitmap.pixels = vec![0; bitmap.size];
        bitmap
    }

    pub fn from_file(filename: &str) -> Result<Self, Error> {

        let mut file = File::open(filename).unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        let cursor = Cursor::new(buf);
        let img_obj = image::load(cursor, image::ImageFormat::Png).unwrap();

        Self::from_image_obj(img_obj)
    }

    pub fn from_resource(descriptor: &StaticBitmapDescriptor) -> Result<Self, Error> {
        Self::from_memory(descriptor.data, descriptor.format)
    }

    pub fn from_memory(data: &[u8], format: &str) -> Result<Self, Error> {
        if format == "charmem" {
            Self::from_charmem(data)
        } else {
            Self::from_image_memory(data)
        }
    }

    pub fn from_image_memory(data: &[u8]) -> Result<Self, Error> {
        let data_ptr = data.as_ptr() as *const std::ffi::c_uchar;
        let data_size = data.len();
        let pixels = unsafe { core::slice::from_raw_parts::<u8>(data_ptr, data_size) };
        let img_obj = image::load_from_memory(pixels as &[u8]).unwrap();
        Self::from_image_obj(img_obj)
    }

    fn from_image_obj(img: image::DynamicImage) -> Result<Self, Error> {
        let width = img.width();
        let height = img.height();
        let bits_per_pixel = match img.color() {
            image::ColorType::L8 | image::ColorType::La8 => 8,
            image::ColorType::L16 | image::ColorType::La16 => 16,
            image::ColorType::Rgb16 => 16,
            image::ColorType::Rgb8 => 24,
            image::ColorType::Rgba8 => 32,
            _ => 32
        };

        let image_buffer = img.to_rgba8();
        let pixels = image_buffer.into_raw();
        let image = Self::from_data(width, height, bits_per_pixel, pixels)?;

        Ok(image)
    }

    pub fn from_data(width: u32, height: u32, bits_per_pixel: u32, pixels: Vec<u8>) -> Result<Self, Error> {
        let size = pixels.len();
        let bytes_per_line = (size as u32) / height;

        Ok(Self {
            width,
            height,
            bits_per_pixel,
            bytes_per_line,
            size,
            pixels
        })
    }

    pub fn from_charmem(data: &[u8]) -> Result<Self, Error> {

        // Decode commodore character set format
        // 2048 bytes which contain 256 characters with 8x8 monochrome pixels
        // optionally, there can be a 2 byte header which indicates the load address

        let data_size = data.len();
        if data_size < 8 {
            return Err(Error::from("invalid character set data"));
        }

        // first 2 bytes might be load address for target machine 
        let data_offset = if data_size % 8 == 2 { 0x2usize } else { 0x0usize };

        let char_count = data_size / 8;
        let char_width = 8;
        let char_height = 8;

        let bits_per_pixel = 32;

        let mut bitmap = Bitmap::alloc((char_count as u32) * char_width, char_height, bits_per_pixel, 0);
        let bytes_per_line = bitmap.bytes_per_line() as usize;
        let pixels = bitmap.pixels_mut();

        let mut src_ofs = data_offset;

        if bits_per_pixel == 8 {
            for i in 0..char_count {
                let mut line_ofs = i * ((char_width * bits_per_pixel) / 8) as usize;
                for _ in 0..char_height {
                    let src = data[src_ofs];
                    let mut dest_ofs = line_ofs;
                    for x in 0..char_width {
                        let bit = (src & (1 << (char_width-x-1))) != 0x0;
                        let color_value = if bit {0xff} else {0x0};
                        pixels[dest_ofs] = color_value;
                        dest_ofs += 1;
                    }
                    src_ofs += 1;
                    line_ofs += bytes_per_line;
                }
            }
        } else {
            for i in 0..char_count {
                let mut line_ofs = i * ((char_width * bits_per_pixel) / 8) as usize;
                for _ in 0..char_height {
                    let src = data[src_ofs];
                    let mut dest_ofs = line_ofs;
                    for x in 0..char_width {
                        let bit = (src & (1 << (char_width-x-1))) != 0x0;
                        let color_value = if bit {0xff} else {0x0};
                        pixels[dest_ofs] = color_value; dest_ofs += 1;
                        pixels[dest_ofs] = color_value; dest_ofs += 1;
                        pixels[dest_ofs] = color_value; dest_ofs += 1;
                        pixels[dest_ofs] = color_value; dest_ofs += 1;
                    }
                    src_ofs += 1;
                    line_ofs += bytes_per_line;
                }
            }
        }

        Ok(bitmap)
    }

}
