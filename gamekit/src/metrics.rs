//!
//! Metrics
//!

use log::trace;

use crate::api::Disposable;

pub struct Metrics {
    pub enable_scaling: bool,
    pub width: f32,
    pub height: f32,
    pub window_width: f32,
    pub window_height: f32,
    pub view_width: f32,
    pub view_height: f32,
    pub view_left: f32,
    pub view_top: f32,
    pub view_right: f32,
    pub view_bottom: f32,
    pub scaling: f32
}

impl Disposable for Metrics {
    fn dispose(&mut self) {
        trace!("destroy metrics");
    }
}

impl Metrics {
    pub fn new() -> Self {
        Self::from_options()
    }

    fn from_options() -> Self {
        let options = crate::globals::options();

        let w = (if options.view_width > 0 { options.view_width } else { options.window_width }) as f32;
        let h = (if options.view_height > 0 { options.view_height } else { options.window_height }) as f32;

        Self {
            enable_scaling: options.enable_scaling,
            width: w,
            height: h,
            window_width: options.window_width as f32,
            window_height: options.window_height as f32,
            view_width: w,
            view_height: h,
            view_left: 0.0,
            view_top: 0.0,
            view_right: w,
            view_bottom: h,
            scaling: 1.0
        }
    }

    pub fn set_window_size(&mut self, width: u32, height: u32) -> &mut Self {
        self.window_width = width as f32;
        self.window_height = height as f32;
        self.update();
        self
    }

    fn update(&mut self) {

        if self.enable_scaling {
            self.scaling = (self.window_width / self.width).min(self.window_height / self.height).floor().max(1.0);
        } else {
            self.scaling = 1.0;
            self.width = self.window_width;
            self.height = self.window_height;
        }

        let w = self.width * self.scaling;
        let h = self.height * self.scaling;

        self.view_width = w.min(self.window_width);
        self.view_height = h.min(self.window_height);

        self.view_left = ((self.window_width - self.view_width) / 2.0).floor();
        self.view_right = self.view_left + self.view_width;

        self.view_top = ((self.window_height - self.view_height) / 2.0).floor();
        self.view_bottom = self.view_top + self.view_height;

    }

}
