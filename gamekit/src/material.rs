//!
//! Material
//!

use log::{*};
use std::{collections::HashMap, sync::{Arc, Mutex, MutexGuard}};

use ash::vk::{self, Handle};

use crate::{api::{Disposable, LockRef}, buffer::{PushConstants, Uniform, UniformBufferLockRef}, error::Error, font::{Font, FontLockRef}, manifest::StaticMaterialDescriptor, primitives::Vertex, shader::{ShaderLockRef, ShaderType}, texture::{Texture, TextureBinding, TextureLockRef}};

const DEFAULT_SHADER_ENTRY_POINT: &str = "main";

pub struct BlendMode {}

impl BlendMode {
    pub const NORMAL: u32 = 0x1;
    pub const ADDITIVE: u32 = 0x2;
    pub const MULTIPLY: u32 = 0x3;

    pub fn from_string(blend_mode: &str) -> u32 {
        match blend_mode {
            "additive" => BlendMode::ADDITIVE,
            "multiply" => BlendMode::MULTIPLY,
            _ => BlendMode::NORMAL
        }
    }
}

pub struct RenderState {
    pub modified: bool,
    pub enable_blending: bool,
    pub blend_mode: u32,
    pub backface_culling: bool,
    pub frontface_clockwise: bool,
    pub depth_testing: bool,
    pub depth_writing: bool
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            modified: true,
            enable_blending: true,
            blend_mode: BlendMode::NORMAL,
            backface_culling: true,
            frontface_clockwise: false,
            depth_testing: false,
            depth_writing: false
        }
    }
}

impl RenderState {
    pub fn invalidate(&mut self) {
        self.modified = true;
    }

    pub fn copy(&mut self, other: &RenderState) {
        self.set_blending(other.enable_blending);
        self.set_blend_mode(other.blend_mode);
        self.set_backface_culling(other.backface_culling);
        self.set_frontface_clockwise(other.frontface_clockwise);
        self.set_depth_testing(other.depth_testing);
        self.set_depth_writing(other.depth_writing);
    }

    pub fn set_blending(&mut self, val: bool) -> &mut Self {
        if val != self.enable_blending {
            self.enable_blending = val;
            self.modified = true;
        }
        self
    }

    pub fn set_blend_mode(&mut self, val: u32) -> &mut Self {
        if val != self.blend_mode {
            self.blend_mode = val;
            self.modified = true;
        }
        self
    }

    pub fn set_backface_culling(&mut self, val: bool) -> &mut Self {
        if val != self.backface_culling {
            self.backface_culling = val;
            self.modified = true;
        }
        self
    }

    pub fn set_frontface_clockwise(&mut self, val: bool) -> &mut Self {
        if val != self.frontface_clockwise {
            self.frontface_clockwise = val;
            self.modified = true;
        }
        self
    }

    pub fn set_depth_testing(&mut self, val: bool) -> &mut Self {
        if val != self.backface_culling {
            self.backface_culling = val;
            self.modified = true;
        }
        self
    }

    pub fn set_depth_writing(&mut self, val: bool) -> &mut Self {
        if val != self.depth_writing {
            self.depth_writing = val;
            self.modified = true;
        }
        self
    }

