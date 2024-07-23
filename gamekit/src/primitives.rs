//!
//! Primitives
//!

use std::mem::offset_of;

use ash::vk;
use cgmath::Zero;

use crate::{api::Disposable, buffer::{IndexBuffer, IndexBufferElementType, VertexBuffer}, math::{Vec2, Vec3, Vec4}};

const DEFAULT_COLOR: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);
const DEFAULT_TEXTURE_COORDS: Vec4 = Vec4::new(0.0, 0.0, 1.0, 1.0);
const DEFAULT_TEXTURE_MASK: u32 = 0x1;
const DEFAULT_FLAGS: u32 = 0x0;
const DEFAULT_QUAD_INDICES: [IndexBufferElementType; 6] = [2,1,0,0,3,2];

#[repr(C)]
#[derive(Clone)]
pub struct Vertex {
    pos: Vec3,
    color: Vec4,
    texcoords: Vec2,
    texmask: u32,
    flags: u32
}

impl Vertex {
    pub const NUM_ATTRIBUTES: usize = 5;

    pub fn new() -> Self {
        Self {
            pos: Vec3::zero(),
            color: Vec4::zero(),
            texcoords: Vec2::zero(),
            texmask: DEFAULT_TEXTURE_MASK,
            flags: 0x0
        }
    }

    pub fn set_pos(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.pos.x = x;
        self.pos.y = y;
        self.pos.z = z;
        self
    }

    pub fn set_color(&mut self, r: f32, g: f32, b: f32, a: f32) -> &mut Self {
        self.color.x = r;
        self.color.y = g;
        self.color.z = b;
        self.color.w = a;
        self
    }

    pub fn set_texcoord(&mut self, u: f32, v: f32) -> &mut Self {
        self.texcoords.x = u;
        self.texcoords.y = v;
        self
    }

    pub fn set_texmask(&mut self, val: u32) -> &mut Self {
        self.texmask = val;
        self
    }

    pub fn set_flags(&mut self, val: u32) -> &mut Self {
        self.flags = val;
        self
    }

    pub fn get_binding_description() -> vk::VertexInputBindingDescription {

        let stride = core::mem::size_of::<Vertex>();

        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(stride as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; Self::NUM_ATTRIBUTES] {

        let attribute_descriptions = [

            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Vertex, pos) as _),

                vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32A32_SFLOAT)
                .offset(offset_of!(Vertex, color) as _),

                vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Vertex, texcoords) as _),

                vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(3)
                .format(vk::Format::R32_UINT)
                .offset(offset_of!(Vertex, texmask) as _),

                vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(4)
                .format(vk::Format::R32_UINT)
                .offset(offset_of!(Vertex, flags) as _)
            ];

        attribute_descriptions

    }
}

pub struct Quad {
    vertices: [Vertex; 4],
    indices: [IndexBufferElementType; 6],
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,

    modified: bool,
    coords: Vec4,
    color: Vec4,
    texcoords: Vec4,
    texmask: u32,
    flags: u32
}

impl Disposable for Quad {
    fn dispose(&mut self) {
        self.vertex_buffer.dispose();
        self.index_buffer.dispose();
    }
}

impl Quad {
    pub fn new() -> Self {

        let vertices: [Vertex; 4] = [ Vertex::new(), Vertex::new(), Vertex::new(), Vertex::new() ];
        let vertex_buffer = VertexBuffer::new(std::mem::size_of::<[Vertex; 4]>());

        let indices = DEFAULT_QUAD_INDICES;
        let index_buffer = IndexBuffer::new(std::mem::size_of::<[IndexBufferElementType; 6]>());
        index_buffer.copy(indices.as_ptr() as *const std::ffi::c_void).unwrap();

        Self {
            vertices,
            indices,
            vertex_buffer,
            index_buffer,
            modified: true,
            coords: Vec4::new(0.0, 0.0, 100.0, 100.0),
            color: DEFAULT_COLOR.clone(),
            texcoords: DEFAULT_TEXTURE_COORDS.clone(),
            texmask: DEFAULT_TEXTURE_MASK,
            flags: DEFAULT_FLAGS
        }

    }

