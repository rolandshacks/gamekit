//!
//! Audio
//!

use crate::api::{Disposable, LockRef};
use crate::error::Error;
use crate::manifest::StaticSampleDescriptor;

extern crate sdl2;

use log::{*};

fn volume_as_i32(volume: f32) -> i32 {
	let v = ((volume * 128.9f32) as i32).clamp(0, 128);
	return v;
}

pub struct MixerChannel {
    channel: u32
}

impl MixerChannel {
    pub fn from(channel: u32) -> Self {
        Self {
            channel
        }
    }

    pub fn as_raw(&self) -> u32 {
        self.channel
    }
}

pub struct Sample {
    obj: sdl2::mixer::Chunk
}

unsafe impl Send for Sample {}

pub type SampleRef = std::sync::Arc<Sample>;
pub type SampleLockRef = LockRef<Sample>;

impl Disposable for Sample {
    fn dispose(&mut self) {
    }
}

impl Sample {
    pub fn from_file(name: &str) -> Result<Self, Error> {
        let obj = sdl2::mixer::Chunk::from_file(name).unwrap();
        Ok(Self {
            obj
        })
    }

    pub fn from_resource(descriptor: &StaticSampleDescriptor) -> Result<Self, Error> {
        Self::from_memory(descriptor.data)
    }

    pub fn from_memory_raw(data: &[u8]) -> Result<Self, Error> {
        let data_ptr = data.as_ptr() as *mut std::ffi::c_uchar;
        let data_size = data.len() as u32;
        let raw = unsafe { sdl2::sys::mixer::Mix_QuickLoad_RAW(data_ptr, data_size) };
        let obj = sdl2::mixer::Chunk { raw, owned: false };
        Ok(Self { obj })
    }

    pub fn from_memory(data: &[u8]) -> Result<Self, Error> {
        let data_ptr = data.as_ptr() as *mut std::ffi::c_void;
        let data_size = data.len() as std::ffi::c_int;

        let rw = unsafe { sdl2::sys::SDL_RWFromConstMem(data_ptr, data_size) };

        if rw.is_null() {
            return Err(Error::from("could not load sample"));
        }

        let raw = unsafe { sdl2::sys::mixer::Mix_LoadWAV_RW(rw, 0) };

        if raw.is_null() {
            return Err(Error::from("could not decode sample"))
        }

        Ok(Self { obj: sdl2::mixer::Chunk {
            raw,
            owned: false
        }})

    }

}

pub struct Music {
    obj: sdl2::mixer::Music<'static>
}

pub type MusicRef = std::sync::Arc<Music>;
pub type MusicLockRef = LockRef<Music>;

impl Disposable for Music {
    fn dispose(&mut self) {
    }
}

impl Music {
    pub fn from_file(name: &str) -> Result<Self, Error> {
        let obj = sdl2::mixer::Music::from_file(name)?;
        Ok(Self { obj })
    }

    pub fn from_resource(descriptor: &StaticSampleDescriptor) -> Result<Self, Error> {
        Self::from_memory(descriptor.data)
    }

    pub fn from_memory(data: &[u8]) -> Result<Self, Error> {
        let data_ptr = data.as_ptr() as *const std::ffi::c_uchar;
        let data_size = data.len();
        let sample_data = unsafe { core::slice::from_raw_parts::<u8>(data_ptr, data_size) };

        let obj = sdl2::mixer::Music::from_static_bytes(sample_data)?;

        Ok(Self { obj })
    }
}

pub struct AudioChannel {
    obj: sdl2::mixer::Channel
}

impl AudioChannel {
    pub fn from(channel: sdl2::mixer::Channel) -> Self {
        Self {
            obj: channel
        }
    }
}

pub struct Audio {
    audio_subsystem: sdl2::AudioSubsystem
}

impl Disposable for Audio {
    fn dispose(&mut self) {
        trace!("Audio::dispose");
        sdl2::mixer::close_audio();
    }
}

impl Audio {
    pub fn new() -> Result<Self, Error> {

        //let options = crate::globals::options();
        let instance = crate::globals::instance();

        let sdl = &instance.sdl;
        let audio_subsystem = sdl.audio()?;

        sdl2::mixer::open_audio(44100, sdl2::mixer::DEFAULT_FORMAT, 2, 1024)?;

        trace!("initialized audio subsystem");

        Ok(Self {
            audio_subsystem
        })
    }

    pub fn play_sample(&self, sample: &SampleLockRef, channel: i32, volume: f32) -> Result<AudioChannel, Error> {

        let requested_channel = sdl2::mixer::Channel(channel);

        let playback_channel = requested_channel.play(&sample.lock().unwrap().obj, 0)?;

        playback_channel.set_volume(volume_as_i32(volume));

        Ok(AudioChannel::from(playback_channel))

    }

    pub fn stop_sample(&self, channel: &AudioChannel) {
        channel.obj.halt();
    }

    pub fn play_music(&self, music: &MusicLockRef, volume: f32) {
        music.lock().unwrap().obj.play(-1).unwrap();
        sdl2::mixer::Music::set_volume(volume_as_i32(volume));
    }

    pub fn stop_music(&self) {
        sdl2::mixer::Music::pause();
    }
}