    pub fn push(&mut self) {

        self.modified = false;

        let device = crate::globals::device();
        let pipeline = crate::globals::pipeline();
        let frame = pipeline.current_frame();
        let command_buffer = frame.command_buffer.obj;

        unsafe {
            device.obj.cmd_set_depth_test_enable(command_buffer, self.depth_testing);
            device.obj.cmd_set_depth_write_enable(command_buffer, self.depth_writing);

            let cull_mode = if self.backface_culling { vk::CullModeFlags::BACK } else { vk::CullModeFlags::NONE };
            device.obj.cmd_set_cull_mode(command_buffer, cull_mode);

            let front_face = if self.frontface_clockwise { vk::FrontFace::CLOCKWISE } else { vk::FrontFace::COUNTER_CLOCKWISE };
            device.obj.cmd_set_front_face(command_buffer, front_face);

            if device.dynamic_state_device.is_some() {

                let dyn_device = device.dynamic_state_device.as_ref().unwrap();

                let color_blend_enables = [ if self.enable_blending { vk::TRUE } else { vk::FALSE } ];
                dyn_device.cmd_set_color_blend_enable(
                    command_buffer,
                    0,
                    &color_blend_enables
                );

                let (src_blend_factor, dst_blend_factor) = match self.blend_mode {
                    BlendMode::NORMAL => (vk::BlendFactor::SRC_ALPHA, vk::BlendFactor::ONE_MINUS_SRC_ALPHA),
                    BlendMode::ADDITIVE => (vk::BlendFactor::SRC_ALPHA, vk::BlendFactor::ONE),
                    BlendMode::MULTIPLY => (vk::BlendFactor::DST_COLOR, vk::BlendFactor::ZERO),
                    _ => (vk::BlendFactor::SRC_ALPHA, vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                };

                let color_blend_equations = [
                    vk::ColorBlendEquationEXT {
                        src_color_blend_factor: src_blend_factor,
                        dst_color_blend_factor: dst_blend_factor,
                        color_blend_op: vk::BlendOp::ADD,
                        src_alpha_blend_factor: vk::BlendFactor::ONE,
                        dst_alpha_blend_factor: vk::BlendFactor::ZERO,
                        alpha_blend_op: vk::BlendOp::ADD
                    }
                ];

                dyn_device.cmd_set_color_blend_equation(
                    command_buffer,
                    0,
                    &color_blend_equations
                );
            }
        }

    }

}

pub struct PushConstantsInfo {
    range: vk::PushConstantRange
}

pub struct ShaderInfo {
    shader: ShaderLockRef,
    entry_point: std::ffi::CString
}

/// Material
pub struct Material {
    invalidated: bool,

    render_state: RenderState,
    textures: Vec<TextureBinding>,
    shaders: Vec<ShaderInfo>,
    uniforms: Vec<UniformBufferLockRef>,
    push_constant_ranges: Vec<vk::PushConstantRange>,
    font: FontLockRef,

    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    pub pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,
    pub descriptor_sets: Vec<vk::DescriptorSet>

}

pub type MaterialRef = std::sync::Arc<Material>;
pub type MaterialLockRef = LockRef<Material>;

impl Disposable for Material {
    fn dispose(&mut self) {

        self.free_descriptor_sets();
        self.free_graphics_pipeline();

        self.font.lock().unwrap().dispose();

        for element in &mut self.textures {
            element.dispose();
        }
        self.textures.clear();
        self.shaders.clear();

        for element in &mut self.uniforms {
            element.lock().unwrap().dispose();
        }
        self.uniforms.clear();

        self.push_constant_ranges.clear();
    }
}

impl Material {
    pub fn new() -> Self {
        Self {
            invalidated: true,
            render_state: RenderState::default(),

            textures: Vec::new(),
            shaders: Vec::new(),
            uniforms: Vec::new(),
            push_constant_ranges: Vec::new(),
            font: Arc::new(Mutex::new(Font::default())),

            descriptor_pool: vk::DescriptorPool::null(),
            descriptor_set_layout: vk::DescriptorSetLayout::null(),
            pipeline_layout: vk::PipelineLayout::null(),
            graphics_pipeline: vk::Pipeline::null(),
            descriptor_sets: Vec::new()

        }
    }

    pub fn from_static(descriptor: &StaticMaterialDescriptor) -> Self {

        let resources = crate::globals::resources();

        let mut material = Self::new();

        material.set_blending(descriptor.blending);
        material.set_blend_mode(BlendMode::from_string(descriptor.blend_mode));
        material.set_backface_culling(descriptor.backface_culling);
        material.set_frontface_clockwise(descriptor.frontface_clockwise);
        material.set_depth_testing(descriptor.depth_testing);
        material.set_depth_writing(descriptor.depth_writing);

        if descriptor.font.len() > 0 {
            let font_ref = &resources.get_font(&descriptor.font);
            material.set_font(font_ref);
        }

        if descriptor.texture.len() > 0 {
            let texture_ref = &resources.get_texture(&descriptor.texture);
            material.add_texture(texture_ref, descriptor.texture_binding, descriptor.texture_filtering);
        }

        if descriptor.vertex_shader.len() > 0 {
            let shader_ref = resources.get_shader(&descriptor.vertex_shader);
            material.add_shader(shader_ref);
        }

        if descriptor.fragment_shader.len() > 0 {
            let shader_ref = resources.get_shader(&descriptor.fragment_shader);
            material.add_shader(shader_ref);
        }

        material
    }

