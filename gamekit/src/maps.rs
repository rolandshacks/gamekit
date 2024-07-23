//!
//! Tile Map
//! Based on LDtk (https://ldtk.io)
//!

use std::collections::HashMap;

use gamebuilder::manifest::StaticMapDescriptor;
use log::{*};

use crate::animator::{Animator, AnimatorMode};
use crate::api::{Disposable, LockRef};
use crate::buffer::Uniform;
use crate::error::Error;

use crate::material::MaterialLockRef;
use crate::primitives::{TileQueue};
use crate::thirdparty::LdtkJson::{LayerInstance, LdtkJson, Level, TilesetDefinition};

use serde::Deserialize;

const TRANSPARENT_TILE_ID: i32 = i32::MAX;

fn default_1_0f32() -> f32 { 1.0 }

#[derive(Default, Deserialize, Debug, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct MapMetaData {
    pub animation_frames: Option<Vec<u32>>,

    #[serde(default = "default_1_0f32")]
    pub animation_step: f32,

    pub animation_mode: String
}

pub struct TileAnimation {
    pub tile_id: u32,
    pub frames: Vec<u32>,
    pub animator: Animator,
    pub current_frame: u32
}

impl TileAnimation {
    pub fn new(tile_id: u32, frames: &[u32], mode: AnimatorMode, step: f32) -> Self {
        let start = 0.0f32;
        let range = frames.len() as f32;
        let end = f32::max(start, range - range * f32::EPSILON);
        let animator = Animator::new(start, end, start, step, mode);
        let current_frame = if !frames.is_empty() { frames[0] } else { 0u32 };

        Self {
            tile_id,
            frames: frames.to_vec(),
            animator,
            current_frame
        }
    }

    pub fn update(&mut self, delta: f32) {
        if self.frames.is_empty() { return; }
        self.animator.update(delta);
        self.current_frame = self.frames[self.animator.value as usize];
    }
}

pub struct TileAnimations {
    animations: Vec<TileAnimation>,
    animation_map: HashMap<u32, usize>
}

impl Default for TileAnimations {
    fn default() -> Self {
        TileAnimations::new()
    }
}

impl TileAnimations {
    pub fn new() -> Self {
        Self {
            animations: Vec::<TileAnimation>::new(),
            animation_map: HashMap::new()
        }
    }

    pub fn push(&mut self, animation: TileAnimation) {
        let tile_id = animation.tile_id;
        let index = self.animations.len();
        self.animations.push(animation);
        self.animation_map.insert(tile_id, index);
    }

    pub fn contains_key(&self, tile_id: u32) -> bool {
        self.animation_map.contains_key(&tile_id)
    }

    pub fn get_index(&self, tile_id: u32) -> Option<&usize> {
        self.animation_map.get(&tile_id)
    }

    pub fn get(&self, tile_id: u32) -> Option<&TileAnimation> {
        match self.animation_map.get(&tile_id) {
            Some(index) => Some(&self.animations[*index]),
            None => None
        }
    }

    pub fn update(&mut self, delta: f32) {
        for animation in &mut self.animations {
            animation.update(delta);
        }
    }
    
    fn store_to(&self, buffer: &mut[u32]) {
        for (index, animation) in self.animations.iter().enumerate().take(buffer.len()) {
            buffer[index] = animation.current_frame;
        }
    }
}

#[repr(C)]
#[derive(Default)]
struct MapShaderParams {
    offset_left: f32,
    offset_top: f32,
    window_width: f32,
    window_height: f32,
    view_width: f32,
    view_height: f32,
    view_x: f32,
    view_y: f32,
    view_scaling: f32,
    texture_width: u32,
    texture_height: u32,
    grid_size: u32,
    map_rows: u32,
    map_cols: u32
}

