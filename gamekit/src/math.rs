//!
//! Math types and helpers based on [cgmath].
//!
//! This is for convenience. Types can be replaced by the original
//! [cgmath] types.
//!
//! [cgmath]: https://github.com/rustgd/cgmath

use cgmath;

pub type Vec2 = cgmath::Vector2<f32>;
pub type Vec3 = cgmath::Vector3<f32>;
pub type Vec4 = cgmath::Vector4<f32>;

pub type Vec2i = cgmath::Vector2<i32>;
pub type Vec3i = cgmath::Vector3<i32>;
pub type Vec4i = cgmath::Vector4<i32>;

#[repr(C)]
#[derive(PartialEq, Eq, Copy, Clone, Hash)]
pub struct Rectangle<S> {
    pub pos: cgmath::Vector2<S>,
    pub size: cgmath::Vector2<S>
}

impl<S: Copy + std::ops::Add<Output=S> > Rectangle<S> {
    pub fn new(x: S, y: S, width: S, height: S) -> Self {
        Self {
            pos: cgmath::Vector2::new(x, y),
            size: cgmath::Vector2::new(width, height)
        }
    }

    pub fn set_pos(&mut self, x: S, y: S) -> &mut Self {
        self.pos.x = x;
        self.pos.y = y;
        self
    }

    pub fn set_size(&mut self, width: S, height: S) -> &mut Self {
        self.size.x = width;
        self.size.y = height;
        self
    }

    pub fn left_top(&self) -> cgmath::Vector2<S> {
        cgmath::Vector2::new(self.pos.x, self.pos.y)
    }

    pub fn right_top(&self) -> cgmath::Vector2<S> {
        cgmath::Vector2::new(self.pos.x + self.size.x, self.pos.y)
    }

    pub fn right_bottom(&self) -> cgmath::Vector2<S> {
        cgmath::Vector2::new(self.pos.x + self.size.x, self.pos.y + self.size.y)
    }

    pub fn left_bottom(&self) -> cgmath::Vector2<S> {
        cgmath::Vector2::new(self.pos.x, self.pos.y + self.size.y)
    }

}

pub type Rect = Rectangle<f32>;
pub type Recti = Rectangle<i32>;