    pub fn to_lockref(material: Self) -> MaterialLockRef {
        Arc::new(Mutex::new(material))
    }

    pub fn set_blending(&mut self, val: bool) -> &mut Self { self.render_state.set_blending(val); self }
    pub fn set_blend_mode(&mut self, val: u32) -> &mut Self { self.render_state.set_blend_mode(val); self }
    pub fn set_backface_culling(&mut self, val: bool) -> &mut Self { self.render_state.set_backface_culling(val); self }
    pub fn set_frontface_clockwise(&mut self, val: bool) -> &mut Self { self.render_state.set_frontface_clockwise(val); self }
    pub fn set_depth_testing(&mut self, val: bool) -> &mut Self { self.render_state.set_depth_testing(val); self }
    pub fn set_depth_writing(&mut self, val: bool) -> &mut Self { self.render_state.set_depth_writing(val); self }

    pub fn add_shader(&mut self, shader: ShaderLockRef) -> &mut Self {

        let entry_point = DEFAULT_SHADER_ENTRY_POINT;

        let entry_point_str = entry_point.to_string();
        let entry_point_cstr: std::ffi::CString = std::ffi::CString::new(entry_point_str.as_str()).unwrap();

        let shader_info = ShaderInfo {
            shader,
            entry_point: entry_point_cstr
        };

        self.shaders.push(shader_info);
        self.invalidated = true;
        self
    }

    pub fn add_push_constants<T: Default>(&mut self, push_constants: &PushConstants<T>) -> &mut Self {
        let range = vk::PushConstantRange::default()
            .offset(0)
            .size(push_constants.size() as u32)
            .stage_flags(vk::ShaderStageFlags::ALL_GRAPHICS);
        self.push_constant_ranges.push(range);
        self.invalidated = true;
        self
    }

    pub fn add_uniform<T: Default>(&mut self, uniform: &Uniform<T>) -> &mut Self {
        let uniform_buffer = uniform.get_buffer_ref();
        self.uniforms.push(uniform_buffer);
        self.invalidated = true;
        self
    }

    pub fn add_texture(&mut self, texture_ref: &TextureLockRef, binding: u32, filtering: bool) -> &mut Self {
        let texture_binding = Texture::get_binding(texture_ref, binding, filtering);
        self.textures.push(texture_binding);
        self.invalidated = true;
        self
    }

    pub fn set_font(&mut self, font_ref: &FontLockRef) -> &mut Self {
        self.font = font_ref.clone();
        self
    }

    pub fn font(&self) -> &FontLockRef {
        &self.font
    }

    fn compile(&mut self) {
        self.validate_pipeline();
    }

    pub fn bind(&mut self) {
        self.validate_pipeline();
        self.bind_pipeline();
        self.render_state.push();
        self.bind_uniforms();
    }

    fn bind_pipeline(&mut self) {
        let device = crate::globals::device();
        let pipeline = crate::globals::pipeline();
        let frame = pipeline.current_frame();
        let command_buffer = &frame.command_buffer;
        let graphics_pipeline = self.graphics_pipeline;

        unsafe {
            device.obj.cmd_bind_pipeline(
                command_buffer.obj,
                vk::PipelineBindPoint::GRAPHICS,
                graphics_pipeline
            );
        }
    }

    pub fn bind_uniforms(&self) {

        let device = crate::globals::device();
        let pipeline_layout = self.pipeline_layout;
        let pipeline = crate::globals::pipeline();
        let frame = pipeline.current_frame();
        let command_buffer = &frame.command_buffer;

        let null: [u32; 0] = [];
        let mut dynamic_offsets: Vec<u32> = vec![];
        for uniform_ref in &self.uniforms {
            let uniform = uniform_ref.lock().unwrap();
            if uniform.is_dynamic() {
                dynamic_offsets.push(uniform.offset() as u32);
            }
        }

        if self.descriptor_sets.len() > frame.index as usize {
            let descriptor_set = self.descriptor_sets[frame.index as usize];
            unsafe {
                device.obj.cmd_bind_descriptor_sets(
                    command_buffer.obj,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline_layout,
                    0,
                    &[descriptor_set],
                    if dynamic_offsets.len() > 0 { &dynamic_offsets } else { &null }
                );
            }
        }
    }

