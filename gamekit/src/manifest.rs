//!
//! Manifest
//!

/*

Note: include the generated files like this:
        include!(concat!(env!("OUT_DIR"), "/manifest.rs"));

*/

use std::path::PathBuf;

use serde::Deserialize;

use crate::constants::Constants;

const MANIFEST_FILENAME: &str = "manifest.json";

pub fn name_from_path(name: &str, path: &str) -> String {
    if name.len() > 0 {
        name.to_owned()
    } else {
        let p = PathBuf::from(path);
        let name = p.file_stem().unwrap().to_str().unwrap().to_owned();
        name
    }
}

pub trait StaticDescriptor {
    fn name(&self) -> &'static str { "" }
    fn text(&self) -> &'static str { "" }
    fn format(&self) -> &'static str { "" }
    fn data(&self) -> &'static [u8] { &[] }
}


fn default_1() -> u32 { 1 }
fn default_true() -> bool { true }
fn default_fps() -> u32 { 60 }
fn default_imax() -> i32 { i32::MAX }
fn default_width() -> u32 { 400 }
fn default_height() -> u32 { 300 }
fn default_title() -> String { "gamekit".to_string() }
fn default_validation_layer() -> bool{ Constants::ENABLE_VALIDATION_LAYER }
fn default_api_dump_layer() -> bool { Constants::ENABLE_API_DUMP_LAYER }

#[derive(Default, Deserialize, Debug, PartialEq)]
#[serde(default, rename = "options", deny_unknown_fields)]
pub struct OptionsDescriptor {
    pub title: String,

    #[serde(default = "default_imax")]
    pub window_x: i32,

    #[serde(default = "default_imax")]
    pub window_y: i32,

    #[serde(default = "default_width")]
    pub window_width: u32,

    #[serde(default = "default_height")]
    pub window_height: u32,

    pub view_width: u32,
    pub view_height: u32,

    pub scaling_mode: String,

    #[serde(default = "default_fps")]
    pub fps: u32,

    pub show_statistics: bool,

    pub queue_size: usize,

    pub headless: bool,

    #[serde(default = "default_validation_layer")]
    pub enable_validation_layer: bool,

    #[serde(default = "default_api_dump_layer")]
    pub enable_api_dump_layer: bool
}

pub struct StaticOptionsDescriptor {
    pub title: &'static str,
    pub window_x: i32,
    pub window_y: i32,
    pub window_width: u32,
    pub window_height: u32,
    pub view_width: u32,
    pub view_height: u32,
    pub scaling_mode: i32,
    pub fps: u32,
    pub show_statistics: bool,
    pub queue_size: usize,
    pub headless: bool,
    pub enable_validation_layer: bool,
    pub enable_api_dump_layer: bool
}


#[derive(Default, Deserialize, Debug, PartialEq)]
#[serde(default, rename = "data", deny_unknown_fields)]
pub struct DataDescriptor {
    name: String,
    path: String
}

