//!
//! Types
//!

use std::fs::File;
use std::io::{Cursor, Read};

use ash::vk;

use crate::api::{Disposable, LockRef};
use crate::error::Error;
use crate::manifest::StaticShaderDescriptor;

pub struct ShaderType {}

impl ShaderType {
    pub const UNKNOWN: u32 = 0x0;
    pub const VERTEX_SHADER: u32 = 0x1;
    pub const FRAGMENT_SHADER: u32 = 0x2;
}

pub struct ShaderDescriptor {
    code: *const std::ffi::c_void,
    code_size: usize,
    shader_type: u32
}

pub struct Shader {
    pub obj: vk::ShaderModule,
    pub shader_type: u32
}

pub type ShaderRef = std::sync::Arc<Shader>;
pub type ShaderLockRef = LockRef<Shader>;

impl Disposable for Shader {
    fn dispose(&mut self) {
        let device = crate::globals::device();
        unsafe { device.obj.destroy_shader_module(self.obj, None); }
        self.obj = vk::ShaderModule::null();
    }
}

impl Shader {

    fn new(code: &Vec<u32>, shader_type: u32) -> Result<Self, Error> {

        let device = crate::globals::device();

        let shader_module_create_info = vk::ShaderModuleCreateInfo::default()
            .code(code);

        let obj = unsafe { match device.obj.create_shader_module(&shader_module_create_info, None) {
            Ok(obj) => obj,
            Err(_) => { return Err(Error::from("failed to create shader module")); }
        } };

        Ok(Self{
            obj,
            shader_type
        })
    }

    pub fn from_resource(descriptor: &StaticShaderDescriptor) -> Result<Self, Error> {
        let data_ptr = descriptor.data.as_ptr() as *const std::ffi::c_uint;
        let num_code_words = descriptor.data.len() / 4;
        let code = unsafe { core::slice::from_raw_parts(data_ptr, num_code_words) }.to_vec();
        let shader_type = if descriptor.format == "vertex" { ShaderType::VERTEX_SHADER } else { ShaderType::FRAGMENT_SHADER };
        Self::new(&code, shader_type)
    }

    pub fn from_file(filename: &str, shader_type: u32) -> Result<Self, Error> {
        let mut file = File::open(filename).unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        let mut cursor = Cursor::new(buf);
        let code = ash::util::read_spv(&mut cursor).unwrap();
        Self::new(&code, shader_type)
    }

}
