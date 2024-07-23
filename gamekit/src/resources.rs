//!
//! Resources
//!

use log::{*};
use std::{collections::HashMap, sync::Mutex};

use crate::{api::Disposable, audio::Music, audio::MusicLockRef, audio::Sample, audio::SampleLockRef, bitmap::{Bitmap, BitmapLockRef}, compiler::ApplicationDescriptorTable, data::{StaticData, StaticDataLockRef}, error::Error, font::{Font, FontLockRef}, shader::{Shader, ShaderLockRef}, texture::{Texture, TextureLockRef}};

pub struct Resources {
    bitmaps: HashMap<String, BitmapLockRef>,
    textures: HashMap<String, TextureLockRef>,
    shaders: HashMap<String, ShaderLockRef>,
    data: HashMap<String, StaticDataLockRef>,
    fonts: HashMap<String, FontLockRef>,
    music: HashMap<String, MusicLockRef>,
    samples: HashMap<String, SampleLockRef>,
}

impl Disposable for Resources {
    fn dispose(&mut self) {

        trace!("Resources::dispose");

        for (_, element) in &mut self.shaders {
            element.lock().unwrap().dispose();
        }
        self.shaders.clear();

        for (_, element) in &mut self.fonts {
            element.lock().unwrap().dispose();
        }
        self.fonts.clear();

        for (_, element) in &mut self.textures {
            element.lock().unwrap().dispose();
        }
        self.textures.clear();

        for (_, element) in &mut self.bitmaps {
            element.lock().unwrap().dispose();
        }
        self.bitmaps.clear();

        for (_, element) in &mut self.data {
            element.lock().unwrap().dispose();
        }
        self.data.clear();

        for (_, element) in &mut self.music {
            element.lock().unwrap().dispose();
        }
        self.music.clear();

        for (_, element) in &mut self.samples {
            element.lock().unwrap().dispose();
        }
        self.samples.clear();


    }
}

impl Default for Resources {
    fn default() -> Self {
        Self {
            shaders: HashMap::new(),
            fonts: HashMap::new(),
            textures: HashMap::new(),
            bitmaps: HashMap::new(),
            data: HashMap::new(),
            music: HashMap::new(),
            samples: HashMap::new()
        }
    }
}

impl Resources {

    pub fn build(descriptors: &'static ApplicationDescriptorTable) -> Result<(), Error> {

        let resources = crate::globals::resources_mut();

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
            let res_ref = MusicLockRef::new(Mutex::new(res));
            resources.music.insert(String::from(descriptor.name), res_ref);
        }

        for descriptor in descriptors.samples {
            let res = Sample::from_resource(descriptor)?;
            let res_ref = SampleLockRef::new(Mutex::new(res));
            resources.samples.insert(String::from(descriptor.name), res_ref);
        }

        Ok(())

    }

    pub fn get_shader(&self, id: &str) -> ShaderLockRef {
        let res_ref = self.shaders.get(id).expect("shader not found");
        return res_ref.clone();
    }

    pub fn get_bitmap(&self, id: &str) -> BitmapLockRef {
        let res_ref = self.bitmaps.get(id).expect("bitmap not found");
        return res_ref.clone();
    }

    pub fn get_texture(&self, id: &str) -> TextureLockRef {
        let res_ref = self.textures.get(id).expect("texture not found");
        return res_ref.clone();
    }

    pub fn get_font(&self, id: &str) -> FontLockRef {
        let res_ref = self.fonts.get(id).expect("font not found");
        return res_ref.clone();
    }

    pub fn get_data(&self, id: &str) -> StaticDataLockRef {
        let res_ref = self.data.get(id).expect("data not found");
        return res_ref.clone();
    }

    pub fn get_music(&self, id: &str) -> MusicLockRef {
        let res_ref = self.music.get(id).expect("music not found");
        return res_ref.clone();
    }

    pub fn get_sample(&self, id: &str) -> SampleLockRef {
        let res_ref = self.samples.get(id).expect("sample not found");
        return res_ref.clone();
    }

}
