//!
//! Options
//!

use crate::{constants::Constants, manifest::StaticOptionsDescriptor};

pub struct ScalingMode {}

impl ScalingMode {
    pub const DISABLED: i32 = 0;
    pub const SCALE: i32 = 1;
    pub const ZOOM: i32 = 2;
    pub const CENTER: i32 = 3;
    pub const RESIZE: i32 = 4;

    pub fn from_string(mode: &str) -> i32 {
        let mode_str = mode.to_lowercase();
        match mode_str.as_str() {
            "scale" => { ScalingMode::SCALE },
            "zoom" => { ScalingMode::ZOOM },
            "center" => { ScalingMode::CENTER },
            "resize" => { ScalingMode::RESIZE },
            _ => { ScalingMode::DISABLED }
        }
    }
}

/// Options
#[derive(Clone, Debug)]
pub struct Options {
    pub title: String,
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

impl Default for Options {
    fn default() -> Self {
        Self {
            title: String::from("gamekit"),
            window_x: i32::MAX,
            window_y: i32::MAX,
            window_width: 400,
            window_height: 300,
            view_width: 0,
            view_height: 0,
            scaling_mode: ScalingMode::DISABLED,
            fps: Constants::DEFAULT_FPS,
            show_statistics: false,
            queue_size: Constants::DEFAULT_BLITTER_BATCH_CAPACITY,
            headless: false,
            enable_validation_layer: Constants::ENABLE_VALIDATION_LAYER,
            enable_api_dump_layer: Constants::ENABLE_API_DUMP_LAYER
        }
    }
}

impl Options {

    pub fn from_static(descriptor: &'static StaticOptionsDescriptor) -> Self {
        Self {
            title: descriptor.title.to_string(),
            window_x: descriptor.window_x,
            window_y: descriptor.window_y,
            window_width: descriptor.window_width,
            window_height: descriptor.window_height,
            view_width: descriptor.view_width,
            view_height: descriptor.view_height,
            scaling_mode: descriptor.scaling_mode,
            fps: descriptor.fps,
            show_statistics: descriptor.show_statistics,
            queue_size: if descriptor.queue_size > 0 { descriptor.queue_size } else { Constants::DEFAULT_BLITTER_BATCH_CAPACITY },
            headless: descriptor.headless,
            enable_validation_layer: descriptor.enable_validation_layer,
            enable_api_dump_layer: descriptor.enable_api_dump_layer
        }
    }

    pub fn set_title(&mut self, title: &str) -> &mut Self {
        self.title = title.to_string();
        self
    }

    pub fn set_window_size(&mut self, width: u32, height: u32) -> &mut Self {
        self.window_width = width;
        self.window_height = height;
        self
    }

    pub fn set_scaling_mode(&mut self, scaling_mode: i32) -> &mut Self {
        self.scaling_mode = scaling_mode;
        self
    }

    pub fn set_show_statistics(&mut self, show_statistics: bool) -> &mut Self {
        self.show_statistics = show_statistics;
        self
    }

    pub fn set_window_position(&mut self, x: i32, y: i32) -> &mut Self {
        self.window_x = x;
        self.window_y = y;
        self
    }

    pub fn set_view_size(&mut self, width: u32, height: u32) -> &mut Self {
        self.view_width = width;
        self.view_height = height;
        self
    }

    pub fn set_fps(&mut self, fps: u32) -> &mut Self {
        self.fps = fps;
        self
    }

    pub fn set_headless(&mut self, headless: bool) -> &mut Self {
        self.headless = headless;
        self
    }

}
