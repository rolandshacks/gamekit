//!
//! Blitter
//!

use std::sync::{Arc, Mutex};

use crate::{api::{Disposable, LockRef, SpriteMeta}, constants::Constants, font::Font, math::Vec4, primitives::VertexQueue, sprite::{Sprite, SpriteData, SpriteSheet}};

pub struct Blitter {
    capacity: usize,
    usage: usize,
    vertex_queue: VertexQueue,
    sprite_sheet: SpriteSheet
}

pub type BlitterRef = std::sync::Arc<Blitter>;
pub type BlitterLockRef = LockRef<Blitter>;

impl Disposable for Blitter {
    fn dispose(&mut self) {
    }
}

impl Default for Blitter {
    fn default() -> Self {
        Blitter::new(Constants::DEFAULT_BLITTER_BATCH_CAPACITY)
    }
}

impl Blitter {
    pub fn new(capacity: usize) -> Self {
        let vertex_queue = VertexQueue::new(capacity);
        let sprite_sheet = SpriteSheet::default();

        Self {
            capacity,
            usage: 0,
            vertex_queue,
            sprite_sheet
        }
    }

    pub fn generate_sprite_sheet(&mut self, width: usize, height: usize, tile_width: usize, tile_height: usize) {
        self.sprite_sheet.alloc(width, height, tile_width, tile_height);
    }

    pub fn to_lockref(blitter: Self) -> BlitterLockRef {
        Arc::new(Mutex::new(blitter))
    }

    pub fn begin(&mut self) {
        self.vertex_queue.begin();
    }

    pub fn end(&mut self) {
        self.vertex_queue.end();
        self.vertex_queue.draw();
    }

    pub fn clear(&mut self) {
        self.vertex_queue.clear();
    }

    pub fn push_sprite(&mut self, data: &SpriteData) {

        let q = &mut self.vertex_queue;

        let position = &data.position;
        let pivot = &data.pivot;
        let size = &data.size;
        let color = &data.color;
        let texcoords = self.sprite_sheet.rect(data.frame as usize);

        q.push(
            position.x - pivot.x, position.y - pivot.x,
            size.x, size.y,
            color.r, color.g, color.b, color.a,
            texcoords.x, texcoords.y, texcoords.z, texcoords.w,
            0x0, 0x0
        )
    }

    pub fn draw_sprite<T: Default + SpriteMeta>(&mut self, sprite: &mut Sprite<T>) {
        let data = sprite.encode();
        self.push_sprite(data);
    }

    fn draw_char_by_index_impl(&mut self, font: &Font, x: f32, y: f32, w: f32, h: f32, idx: u32) {
        let q = &mut self.vertex_queue;

        let r = font.get_rect_by_idx(idx);

        q.push(
            x, y, w, h,
            1.0, 1.0, 1.0, 1.0,
            r.x, r.y, r.z, r.w,
            0x0, 0x0
        );
    }

    pub fn draw_char_by_index(&mut self, font: &Font, x: f32, y: f32, idx: u32) {
        self.draw_char_by_index_impl(font, x, y, font.char_width() as f32, font.char_height() as f32, idx);
    }

    fn draw_char_impl(&mut self, font: &Font, x: f32, y: f32, w: f32, h: f32, c: char) {
        let idx = match font.charset().find(c) {
            Some(idx) => idx,
            _ => 0
        };

        self.draw_char_by_index_impl(font, x, y, w, h, idx as u32);
    }

    pub fn draw_char(&mut self, font: &Font, x: f32, y: f32, c: char) {
        self.draw_char_impl(font, x, y, font.char_width() as f32, font.char_height() as f32, c);
    }

    pub fn draw_text(&mut self, font: &Font, x: f32, y: f32, text: &str) {

        if text.len() < 1 { return; }

        let mut xpos = x;
        let ypos = y;
        let w = font.char_width() as f32;
        let h = font.char_height() as f32;

        for c in text.chars() {
            self.draw_char_impl(font, xpos, ypos, w, h, c);
            xpos += w;
        }

    }

    pub fn draw_text_scaled(&mut self, font: &Font, x: f32, y: f32, scale_x: f32, scale_y: f32, text: &str) {

        if text.len() < 1 { return; }

        let mut xpos = x;
        let ypos = y;
        let w = scale_x * font.char_width() as f32;
        let h = scale_y * font.char_height() as f32;

        for c in text.chars() {
            self.draw_char_impl(font, xpos, ypos, w, h, c);
            xpos += w;
        }
    }

    pub fn draw_text_rect(&mut self, font: &Font, rect: &Vec4, text: &str) {

        if text.len() < 1 { return; }

        let mut xpos = rect.x;
        let ypos = rect.y;
        let w = rect.z / (text.len() as f32);
        let h = rect.w;

        for c in text.chars() {
            self.draw_char_impl(font, xpos, ypos, w, h, c);
            xpos += w;
        }

    }

}