    fn validate_pipeline(&mut self) {

        if !self.invalidated {
            return;
        }

        self.invalidated = false;

        self.free_descriptor_sets();
        self.free_graphics_pipeline();

        self.create_graphics_pipeline();
        self.create_descriptor_sets();

    }

    fn create_graphics_pipeline(&mut self) {

        let metrics = crate::globals::metrics();
        let device = crate::globals::device();
        let pipeline = crate::globals::pipeline();

        let state = &self.render_state;

        let viewport = vk::Viewport::default();
        let viewports = [viewport];

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D { width: metrics.window_width as u32, height: metrics.window_height as u32 }
        };
        let scissors = [scissor];

        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewports(&viewports)
            .scissors(&scissors);

        let vertex_binding_descriptions = [ Vertex::get_binding_description() ];
        let vertex_attribute_descriptions = Vertex::get_attribute_descriptions();

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&vertex_binding_descriptions)
            .vertex_attribute_descriptions(&vertex_attribute_descriptions);

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let cull_mode = if state.backface_culling { vk::CullModeFlags::BACK } else { vk::CullModeFlags::NONE };
        let front_face = if state.frontface_clockwise { vk::FrontFace::CLOCKWISE } else { vk::FrontFace::COUNTER_CLOCKWISE };

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(cull_mode)
            .front_face(front_face)
            .depth_bias_enable(false)
            .depth_bias_constant_factor(0.0)
            .depth_bias_clamp(0.0)
            .depth_bias_slope_factor(0.0);