    pub fn set_position(&mut self, x: f32, y: f32) -> &mut Self {
        self.coords.x = x;
        self.coords.y = y;
        self.modified = true;
        self
    }

    pub fn set_size(&mut self, w: f32, h: f32) -> &mut Self {
        self.coords.z = w;
        self.coords.w = h;
        self.modified = true;
        self
    }

    pub fn set_coords(&mut self, x: f32, y: f32, w: f32, h: f32) -> &mut Self {
        self.coords.x = x;
        self.coords.y = y;
        self.coords.z = w;
        self.coords.w = h;
        self.modified = true;
        self
    }

    pub fn set_color(&mut self, r: f32, g: f32, b: f32, a: f32) -> &mut Self {
        self.color.x = r;
        self.color.y = g;
        self.color.z = b;
        self.color.w = a;
        self.modified = true;
        self
    }

    pub fn set_texture_coords(&mut self, u0: f32, v0: f32, u1: f32, v1: f32) -> &mut Self {
        self.texcoords.x = u0;
        self.texcoords.y = v0;
        self.texcoords.z = u1;
        self.texcoords.w = v1;
        self.modified = true;
        self
    }

    pub fn set_texture_mask(&mut self, val: u32) -> &mut Self {
        self.texmask = val;
        self.modified = true;
        self
    }

    pub fn set_flags(&mut self, val: u32) -> &mut Self {
        self.flags = val;
        self.modified = true;
        self
    }

    pub fn draw(&mut self) {
        self.update();

        let device = crate::globals::device();
        let num_indices = self.indices.len();

        let pipeline = crate::globals::pipeline();
        let frame = pipeline.current_frame();

        self.vertex_buffer.bind(frame).unwrap();
        self.index_buffer.bind(frame).unwrap();

        let command_buffer = frame.command_buffer.obj;

        unsafe { device.obj.cmd_draw_indexed(
            command_buffer,
            num_indices as u32,
            1,
            0,
            0,
            0
        ) };
    }

    pub fn update(&mut self) {

        if !self.modified {
            return;
        }

        let vertex_buffer = &self.vertex_buffer;

        let coords = &self.coords;
        let texcoords = &self.texcoords;
        let color = &self.color;
        let texmask = self.texmask;
        let flags = self.flags;

        let x0 = coords.x;
        let y0 = coords.y;
        let x1 = x0 + coords.z;
        let y1 = y0 + coords.w;
        let z = 0.0f32;

        let u0 = texcoords.x;
        let v0 = texcoords.y;
        let u1 = u0 + texcoords.z;
        let v1 = v0 + texcoords.w;

        let r = color.x;
        let g = color.y;
        let b = color.z;
        let a = color.w;

        let v = &mut self.vertices[0];
        v.set_pos(x0, y0, z);
        v.set_texcoord(u0, v0);
        v.set_color(r, g, b, a);
        v.set_texmask(texmask);
        v.set_flags(flags);

        let v = &mut self.vertices[1];
        v.set_pos(x1, y0, z);
        v.set_texcoord(u1, v0);
        v.set_color(r, g, b, a);
        v.set_texmask(texmask);
        v.set_flags(flags);

        let v = &mut self.vertices[2];
        v.set_pos(x1, y1, z);
        v.set_texcoord(u1, v1);
        v.set_color(r, g, b, a);
        v.set_texmask(texmask);
        v.set_flags(flags);

        let v = &mut self.vertices[3];
        v.set_pos(x0, y1, z);
        v.set_texcoord(u0, v1);
        v.set_color(r, g, b, a);
        v.set_texmask(texmask);
        v.set_flags(flags);

        self.modified = false;

        let vertex_data = self.vertices.as_ptr() as *const std::ffi::c_void;
        vertex_buffer.copy(vertex_data).unwrap();

    }

}

pub struct VertexQueue {
    capacity: usize,
    reserved: usize,
    modified: bool,
    count: usize,

    vertices: Vec<Vertex>,
    indices: Vec<IndexBufferElementType>,

    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer
}

impl Disposable for VertexQueue {
    fn dispose(&mut self) {
        self.vertex_buffer.dispose();
        self.index_buffer.dispose();
        self.clear();
    }
}

