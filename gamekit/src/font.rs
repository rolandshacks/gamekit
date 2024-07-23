//!
//! Font
//!

use std::sync::{Arc, Mutex};

use gamebuilder::manifest::StaticFontDescriptor;

use crate::{api::{Disposable, LockRef}, error::Error, math::{Vec2, Vec4}, texture::TextureLockRef};

#[derive(Default, Clone, Debug)]
pub struct Font {
    charset: &'static str,
    char_width: u32,
    char_height: u32,
    texture_width: u32
}

pub type FontRef = std::sync::Arc<Font>;
pub type FontLockRef = LockRef<Font>;

impl Disposable for Font {
    fn dispose(&mut self) {
    }
}

impl Font {
    pub fn new(charset: &'static str, char_width: u32, char_height: u32, texture: &TextureLockRef) -> Result<Self, Error> {

        let texture_width = texture.lock().unwrap().width;

        Ok(Self {
            charset,
            char_width,
            char_height,
            texture_width
        })
    }

    pub fn to_lockref(font: Self) -> FontLockRef {
        Arc::new(Mutex::new(font))
    }

    pub fn from_resource(descriptor: &StaticFontDescriptor) -> Result<Self, Error> {
        let resources = crate::globals::resources();
        let texture = resources.get_texture(descriptor.texture);
        Self::new(descriptor.charset, descriptor.char_width, descriptor.char_height, &texture)
    }

    pub fn char_width(&self) -> u32 {
        self.char_width
    }

    pub fn char_height(&self) -> u32 {
        self.char_height
    }

    pub fn charset(&self) -> &str {
        self.charset
    }

    pub fn size(&self) -> usize {
        (self.texture_width / self.char_width) as usize
    }

    pub fn get_rect_by_idx(&self, idx: u32) -> Vec4 {
        let w = (self.char_width as f32) / (self.texture_width as f32);
        let h = 1.0f32;
        let x = w * (idx as f32);

        Vec4::new(
            x, 0.0, w, h
        )
    }

    pub fn get_rect(&self, c: char) -> Vec4 {
        let idx = self.charset.find(c).unwrap_or_default();
        self.get_rect_by_idx(idx as u32)
    }

    pub fn get_text_extent(&self, text: &str) -> Vec2 {
        Vec2::new((text.len() * self.char_width as usize) as f32, self.char_height as f32)
    }

}