        let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .min_sample_shading(1.0)
            .sample_mask(&[])
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false);

        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(state.depth_testing)
            .depth_write_enable(state.depth_writing)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false);

        let (src_blend_factor, dst_blend_factor) = match state.blend_mode {
            BlendMode::NORMAL => (vk::BlendFactor::SRC_ALPHA, vk::BlendFactor::ONE_MINUS_SRC_ALPHA),
            BlendMode::ADDITIVE => (vk::BlendFactor::SRC_ALPHA, vk::BlendFactor::ONE),
            BlendMode::MULTIPLY => (vk::BlendFactor::DST_COLOR, vk::BlendFactor::ZERO),
            _ => (vk::BlendFactor::SRC_ALPHA, vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        };

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B | vk::ColorComponentFlags::A)
            .blend_enable(state.enable_blending)
            .src_color_blend_factor(src_blend_factor)
            .dst_color_blend_factor(dst_blend_factor)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD);

        let color_blend_attachments = [ color_blend_attachment ];

        let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&color_blend_attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        ///////////////////////////////////////////////////////////////////////////////
        // Dynamic state changes at draw time
        ///////////////////////////////////////////////////////////////////////////////

        let mut dynamic_states = Vec::new();
        dynamic_states.push(vk::DynamicState::VIEWPORT);
        dynamic_states.push(vk::DynamicState::SCISSOR);
        dynamic_states.push(vk::DynamicState::DEPTH_TEST_ENABLE);
        dynamic_states.push(vk::DynamicState::DEPTH_WRITE_ENABLE);
        dynamic_states.push(vk::DynamicState::CULL_MODE);
        dynamic_states.push(vk::DynamicState::FRONT_FACE);

        if device.features.has_dynamic_state_3() {
            dynamic_states.push(vk::DynamicState::COLOR_BLEND_ENABLE_EXT);
            dynamic_states.push(vk::DynamicState::COLOR_BLEND_EQUATION_EXT);
        }

        let dynamic_state = vk::PipelineDynamicStateCreateInfo::default()
            .dynamic_states(&dynamic_states);

        ///////////////////////////////////////////////////////////////////////////////
        // Pipeline Layout
        ///////////////////////////////////////////////////////////////////////////////

        let mut descriptor_set_layout_bindings: Vec<vk::DescriptorSetLayoutBinding> = vec![];

        for uniform_ref in &self.uniforms {
            let uniform = uniform_ref.lock().unwrap();
            let descriptor_type = if uniform.dynamic { vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC } else { vk::DescriptorType::UNIFORM_BUFFER };
            descriptor_set_layout_bindings.push(vk::DescriptorSetLayoutBinding::default()
                .descriptor_type(descriptor_type)
                .binding(uniform.binding())
                .stage_flags(vk::ShaderStageFlags::ALL_GRAPHICS)
                .descriptor_count(1));
        }

        for texture_info in &self.textures {
            descriptor_set_layout_bindings.push(vk::DescriptorSetLayoutBinding::default()
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .binding(texture_info.binding())
                .stage_flags(vk::ShaderStageFlags::ALL_GRAPHICS)
                .descriptor_count(1));
        }

        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&descriptor_set_layout_bindings);


        let descriptor_set_layout =  if descriptor_set_layout_bindings.len() > 0 {
            unsafe { device.obj.create_descriptor_set_layout(&layout_info, None).unwrap() }
        } else {
            vk::DescriptorSetLayout::null()
        };

        let mut descriptor_set_layouts: Vec<vk::DescriptorSetLayout> = Vec::new();
        if descriptor_set_layout_bindings.len() > 0 {
            descriptor_set_layouts.push(descriptor_set_layout);
        };

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&descriptor_set_layouts)
            .push_constant_ranges(&self.push_constant_ranges);

        let pipeline_layout = unsafe { device.obj.create_pipeline_layout(&pipeline_layout_info, None).unwrap() };

        ///////////////////////////////////////////////////////////////////////////////
        // Shaders
        ///////////////////////////////////////////////////////////////////////////////

        let mut shader_stages: Vec<vk::PipelineShaderStageCreateInfo> = vec![];

        for shader_info in &self.shaders {
            let shader = shader_info.shader.lock().unwrap();
            let stage = if shader.shader_type == ShaderType::FRAGMENT_SHADER { vk::ShaderStageFlags::FRAGMENT } else { vk::ShaderStageFlags::VERTEX };

            shader_stages.push(vk::PipelineShaderStageCreateInfo::default()
                .stage(stage)
                .module(shader.obj)
                .name(shader_info.entry_point.as_c_str()) // c"main"
            );
        }

        let render_pass = pipeline.render_pass();

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .depth_stencil_state(&depth_stencil)
            .color_blend_state(&color_blending)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .render_pass(*render_pass)
            .subpass(0u32)
            .base_pipeline_handle(vk::Pipeline::null())
            .base_pipeline_index(-1);

        let pipeline_infos = [pipeline_info];

        let graphics_pipeline = unsafe { device.obj.create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_infos, None).unwrap() };

        self.descriptor_set_layout = descriptor_set_layout;
        self.pipeline_layout = pipeline_layout;
        self.graphics_pipeline = graphics_pipeline[0];

    }

    fn free_graphics_pipeline(&mut self) {

        let device = crate::globals::device();
        if self.graphics_pipeline != vk::Pipeline::null() {
            unsafe {
                device.obj.destroy_pipeline(self.graphics_pipeline, None);
                self.graphics_pipeline = vk::Pipeline::null();

                device.obj.destroy_pipeline_layout(self.pipeline_layout, None);
                self.pipeline_layout = vk::PipelineLayout::null();

                device.obj.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
                self.descriptor_set_layout = vk::DescriptorSetLayout::null();
            }
        }
    }

    fn create_descriptor_sets(&mut self) {

        if self.descriptor_set_layout.is_null() {
            // no objects (such as uniforms, textures, etc.) defined
            return;
        }


        let device = crate::globals::device();
        let pipeline = crate::globals::pipeline();

        let num_frames = pipeline.frame_count();

        let mut pool_sizes : Vec<vk::DescriptorPoolSize> = vec![];

        let mut num_static_uniforms = 0usize;
        let mut num_dynamic_uniforms = 0usize;

        for uniform_ref in &self.uniforms {
            let uniform = uniform_ref.lock().unwrap();
            if uniform.dynamic {
                num_dynamic_uniforms += 1;
            } else {
                num_static_uniforms += 1;
            }
        }

        if num_static_uniforms > 0 {
            pool_sizes.push(vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(num_frames as u32)
            );
        }

        if num_dynamic_uniforms > 0 {
            pool_sizes.push(vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                .descriptor_count(num_frames as u32)
            );
        }

        let num_textures = self.textures.len();
        if num_textures > 0 {
            pool_sizes.push(vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(num_frames as u32)
            );
        }

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(num_frames as u32);

        self.descriptor_pool = unsafe {
            device.obj.create_descriptor_pool(&descriptor_pool_create_info, None).unwrap()
        };

        self.descriptor_sets.clear();

        for frame_index in 0..num_frames {
            let descriptor_set = self.create_descriptor_set(frame_index);
            self.descriptor_sets.push(descriptor_set);
        }
    }

    fn create_descriptor_set(&mut self, frame_index: usize) -> vk::DescriptorSet {

        let device = crate::globals::device();
        let layouts = [ self.descriptor_set_layout ];

        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.descriptor_pool)
            .set_layouts(&layouts);

        let descriptor_sets = unsafe {
            device.obj.allocate_descriptor_sets(&alloc_info).unwrap()
        };

        let descriptor_set = descriptor_sets[0];

        for uniform_ref in &self.uniforms {
            let uniform = uniform_ref.lock().unwrap();

            let buffer_info = uniform.get_buffer_info(frame_index);
            let buffer_infos = &[buffer_info];
            let descriptor_type = if uniform.dynamic { vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC } else { vk::DescriptorType::UNIFORM_BUFFER };

            let descriptor_write = vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(uniform.binding())
                .dst_array_element(0)
                .descriptor_type(descriptor_type)
                .buffer_info(buffer_infos);

            unsafe { device.obj.update_descriptor_sets(&[descriptor_write], &[]); }
        }

        for texture_info in &self.textures {
            let binding = texture_info.binding();
            if binding == u32::MAX {
                continue;
            }

            let &descriptor_info = &texture_info.descriptor;
            let image_info_ref = &[descriptor_info];
            let descriptor_write = vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(binding)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(image_info_ref);

            unsafe { device.obj.update_descriptor_sets(&[descriptor_write], &[]); }
        }

        descriptor_set
    }

    fn free_descriptor_sets(&mut self) {

        let device = crate::globals::device();

        if !self.descriptor_pool.is_null() {
            unsafe {
                if self.descriptor_sets.len() > 0 {
                    //let _ = device.free_descriptor_sets(self.descriptor_pool, &self.descriptor_sets);
                    self.descriptor_sets.clear();
                }

                device.obj.destroy_descriptor_pool(self.descriptor_pool, None);
                self.descriptor_pool = vk::DescriptorPool::null();
            };
        }
    }


}