impl MapShaderParams {
    pub fn new(texture_width: u32, texture_height: u32, grid_size: u32) -> Result<Uniform::<Self>, Error> {
        let metrics = crate::api::metrics();

        let mut uniform = Uniform::<Self>::new(0, 0)?;
        {
            let data = uniform.data_mut();
            data.window_width = metrics.window_width;
            data.window_height = metrics.window_height;
            data.view_width = metrics.view_width;
            data.view_height = metrics.view_height;
            data.view_x = metrics.view_x;
            data.view_y = metrics.view_y;
            data.view_scaling = metrics.view_scaling;

            data.offset_left = 0.0;
            data.offset_top = 0.0;

            data.texture_width = texture_width;
            data.texture_height = texture_height;
            data.grid_size = grid_size;
            data.map_rows = 0;
            data.map_cols = 0;
        }

        Ok(uniform)
    }

}

#[repr(C)]
struct MapShaderTileLookupBuffer {
    animation_frames: [u32; 256]
}

impl Default for MapShaderTileLookupBuffer {
    fn default() -> Self {
        Self {
            animation_frames: [0u32; 256]
        }
    }
}

impl MapShaderTileLookupBuffer {
    pub fn new() -> Result<Uniform::<Self>, Error> {
        Uniform::<Self>::new(2, 0)
    }
}

pub struct MapLayer {
    tile_queue: TileQueue,
    rows: usize,
    cols: usize,
    width: usize,
    height: usize,
    visible: bool,
    random_access: bool
}

impl Disposable for MapLayer {
    fn dispose(&mut self) {
        self.tile_queue.dispose();
    }
}

impl MapLayer {
    fn new(layer: &LayerInstance, _tileset: &TilesetDefinition, animations: &TileAnimations, random_access: bool) -> Self {

        let data = if layer.layer_instance_type == "IntGrid" {
            &layer.auto_layer_tiles
        } else {
            &layer.grid_tiles
        };

        let grid_size = layer.grid_size as usize;
        let cols = layer.c_wid as usize;
        let rows = layer.c_hei as usize;
        let width = cols * grid_size;
        let height = rows * grid_size;
        let num_tiles = if random_access { cols * rows } else { data.len() };

        let mut tile_queue = TileQueue::new(num_tiles);

        tile_queue.begin();

            if random_access {
                for pos_index in 0..num_tiles {
                    // initialize with transparent tiles
                    tile_queue.push(pos_index as u32, TRANSPARENT_TILE_ID);
                }
            }

            for t in data {

                let alpha = t.a;
                if alpha == 0.0 {
                    continue; // transparent
                } 

                let col = t.px[0] as usize / grid_size;
                let row = t.px[1] as usize / grid_size;
                let pos_index = row * cols + col;

                let map_tile_index = t.t as u32;
                let tile_index = if let Some(animation_index) = animations.get_index(map_tile_index) {
                    -(*animation_index as i32) - 1
                } else {
                    map_tile_index as i32
                };

                if random_access {
                    // write into buffer
                    tile_queue.set_value(pos_index, pos_index as u32, tile_index);
                } else {
                    // append to buffer
                    tile_queue.push(pos_index as u32, tile_index);
                }

            }

        tile_queue.end();

        Self {
            tile_queue,
            rows,
            cols,
            width,
            height,
            visible: true,
            random_access
        }
    }

    pub fn draw(&mut self) {
        self.tile_queue.draw();
    }

    pub fn set_index(&mut self, vertex_index: usize, pos_index: usize, tile_index: i32) {
        self.tile_queue.set_value(vertex_index, pos_index as u32, tile_index);
    }

    pub fn set_tile(&mut self, vertex_index: usize, tile_index: i32) {
        self.tile_queue.set_tile(vertex_index, tile_index);
    }

    pub fn set_tile_xy(&mut self, x: usize, y: usize, tile_index: i32) {
        self.set_tile(y * self.cols + x, tile_index);
    }

    pub fn get_tile(&self, vertex_index: usize) -> i32 {
        self.tile_queue.get_tile(vertex_index)
    }

    pub fn get_tile_xy(&self, x: usize, y: usize) -> i32 {
        self.get_tile(y * self.cols + x)
    }

}

pub struct MapLevel {
    pub layers: Vec<MapLayer>,
    pub animations: TileAnimations,
    pub rows: usize,
    pub cols: usize,
    pub width: usize,
    pub height: usize,
}