impl VertexQueue {

    const NPOS: usize = usize::MAX;

    pub fn new(capacity: usize) -> Self {
        let num_vertices = capacity * 4;
        let mut vertices: Vec<Vertex> = Vec::new();
        vertices.resize(num_vertices, Vertex::new());

        let vertex_buffer = VertexBuffer::new(num_vertices * std::mem::size_of::<Vertex>());

        let num_indices = capacity * 6;
        let mut indices: Vec<IndexBufferElementType> = Vec::with_capacity(num_indices);

        let mut ofs: IndexBufferElementType = 0;
        for _ in 0..capacity {
            indices.push(ofs+2);
            indices.push(ofs+1);
            indices.push(ofs+0);
            indices.push(ofs+0);
            indices.push(ofs+3);
            indices.push(ofs+2);
            ofs += 4;
        }

        let index_buffer = IndexBuffer::new(num_indices * std::mem::size_of::<IndexBufferElementType>());
        index_buffer.copy(indices.as_ptr() as *const std::ffi::c_void).unwrap();

        Self {
            capacity,
            reserved: 0,
            modified: false,
            count: 0,

            vertices,
            indices,
            vertex_buffer,
            index_buffer
        }
    }

    pub fn realloc(&mut self, capacity: usize) {

        self.dispose();

        let q = Self::new(capacity);

        self.capacity = capacity;
        self.reserved = 0;
        self.modified = false;
        self.count = 0;

        self.vertices = q.vertices;
        self.indices = q.indices;
        self.vertex_buffer = q.vertex_buffer;
        self.index_buffer = q.index_buffer;

    }

    pub fn begin(&mut self) {
        self.count = 0;
    }

    pub fn end(&mut self) {
    }

    pub fn clear(&mut self) {
        self.count = 0;
        self.reserved = 0;
    }

    pub fn reserve(&mut self, num_indices: usize) -> usize {

        if self.count > 0 {
            panic!("cannot reserve after dynamic push to sprite batch");
        }

        if self.count + self.reserved + num_indices > self.capacity {
            panic!("vertex queue overflow");
        }

        let index = self.reserved;

        self.reserved += num_indices;

        index
    }

    pub fn update(&mut self) {
        let num = self.count + self.reserved;

        if !self.modified || 0 == num {
            return;
        }

        self.modified = false;

        let num_vertices = num * 4;

        let vertex_data = self.vertices.as_ptr() as *const std::ffi::c_void;
        let vertex_data_size = num_vertices * std::mem::size_of::<Vertex>();

        self.vertex_buffer.copy_region(vertex_data, 0, vertex_data_size).unwrap();

    }