pub struct Materials {
    materials: HashMap<String, MaterialLockRef>
}

impl Disposable for Materials {
    fn dispose(&mut self) {
        trace!("Materials::dispose");

        for (_, material) in &mut self.materials {
            material.lock().unwrap().dispose();
        }

        self.materials.clear();
    }
}

impl Default for Materials {
    fn default() -> Self {
        Self {
            //materials: vec![]
            materials: HashMap::new()
        }
    }
}

impl Materials {

    pub fn build(descriptors: &'static [StaticMaterialDescriptor]) -> Result<(), Error> {

        let materials = crate::globals::materials_mut();

        for descriptor in descriptors {
            let material = Material::from_static(descriptor);
            materials.add_material(descriptor.name, material);
        }

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.materials.len()
    }

    pub fn get_default(&self) -> MaterialLockRef {

        let mut default_name: &str = "";

        for (name, _) in &self.materials {
            default_name = name.as_str();
            break;
        }

        return self.get(default_name);
    }

    pub fn get(&self, name: &str) -> MaterialLockRef {
        let material_ref = self.materials.get(name).expect("material not found");
        material_ref.clone()
    }

    pub fn get_lock(&self, name: &str) -> MutexGuard<Material> {
        let material_ref = self.materials.get(name).expect("material not found");
        material_ref.lock().unwrap()
    }

    pub fn add_material(&mut self, name: &str, material: Material) -> MaterialLockRef {
        let material_ref = Material::to_lockref(material);
        self.materials.insert(name.to_string(), material_ref.clone());

        material_ref
    }

    pub fn compile(&self) {
        for (_, material) in &self.materials {
            material.lock().unwrap().compile();
        }
    }
}