impl Disposable for MapLevel {
    fn dispose(&mut self) {
        for layer in &mut self.layers {
            layer.dispose();
        }
        self.layers.clear();
    }
}

impl MapLevel {

    pub fn new(level: &Level, tileset: &TilesetDefinition, uncompressed_layers: &[&str]) -> Result<Self, Error> {

        let mut rows: usize = 0;
        let mut cols: usize = 0;
        let mut width: usize = 0;
        let mut height: usize = 0;
        let mut map_layers = Vec::new();
        let mut animations = TileAnimations::default();

        // process map meta data
        let custom_data = &tileset.custom_data;
        for meta_data in custom_data {
            let tile_id = meta_data.tile_id;
            let json = &meta_data.data;

            if !json.is_empty() {
                let meta: MapMetaData = match json5::from_str(json.as_str()) {
                    Ok(manifest) => manifest,
                    Err(e) => {
                        trace!("failed to load custom map data: {}", e);
                        return Err(Error::from("failed to load custom map data"));
                    }
                };

                // processing animated tiles
                if let Some(frames) = meta.animation_frames {
                    trace!("adding animation for tile {}", tile_id);
                    animations.push(
                        TileAnimation::new(
                            tile_id as u32,
                            &frames,
                            AnimatorMode::from_string(&meta.animation_mode),
                            meta.animation_step
                        )
                    );
                }
            }

        }

        let layers = level.layer_instances.as_ref().unwrap();

        for layer in layers {

            let identifier = &layer.identifier;
            let uncompressed = uncompressed_layers.contains(&identifier.as_str());

            let mut map_layer = MapLayer::new(layer, tileset, &animations, uncompressed);
            map_layer.visible = layer.visible;

            rows = rows.max(map_layer.rows);
            cols = cols.max(map_layer.cols);

            width = width.max(map_layer.width);
            height = height.max(map_layer.height);

            map_layers.push(map_layer);
        }

        Ok(Self {
            layers: map_layers,
            animations,
            rows,
            cols,
            width,
            height
        })
    }

    pub fn get_layer(&self, layer: usize) -> Result<&MapLayer, Error> {
        if layer >= self.layers.len() {
            return Err(Error::from("layer index out of range"));
        }

        Ok(&self.layers[layer])
    }    

    pub fn get_layer_mut(&mut self, layer: usize) -> Result<&mut MapLayer, Error> {
        if layer >= self.layers.len() {
            return Err(Error::from("layer index out of range"));
        }

        Ok(&mut self.layers[layer])
    }    

    pub fn animate(&mut self, delta: f32) {
        self.animations.update(delta);
    }

    pub fn draw(&mut self) {
        for layer in &mut self.layers.iter_mut().rev() {
            if layer.visible {
                layer.draw();
            }
        }
    }
    
    fn store_animations_to(&mut self, buffer: &mut [u32]) {
        self.animations.store_to(buffer);
    }

}

pub struct Map {
    pub obj: LdtkJson,
    pub levels: Vec<MapLevel>,
    pub material: Option<MaterialLockRef>,
    shader_params: Uniform::<MapShaderParams>,
    shader_lookup_buffer: Uniform::<MapShaderTileLookupBuffer>
}

unsafe impl Send for Map {}

pub type MapRef = std::sync::Arc<Map>;
pub type MapLockRef = LockRef<Map>;

impl Disposable for Map {
    fn dispose(&mut self) {
        for level in &mut self.levels {
            level.dispose();
        }
        self.levels.clear();
    }
}

impl Map {
    pub fn get_level(&self, level: usize) -> Result<&MapLevel, Error> {
        if level >= self.levels.len() {
            return Err(Error::from("level index out of range"));
        }

        Ok(&self.levels[level])
    }

    pub fn get_level_mut(&mut self, level: usize) -> Result<&mut MapLevel, Error> {
        if level >= self.levels.len() {
            return Err(Error::from("level index out of range"));
        }

        Ok(&mut self.levels[level])
    }