    pub fn draw(&mut self) {
        self.update();

        let num = self.count + self.reserved;

        if 0 == num {
            return;
        }

        let num_indices = num * 6;
        let pipeline = crate::globals::pipeline();
        let frame = pipeline.current_frame();
        self.vertex_buffer.bind(frame).unwrap();
        self.index_buffer.bind(frame).unwrap();

        let command_buffer = frame.command_buffer.obj;

        let device = crate::globals::device();
        unsafe { device.obj.cmd_draw_indexed (
            command_buffer,
            num_indices as u32,
            1, 0, 0, 0
        ) };
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn count(&self) -> usize {
        self.count
    }

    fn check_index(&mut self, index: &mut usize) {
        if *index == Self::NPOS {
            if self.count + self.reserved >= self.capacity {
                panic!("vertex queue overflow");
            }
            *index = self.count + self.reserved;
            self.count += 1;
        } else {
            if *index >= self.count + self.reserved {
                panic!("vertex queue index out of bounds")
            }
        }
    }

    pub fn set_coords(&mut self, index: usize, x: f32, y: f32, w: f32, h: f32) {
        let x0 = x;
        let y0 = y;
        let x1 = x0 + w;
        let y1 = y0 + h;
        let z = 0.0f32;

        let ofs = index * 4;
        let vertices = &mut self.vertices;
        vertices[ofs+0].set_pos(x0, y0, z);
        vertices[ofs+1].set_pos(x1, y0, z);
        vertices[ofs+2].set_pos(x1, y1, z);
        vertices[ofs+3].set_pos(x0, y1, z);
        self.modified = true;
    }

    pub fn set_color(&mut self, index: usize, r: f32, g: f32, b: f32, a: f32) {
        let ofs = index * 4;
        let vertices = &mut self.vertices;
        vertices[ofs+0].set_color(r, g, b, a);
        vertices[ofs+1].set_color(r, g, b, a);
        vertices[ofs+2].set_color(r, g, b, a);
        vertices[ofs+3].set_color(r, g, b, a);
        self.modified = true;
    }

    pub fn set_texture_coords(&mut self, index: usize, x: f32, y: f32, w: f32, h: f32) {
        let x0 = x;
        let y0 = y;
        let x1 = x0 + w;
        let y1 = y0 + h;

        let ofs = index * 4;
        let vertices = &mut self.vertices;
        vertices[ofs+0].set_texcoord(x0, y0);
        vertices[ofs+1].set_texcoord(x1, y0);
        vertices[ofs+2].set_texcoord(x1, y1);
        vertices[ofs+3].set_texcoord(x0, y1);
        self.modified = true;
    }

    pub fn set_texture_mask(&mut self, index: usize, texture_mask: u32) {
        let ofs = index * 4;
        let vertices = &mut self.vertices;
        vertices[ofs+0].set_texmask(texture_mask);
        vertices[ofs+1].set_texmask(texture_mask);
        vertices[ofs+2].set_texmask(texture_mask);
        vertices[ofs+3].set_texmask(texture_mask);
        self.modified = true;
    }

    pub fn set_flags(&mut self, index: usize, flags: u32) {
        let ofs = index * 4;
        let vertices = &mut self.vertices;
        vertices[ofs+0].set_flags(flags);
        vertices[ofs+1].set_flags(flags);
        vertices[ofs+2].set_flags(flags);
        vertices[ofs+3].set_flags(flags);
        self.modified = true;
    }

    pub fn push(&mut self,
        x: f32, y: f32, w: f32, h: f32,
        r: f32, g: f32, b: f32, a: f32,
        tx: f32, ty: f32, tw: f32, th: f32,
        texture_mask: u32, flags: u32) {

        let index = self.count + self.reserved;

        self.store(index,
            x, y, w, h,
            r, g, b, a,
            tx, ty, tw, th,
            texture_mask, flags
        );

        self.count += 1;
    }

    pub fn store(&mut self, index: usize,
        x: f32, y: f32, w: f32, h: f32,
        r: f32, g: f32, b: f32, a: f32,
        tx: f32, ty: f32, tw: f32, th: f32,
        texture_mask: u32, flags: u32) {

        if index >= self.capacity {
            panic!("vertex queue overflow");
        }

        self.set_coords(index, x, y, w, h);
        self.set_color(index, r, g, b, a);
        self.set_texture_coords(index, tx, ty, tw, th);
        self.set_texture_mask(index, texture_mask);
        self.set_flags(index, flags);

        self.modified = true;

    }

}

#[repr(C)]
#[derive(Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32
}

impl Default for Color {
    fn default() -> Self {
        Self { r:0.0, g:0.0, b:0.0, a:0.0 }
    }
}

impl Color {
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {r, g, b, a}
    }

    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self {r, g, b, a:1.0}
    }

    pub const fn zero() -> Self {
        Self { r:0.0, g:0.0, b:0.0, a:0.0 }
    }

    pub const fn black() -> Self {
        Self { r:0.0, g:0.0, b:0.0, a:1.0 }
    }

    pub fn white() -> Self {
        Self { r:1.0, g:1.0, b:1.0, a:1.0 }
    }

    pub fn red(&self) -> f32 { self.r }
    pub fn green(&self) -> f32 { self.g }
    pub fn blue(&self) -> f32 { self.b }
    pub fn alpha(&self) -> f32 { self.a }

    pub fn set(&mut self, color: &Self) {
        self.r = color.r;
        self.g = color.g;
        self.b = color.b;
        self.a = color.a;
    }

}
