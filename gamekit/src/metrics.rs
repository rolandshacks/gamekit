//!
//! Metrics
//!

use log::trace;

use crate::{api::Disposable, options::ScalingMode};

pub struct Metrics {
    pub scaling_mode: i32,
    pub window_width: f32,
    pub window_height: f32,
    pub view_width: f32,
    pub view_height: f32,
    pub view_x: f32,
    pub view_y: f32,
    pub view_scaling: f32,
}

impl Disposable for Metrics {
    fn dispose(&mut self) {
        trace!("Metrics::dispose");
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
            scaling_mode: options.scaling_mode,
            window_width: options.window_width as f32,
            window_height: options.window_height as f32,
            view_width: w,
            view_height: h,
            view_x: 0.0,
            view_y: 0.0,
            view_scaling: 1.0
        }
    }

    pub fn set_window_size(&mut self, width: u32, height: u32) -> &mut Self {
        self.window_width = width as f32;
        self.window_height = height as f32;
        self.update();
        self
    }

    fn update(&mut self) {

        match self.scaling_mode {
            ScalingMode::DISABLED => {
                // do nothing, view is fixed at (0,0)
                self.view_scaling = 1.0;
                self.view_x = 0.0;
                self.view_y = 0.0;
            },
            ScalingMode::CENTER => {
                // center view, keep size
                self.view_scaling = 1.0;
                self.view_x = ((self.window_width - self.view_width) / 2.0).floor();
                self.view_y = ((self.window_height - self.view_height) / 2.0).floor();
            },
            ScalingMode::RESIZE => {
                // resize view to window size, keep ratio
                let scale_x = self.window_width / self.view_width;
                let scale_y = self.window_height / self.view_height;
                self.view_scaling = scale_x.min(scale_y);
                self.view_x = ((self.window_width - self.view_width * self.view_scaling) / 2.0).floor();
                self.view_y = ((self.window_height - self.view_height * self.view_scaling) / 2.0).floor();
            },
            ScalingMode::ZOOM => {
                // zoom pixels of view size in integer steps, center view
                let scale_x = self.window_width / self.view_width;
                let scale_y = self.window_height / self.view_height;
                self.view_scaling = scale_x.min(scale_y).floor().max(1.0);
                self.view_x = ((self.window_width - self.view_width * self.view_scaling) / 2.0).floor();
                self.view_y = ((self.window_height - self.view_height * self.view_scaling) / 2.0).floor();
            },
            _ => {}
        };

    }

}
