//!
//! Renderer
//!

use ash::vk;

use crate::api::{Disposable, SpriteMeta};
use crate::blitter::Blitter;
use crate::error::Error;
use crate::font::{Font, FontLockRef};
use crate::material::{Material, MaterialLockRef};
use crate::math::Vec4;
use crate::sprite::{Sprite, SpriteData};

pub struct Renderer {
    valid: bool,
    material: MaterialLockRef,
    blitter: Blitter,
    pipeline_active: bool,
    pub viewport: vk::Viewport,
    pub scissor: vk::Rect2D,
    font: Font
}

impl Disposable for Renderer {
    fn dispose(&mut self) {
    }
}

impl Renderer {

    pub fn new(queue_size: usize) -> Result<Self, Error> {

        let default_material = Material::new();

        let viewport = vk::Viewport::default()
            .min_depth(0.0)
            .max_depth(1.0);

        let scissor = vk::Rect2D::default();
        let blitter = Blitter::new(queue_size);
        let font = Font::default();

        let mut renderer = Self {
            valid: false,
            material: Material::to_lockref(default_material),
            blitter,
            pipeline_active: false,
            viewport,
            scissor,
            font
        };

        renderer.reset_viewport();
        renderer.reset_scissor();

        Ok(renderer)
    }

    pub fn material(&self) -> &MaterialLockRef {
        &self.material
    }

    pub fn set_material(&mut self, material_ref: &MaterialLockRef) -> &mut Self {

        self.valid = true;

        self.material = material_ref.clone();
        if self.pipeline_active {
            let mut material_lock = self.material.lock().unwrap();
            self.font = material_lock.font().lock().unwrap().clone();
            material_lock.bind();
        }

        self
    }

    pub fn set_font(&mut self, font: &FontLockRef) {
        self.font = font.lock().unwrap().clone();
    }

    pub fn font(&self) -> &Font {
        &self.font
    }

    pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.viewport.x = x;
        self.viewport.y = y;
        self.viewport.width = w;
        self.viewport.height = h;

        if self.pipeline_active {
            unsafe {
                let pipeline = crate::globals::pipeline();
                let device = crate::globals::device();
                let frame = pipeline.current_frame();
                let command_buffer = &frame.command_buffer;
                device.obj.cmd_set_viewport(command_buffer.obj, 0, &[self.viewport]) ;
            }
        }
    }

    pub fn reset_viewport(&mut self) {
        let metrics = crate::globals::metrics();
        self.set_viewport(0.0, 0.0, metrics.window_width.max(0.0), metrics.window_height.max(0.0));
    }

    pub fn set_scissor(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.scissor.offset.x = x.max(0.0) as i32;
        self.scissor.offset.y = y.max(0.0) as i32;
        self.scissor.extent.width = w.max(0.0) as u32;
        self.scissor.extent.height = h.max(0.0) as u32;

        if self.pipeline_active {
            unsafe {
                let pipeline = crate::globals::pipeline();
                let device = crate::globals::device();
                let frame = pipeline.current_frame();
                let command_buffer = &frame.command_buffer;
                device.obj.cmd_set_scissor(command_buffer.obj, 0, &[self.scissor]);
            }
        }
    }

    pub fn reset_scissor(&mut self) {
        let metrics = crate::globals::metrics();
        self.set_scissor(metrics.view_x, metrics.view_y, metrics.view_width * metrics.view_scaling, metrics.view_height * metrics.view_scaling);
    }

    pub fn clear_scissor(&mut self) {
        let metrics = crate::globals::metrics();
        self.set_scissor(0.0, 0.0, metrics.window_width, metrics.window_height);
    }

    pub fn begin_frame(&mut self) -> Result<bool, Error> {

        if !self.valid {
            return Err(Error::from("renderer not initialized"));
        }

        let pipeline = crate::globals::pipeline_mut();
        let pipeline_reinitialized = pipeline.begin_frame()?;

        if pipeline_reinitialized {
            self.reset_viewport();
            self.reset_scissor();
        }

        unsafe {
            let device = crate::globals::device();
            let frame = pipeline.current_frame();
            let command_buffer = &frame.command_buffer;
            device.obj.cmd_set_viewport(command_buffer.obj, 0, &[self.viewport]) ;
            device.obj.cmd_set_scissor(command_buffer.obj, 0, &[self.scissor]);
        }

        self.pipeline_active = true;

        self.material.lock().unwrap().bind();

        Ok(pipeline_reinitialized)
    }

    pub fn end_frame(&mut self) -> Result<(), Error> {

        let pipeline = crate::globals::pipeline_mut();
        pipeline.end_frame()?;

        self.pipeline_active = false;

        Ok(())
    }

    pub fn begin(&mut self) {
        self.blitter.begin();
    }

    pub fn end(&mut self) {
        self.blitter.end();
    }

    pub fn clear(&mut self) {
        self.blitter.clear();
    }

    pub fn push_sprite(&mut self, data: &SpriteData) {
        self.blitter.push_sprite(data);
    }

    pub fn draw_sprite<T: Default + SpriteMeta>(&mut self, sprite: &mut Sprite<T>) {
        self.blitter.draw_sprite(sprite);
    }

    pub fn draw_char_by_index(&mut self, x: f32, y: f32, idx: u32) {
        self.blitter.draw_char_by_index(&self.font, x, y, idx);
    }

    pub fn draw_text(&mut self, x: f32, y: f32, text: &str) {
        self.blitter.draw_text(&self.font, x, y, text);
    }

    pub fn draw_text_rect(&mut self, rect: &Vec4, text: &str) {
        self.blitter.draw_text_rect(&self.font, rect, text);
    }

    pub fn draw_text_scaled(&mut self, x: f32, y: f32, scale_x: f32, scale_y: f32, text: &str) {
        self.blitter.draw_text_scaled(&self.font, x, y, scale_x, scale_y, text);
    }

    pub fn generate_sprite_sheet(&mut self, width: usize, height: usize, tile_width: usize, tile_height: usize) {
        self.blitter.generate_sprite_sheet(width, height, tile_width, tile_height);
    }

}