impl DataDescriptor {
    pub fn name(&self) -> String {
        name_from_path(&self.name, &self.path)
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

pub struct StaticDataDescriptor {
    pub name: &'static str,
    pub data: &'static [u8],
    pub size: usize
}

impl StaticDataDescriptor {
    pub const fn new(name: &'static str, data: &'static [u8]) -> Self {
        Self { name, data, size: data.len() }
    }
}

#[derive(Default, Deserialize, Debug, PartialEq)]
#[serde(default, rename = "texture", deny_unknown_fields)]
pub struct TextureDescriptor {
    name: String,
    path: String
}

impl TextureDescriptor {
    pub fn name(&self) -> String {
        name_from_path(&self.name, &self.path)
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

pub struct StaticTextureDescriptor {
    pub name: &'static str,
    pub data: &'static [u8],
    pub size: usize,
    pub format: &'static str
}

impl StaticTextureDescriptor {
    pub const fn new(name: &'static str, data: &'static [u8], format: &'static str) -> Self {
        Self { name, data, size: data.len(), format }
    }
}

#[derive(Default, Deserialize, Debug, PartialEq)]
#[serde(default, rename = "shader", deny_unknown_fields)]
pub struct ShaderDescriptor {
    name: String,
    path: String
}

impl ShaderDescriptor {
    pub fn name(&self) -> String {
        name_from_path(&self.name, &self.path)
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

pub struct StaticShaderDescriptor {
    pub name: &'static str,
    pub data: &'static [u8],
    pub size: usize,
    pub format: &'static str
}

impl StaticShaderDescriptor {
    pub const fn new(name: &'static str, data: &'static [u8], format: &'static str) -> Self {
        Self { name, data, size: data.len(), format }
    }
}

#[derive(Default, Deserialize, Debug, PartialEq)]
#[serde(default, rename = "font", deny_unknown_fields)]
pub struct FontDescriptor {
    name: String,
    charset: String,
    texture: String,
    char_width: u32,
    char_height: u32
}

impl FontDescriptor {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn charset(&self) -> &str {
        &self.charset
    }

    pub fn char_width(&self) -> u32 {
        self.char_width
    }

    pub fn char_height(&self) -> u32 {
        self.char_height
    }

    pub fn texture(&self) -> &str {
        &self.texture
    }
}

pub struct StaticFontDescriptor {
    pub name: &'static str,
    pub charset: &'static str,
    pub char_width: u32,
    pub char_height: u32,
    pub texture: &'static str
}

impl StaticFontDescriptor {
    pub const fn new(name: &'static str, charset: &'static str, char_width: u32, char_height: u32, texture: &'static str) -> Self {
        Self { name, charset, char_width, char_height, texture }
    }
}

#[derive(Default, Deserialize, Debug, PartialEq)]
#[serde(default, rename = "bitmap", deny_unknown_fields)]
pub struct BitmapDescriptor {
    name: String,
    path: String
}

impl BitmapDescriptor {
    pub fn name(&self) -> String {
        name_from_path(&self.name, &self.path)
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

pub struct StaticBitmapDescriptor {
    pub name: &'static str,
    pub data: &'static [u8],
    pub size: usize,
    pub format: &'static str
}

impl StaticBitmapDescriptor {
    pub const fn new(name: &'static str, data: &'static [u8], format: &'static str) -> Self {
        Self { name, data, size: data.len(), format }
    }
}

#[derive(Default, Deserialize, Debug, PartialEq)]
#[serde(default, rename = "material", deny_unknown_fields)]
pub struct MaterialDescriptor {
    pub name: String,
    pub font: String,
    pub texture: String,

    #[serde(default = "default_1")]
    pub texture_binding: u32,

    #[serde(default = "default_true")]
    pub texture_filtering: bool,

    pub vertex_shader: String,
    pub fragment_shader: String,

    #[serde(default = "default_true")]
    pub blending: bool,

    pub blend_mode: String,

    #[serde(default = "default_true")]
    pub backface_culling: bool,

    pub frontface_clockwise: bool,

    pub depth_testing: bool,
    pub depth_writing: bool
}


pub struct StaticMaterialDescriptor {
    pub name: &'static str,
    pub font: &'static str,
    pub texture: &'static str,
    pub texture_binding: u32,
    pub texture_filtering: bool,
    pub vertex_shader: &'static str,
    pub fragment_shader: &'static str,
    pub blending: bool,
    pub blend_mode: &'static str,
    pub backface_culling: bool,
    pub frontface_clockwise: bool,
    pub depth_testing: bool,
    pub depth_writing: bool
}

impl StaticMaterialDescriptor {
    pub const fn new(
        name: &'static str,
        font: &'static str,
        texture: &'static str,
        texture_binding: u32,
        texture_filtering: bool,
        vertex_shader: &'static str,
        fragment_shader: &'static str,
        blending: bool,
        blend_mode: &'static str,
        backface_culling: bool,
        frontface_clockwise: bool,
        depth_testing: bool,
        depth_writing: bool
    ) -> Self {
        Self {
            name,
            font,
            texture,
            texture_binding,
            texture_filtering,
            vertex_shader,
            fragment_shader,
            blending,
            blend_mode,
            backface_culling,
            frontface_clockwise,
            depth_testing,
            depth_writing
        }
    }
}

#[derive(Default, Deserialize, Debug, PartialEq)]
#[serde(default, rename = "task", deny_unknown_fields)]
pub struct TaskDescriptor {
    pub name: String,
    pub id: u32,
    pub interval: u64
}

pub struct StaticTaskDescriptor {
    pub name: &'static str,
    pub id: u32,
    pub interval: u64
}

impl StaticTaskDescriptor {
    pub const fn new(name: &'static str, id: u32, interval: u64) -> Self {
        Self {
            name,
            id,
            interval
        }
    }
}


#[derive(Default, Deserialize, Debug, PartialEq)]
#[serde(default, rename = "music", deny_unknown_fields)]
pub struct MusicDescriptor {
    name: String,
    path: String
}

impl MusicDescriptor {
    pub fn name(&self) -> String {
        name_from_path(&self.name, &self.path)
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

pub struct StaticMusicDescriptor {
    pub name: &'static str,
    pub data: &'static [u8],
    pub size: usize
}

impl StaticMusicDescriptor {
    pub const fn new(name: &'static str, data: &'static [u8]) -> Self {
        Self { name, data, size: data.len() }
    }
}

#[derive(Default, Deserialize, Debug, PartialEq)]
#[serde(default, rename = "sample", deny_unknown_fields)]
pub struct SampleDescriptor {
    name: String,
    path: String
}

impl SampleDescriptor {
    pub fn name(&self) -> String {
        name_from_path(&self.name, &self.path)
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

pub struct StaticSampleDescriptor {
    pub name: &'static str,
    pub data: &'static [u8],
    pub size: usize
}

impl StaticSampleDescriptor {
    pub const fn new(name: &'static str, data: &'static [u8]) -> Self {
        Self { name, data, size: data.len() }
    }
}

/// Application descriptor table
pub struct ApplicationDescriptorTable {
    pub options: &'static StaticOptionsDescriptor,
    pub data: &'static [StaticDataDescriptor],
    pub bitmaps: &'static [StaticBitmapDescriptor],
    pub textures: &'static [StaticTextureDescriptor],
    pub fonts: &'static [StaticFontDescriptor],
    pub shaders: &'static [StaticShaderDescriptor],
    pub materials: &'static [StaticMaterialDescriptor],
    pub tasks: &'static [StaticTaskDescriptor],
    pub music: &'static [StaticSampleDescriptor],
    pub samples: &'static [StaticSampleDescriptor]
}

#[derive(Default, Deserialize, Debug, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Manifest {
    pub options: Option<OptionsDescriptor>,
    pub data: Vec<DataDescriptor>,
    pub bitmaps: Vec<BitmapDescriptor>,
    pub textures: Vec<TextureDescriptor>,
    pub fonts: Vec<FontDescriptor>,
    pub shaders: Vec<ShaderDescriptor>,
    pub materials: Vec<MaterialDescriptor>,
    pub tasks: Vec<TaskDescriptor>,
    pub music: Vec<SampleDescriptor>,
    pub samples: Vec<SampleDescriptor>
}