    pub fn from_file(name: &str, uncompressed_layers: &[&str]) -> Result<Self, Error> {

        let file = match std::fs::File::open(name) {
            Ok(file) => file,
            Err(_e) => {
                return Err(Error::from(format!("failed to load map from file \"{name}\"")));
            }
        };

        let reader = std::io::BufReader::new(file);

        let obj: LdtkJson = match serde_json::from_reader(reader) {
            Ok(obj) => obj,
            Err(_e) => {
                return Err(Error::from("failed to load map"));
            }
        };

        Self::from_ldtk(obj, uncompressed_layers)
    }

    pub fn from_resource(descriptor: &StaticMapDescriptor) -> Result<Self, Error> {
        let mut map = Self::from_memory(descriptor.data, descriptor.uncompressed_layers)?;

        if !descriptor.material.is_empty() {
            let materials = crate::api::materials();
            let material_ref = materials.get(descriptor.material);
            map.set_material(&material_ref);
        }

        Ok(map)
    }

    pub fn from_memory(data: &[u8], uncompressed_layers: &[&str]) -> Result<Self, Error> {

        let obj: LdtkJson = match serde_json::from_slice(data) {
            Ok(obj) => obj,
            Err(_e) => {
                return Err(Error::from("failed to load map"));
            }
        };

        Self::from_ldtk(obj, uncompressed_layers)
    }

    pub fn set_material(&mut self, material_ref: &MaterialLockRef) {
        material_ref.lock().unwrap().add_uniform(&self.shader_params);
        material_ref.lock().unwrap().add_uniform(&self.shader_lookup_buffer);
        self.material = Some(material_ref.clone());
    }

    pub fn from_ldtk(ldtk: LdtkJson, uncompressed_layers: &[&str]) -> Result<Self, Error> {

        let tilesets = &ldtk.defs.tilesets;
        let tileset = &tilesets[0];

        let shader_params = MapShaderParams::new(
            tileset.px_wid as u32,
            tileset.px_hei as u32,
            tileset.tile_grid_size as u32
        )?;

        let shader_lookup_buffer = MapShaderTileLookupBuffer::new()?;

        let mut map_levels = Vec::new();

        let levels = &ldtk.levels;
        for level in levels {
            let map_level = MapLevel::new(level, tileset, uncompressed_layers)?;
            map_levels.push(map_level);
        }

        Ok(Self {
            obj: ldtk,
            levels: map_levels,
            material: None,
            shader_params,
            shader_lookup_buffer
        })
    }

    pub fn animate(&mut self, level_index: usize, delta: f32) {
        if level_index >= self.levels.len() {
            return;
        }

        let level = &mut self.levels[level_index];
        level.animate(delta);
    }

    pub fn draw(&mut self, level_index: usize, offset_x: f32, offset_y: f32) {

        if level_index >= self.levels.len() {
            return;
        }

        let level = &mut self.levels[level_index];

        let renderer = crate::api::renderer_mut();

        if self.material.is_some() {
            renderer.set_material(self.material.as_ref().unwrap());
        }

        let shader_params = self.shader_params.data_mut();
        shader_params.offset_left = offset_x;
        shader_params.offset_top = offset_y;
        shader_params.map_rows = level.rows as u32;
        shader_params.map_cols = level.cols as u32;

        // window metrics
        let metrics = crate::api::metrics();
        shader_params.window_width = metrics.window_width;
        shader_params.window_height = metrics.window_height;
        shader_params.view_width = metrics.view_width;
        shader_params.view_height = metrics.view_height;
        shader_params.view_x = metrics.view_x;
        shader_params.view_y = metrics.view_y;
        shader_params.view_scaling = metrics.view_scaling;

        self.shader_params.update().unwrap();

        let shader_lookup_buffer = self.shader_lookup_buffer.data_mut();
        level.store_animations_to(&mut shader_lookup_buffer.animation_frames);
        self.shader_lookup_buffer.update().unwrap();

        if self.material.is_some() {
            self.material.as_ref().unwrap().lock().unwrap().bind_uniforms();
        }

        level.draw();

    }

}
