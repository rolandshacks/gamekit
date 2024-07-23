//!
//! Sprite
//!

use std::sync::{Arc, Mutex};

use cgmath::Zero;

use crate::{api::{Disposable, LockRef, SpriteMeta}, math::{Vec2, Vec4}, primitives::Color};

pub struct SpriteData {
    pub position: Vec2,
    pub pivot: Vec2,
    pub size: Vec2,
    pub color: Color,
    pub frame: f32
}

impl Default for SpriteData {
    fn default() -> Self {
        Self {
            position: Vec2::zero(),
            pivot: Vec2::zero(),
            size: Vec2::zero(),
            color: Color::white(),
            frame: 0.0
        }
    }
}

impl SpriteData {

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position.x = x;
        self.position.y = y;
    }

    pub fn set_pivot(&mut self, x: f32, y: f32) {
        self.pivot.x = x;
        self.pivot.y = y;
    }

    pub fn set_size(&mut self, w: f32, h: f32) {
        self.size.x = w;
        self.size.y = h;
    }

    pub fn set_frame(&mut self, frame: f32) {
        self.frame = frame;
    }

    pub fn set_color(&mut self, color: &Color) {
        self.color.set(color);
    }

}


#[derive(Default)]
pub struct DefaultSpriteMeta {}

impl SpriteMeta for DefaultSpriteMeta {
    fn update(&mut self, _data: &mut SpriteData, _step: f32) {
    }
}

pub struct Sprite<T=DefaultSpriteMeta> {
    data: SpriteData,
    pub meta: T
}

impl <T: Default + SpriteMeta> Default for Sprite<T> {
    fn default() -> Self {
        Self {
            data: SpriteData::default(),
            meta: T::default()
        }
    }
}

impl <T: Default + SpriteMeta> Sprite<T> {
    pub fn update(&mut self, step: f32) {
        self.meta.update(&mut self.data, step);
    }

    pub fn data(&self) -> &SpriteData {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut SpriteData {
        self.meta.encode(&mut self.data);
        &mut self.data
    }

    pub fn encode(&mut self) -> &SpriteData {
        self.meta.encode(&mut self.data);
        &self.data
    }

    pub fn pivot(&self) -> &Vec2 {
        &self.data.pivot
    }

    pub fn position(&self) -> &Vec2 {
        &self.data.position
    }

    pub fn size(&self) -> &Vec2 {
        &self.data.size
    }

    pub fn frame(&self) -> f32 {
        self.data.frame
    }

    pub fn color(&self) -> &Color {
        &self.data.color
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.data.position.x = x;
        self.data.position.y = y;
    }

    pub fn set_pivot(&mut self, x: f32, y: f32) {
        self.data.pivot.x = x;
        self.data.pivot.y = y;
    }

    pub fn set_size(&mut self, w: f32, h: f32) {
        self.data.size.x = w;
        self.data.size.y = h;
    }

    pub fn set_frame(&mut self, frame: f32) {
        self.data.frame = frame;
    }

    pub fn set_color(&mut self, color: &Color) {
        self.data.color.set(color);
    }
}

pub struct SpriteSheet {
    coords: Vec<Vec4>
}

pub type SpriteSheetRef = std::sync::Arc<SpriteSheet>;
pub type SpriteSheetLockRef = LockRef<SpriteSheet>;

impl Disposable for SpriteSheet {
    fn dispose(&mut self) {
        self.coords.clear()
    }
}

impl Default for SpriteSheet {
    fn default() -> Self {
        Self {
            coords: vec!( Vec4::new(0.0, 0.0, 1.0, 1.0) )
        }
    }
}

impl SpriteSheet {

    pub fn new(width: usize, height: usize, tile_width: usize, tile_height: usize) -> Self {
        let mut sheet = Self { coords: Vec::new() };
        sheet.alloc(width, height, tile_width, tile_height);
        sheet
    }

    pub fn to_lockref(sprite_sheet: Self) -> SpriteSheetLockRef {
        Arc::new(Mutex::new(sprite_sheet))
    }

    pub fn alloc(&mut self, width: usize, height: usize, tile_width: usize, tile_height: usize) {
        self.coords.clear();

        let cols = width / tile_width;
        let rows = height / tile_height;

        let count = rows * cols;
        self.coords.reserve(count);

        let w = (tile_width as f32) / (width as f32);
        let h = (tile_height as f32) / (height as f32);

        for r in 0..rows {
            let y = (r as f32) / (rows as f32);
            for c in 0..cols {
                let x = (c as f32) / (cols as f32);
                self.coords.push(Vec4::new(x, y, w, h));
            }
        }
    }

    pub fn rect(&self, index: usize) -> &Vec4 {
        let i = if index >= self.coords.len() { 0 } else { index };
        return &self.coords[i];
    }

}
