//!
//! Resources
//!

use gamebuilder::manifest::ApplicationDescriptorTable;
use log::{*};
use std::{collections::HashMap, sync::Mutex};

use crate::{api::Disposable, audio::{Music, MusicLockRef, Sample, SampleLockRef}, bitmap::{Bitmap, BitmapLockRef}, data::{StaticData, StaticDataLockRef}, error::Error, font::{Font, FontLockRef}, maps::{Map, MapLockRef}, shader::{Shader, ShaderLockRef}, texture::{Texture, TextureLockRef}};

#[derive(Default)]
pub struct Resources {
    bitmaps: HashMap<String, BitmapLockRef>,
    textures: HashMap<String, TextureLockRef>,
    shaders: HashMap<String, ShaderLockRef>,
    data: HashMap<String, StaticDataLockRef>,
    fonts: HashMap<String, FontLockRef>,
    music: HashMap<String, MusicLockRef>,
    samples: HashMap<String, SampleLockRef>,
    maps: HashMap<String, MapLockRef>
}

impl Disposable for Resources {
    fn dispose(&mut self) {

        trace!("Resources::dispose");

        for element in self.shaders.values_mut() {
            element.lock().unwrap().dispose();
        }
        self.shaders.clear();

        for element in self.fonts.values_mut() {
            element.lock().unwrap().dispose();
        }
        self.fonts.clear();

        for element in self.textures.values_mut() {
            element.lock().unwrap().dispose();
        }
        self.textures.clear();

        for element in self.bitmaps.values_mut() {
            element.lock().unwrap().dispose();
        }
        self.bitmaps.clear();

        for element in self.data.values_mut() {
            element.lock().unwrap().dispose();
        }
        self.data.clear();

        for element in self.music.values_mut() {
            element.lock().unwrap().dispose();
        }
        self.music.clear();

        for element in self.samples.values_mut() {
            element.lock().unwrap().dispose();
        }
        self.samples.clear();

        for element in self.maps.values_mut() {
            element.lock().unwrap().dispose();
        }
        self.maps.clear();

    }
}

impl Resources {

    pub fn build(descriptors: &'static ApplicationDescriptorTable, stage: usize) -> Result<(), Error> {

        let resources = crate::globals::resources_mut();

        if 0 == stage {

            for descriptor in descriptors.data {
                let res = StaticData::from_resource(descriptor)?;
                let res_ref = StaticDataLockRef::new(Mutex::new(res));
                resources.data.insert(String::from(descriptor.name), res_ref);
            }

            for descriptor in descriptors.bitmaps {
                let res = Bitmap::from_resource(descriptor)?;
                let res_ref = BitmapLockRef::new(Mutex::new(res));
                resources.bitmaps.insert(String::from(descriptor.name), res_ref);
            }

            for descriptor in descriptors.textures {
                let res = Texture::from_resource(descriptor)?;
                let res_ref = TextureLockRef::new(Mutex::new(res));
                resources.textures.insert(String::from(descriptor.name), res_ref);
            }

            for descriptor in descriptors.fonts {
                let res = Font::from_resource(descriptor)?;
                let res_ref = FontLockRef::new(Mutex::new(res));
                resources.fonts.insert(String::from(descriptor.name), res_ref);
            }

            for descriptor in descriptors.shaders {
                let res = Shader::from_resource(descriptor)?;
                let res_ref = ShaderLockRef::new(Mutex::new(res));
                resources.shaders.insert(String::from(descriptor.name), res_ref);
            }

            for descriptor in descriptors.music {
                let res = Music::from_resource(descriptor)?;
                #[allow(clippy::arc_with_non_send_sync)] 
                let res_ref = MusicLockRef::new(Mutex::new(res));
                resources.music.insert(String::from(descriptor.name), res_ref);
            }

            for descriptor in descriptors.samples {
                let res = Sample::from_resource(descriptor)?;
                let res_ref = SampleLockRef::new(Mutex::new(res));
                resources.samples.insert(String::from(descriptor.name), res_ref);
            }
        } else if 1 == stage {

            // depends on materials to be built

            for descriptor in descriptors.maps {
                let res = Map::from_resource(descriptor)?;
                let res_ref = MapLockRef::new(Mutex::new(res));
                resources.maps.insert(String::from(descriptor.name), res_ref);
            }

        }

        Ok(())

    }

    pub fn get_shader(&self, id: &str) -> ShaderLockRef {
        let res_ref = self.shaders.get(id).unwrap_or_else(|| panic!("shader not found: \"{id}\""));
        res_ref.clone()
    }

    pub fn get_bitmap(&self, id: &str) -> BitmapLockRef {
        let res_ref = self.bitmaps.get(id).unwrap_or_else(|| panic!("bitmap not found: \"{id}\""));
        res_ref.clone()
    }

    pub fn get_texture(&self, id: &str) -> TextureLockRef {
        let res_ref = self.textures.get(id).unwrap_or_else(|| panic!("texture not found: \"{id}\""));
        res_ref.clone()
    }

    pub fn get_font(&self, id: &str) -> FontLockRef {
        let res_ref = self.fonts.get(id).unwrap_or_else(|| panic!("font not found: \"{id}\""));
        res_ref.clone()
    }

    pub fn get_data(&self, id: &str) -> StaticDataLockRef {
        let res_ref = self.data.get(id).unwrap_or_else(|| panic!("data not found: \"{id}\""));
        res_ref.clone()
    }

    pub fn get_music(&self, id: &str) -> MusicLockRef {
        let res_ref = self.music.get(id).unwrap_or_else(|| panic!("music not found: \"{id}\""));
        res_ref.clone()
    }

    pub fn get_sample(&self, id: &str) -> SampleLockRef {
        let res_ref = self.samples.get(id).unwrap_or_else(|| panic!("sample not found: \"{id}\""));
        res_ref.clone()
    }

    pub fn get_map(&self, id: &str) -> MapLockRef {
        let res_ref = self.maps.get(id).unwrap_or_else(|| panic!("map not found: \"{id}\""));
        res_ref.clone()
    }

}
