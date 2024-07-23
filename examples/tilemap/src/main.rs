//!
//! Gamekit scrolling tile map demo.
//!

#![allow(dead_code)]

gamekit::load!();

use log::{*};
use gamekit::api::{*};
use gamekit::api;


#[repr(C)]
#[derive(Default)]
struct BackgroundShaderParams {
    offset_x: f32,
    offset_y: f32
}

impl BackgroundShaderParams {
    pub fn new_uniform() -> Result<Uniform::<Self>, Error> {
        let mut uniform = Uniform::<Self>::new(0, 0)?;
        {
            let data = uniform.data_mut();
            data.offset_x = 0.0;
            data.offset_y = 0.0;
        }
        Ok(uniform)
    }
}

struct Background {
    frame: Frame,
    material: MaterialLockRef,
    shader_params: Uniform::<BackgroundShaderParams>
}

impl Background {
    pub fn new(material_name: &str) -> Result<Self, Error> {
        let mut frame = Frame::new();
        frame.set_position(-1.0, -1.0);
        frame.set_size(2.0, 2.0);

        let shader_params = BackgroundShaderParams::new_uniform()?;

        let materials = crate::api::materials();
        let material = materials.get(material_name);

        material.lock().unwrap().add_uniform(&shader_params);

        Ok(Self {
            frame,
            material,
            shader_params
        })
    }

    pub fn draw(&mut self, offset_x: f32, offset_y: f32) {

        let renderer = crate::api::renderer_mut();
        renderer.set_material(&self.material);

        let shader_params = self.shader_params.data_mut();
        shader_params.offset_x = offset_x;
        shader_params.offset_y = offset_y;

        self.shader_params.update().unwrap();
        self.material.lock().unwrap().bind_uniforms();

        self.frame.draw();
    }
}

struct ApplicationData {
    map: MapLockRef,
    level: usize,
    scroll_range: Vec2,
    scroll_limits: Vec2,
    scroll_pos: Vec2,
    scroll_speed: Vec2,
    background: Background,
    counter: usize
}

impl Disposable for ApplicationData {
    fn dispose(&mut self) {
    }
}

impl ApplicationData {
    pub fn new() -> Result<Self, Error> {

        let resources = crate::api::resources();
        let metrics = crate::api::metrics();

        let map = resources.get_map("map");
        let level = 0;

        let mut scroll_range = Vec2::new(0.0, 0.0);
        let mut scroll_limits = Vec2::new(0.0, 0.0);

        if let Ok(map) = map.lock() {
            let level = map.get_level(level).unwrap();
            scroll_range.x = level.width as f32;
            scroll_range.y = level.height as f32;
            scroll_limits.x = scroll_range.x - metrics.view_width;
            scroll_limits.y = scroll_range.y - metrics.view_height;
        };

        let scroll_pos = Vec2::new(0.0, scroll_limits.y);
        let scroll_speed = Vec2::new(0.0, 0.0);

        let background = Background::new("background")?;

        Ok(Self {
            map,
            level,
            scroll_range,
            scroll_limits,
            scroll_pos,
            scroll_speed,
            background,
            counter: 0
        })
    }
}

struct App {
    appdata: Option<ApplicationData>
}

impl Disposable for App {
    fn dispose(&mut self) {
        let appdata = self.appdata.as_mut().unwrap();
        appdata.dispose();
    }
}

impl Application for App {
    fn new() -> Result<Self, Error> {
        Ok(Self {
            appdata: Some(ApplicationData::new()?)
        })
    }

    fn on_init(&mut self) {
        trace!("Application::on_init");
    }

    fn on_shutdown(&mut self) {
        trace!("Application::on_shutdown");
    }

    fn on_metrics(&mut self) {
        trace!("Application::on_metrics");
    }

    fn on_update(&mut self) {
        //trace!("Application::on_udpate");

        if self.appdata.is_none() {
            return;
        }

        let appdata = self.appdata.as_mut().unwrap();

        let input = crate::api::input();
        let keyboard_state = input.keyboard_state();

        let tm = crate::api::time();
        let delta = tm.step;

        if let Ok(mut map) = appdata.map.lock() {
            map.animate(appdata.level, delta);

            /*
            if let Ok(level) = map.get_level_mut(0)
                && let Ok(layer) = level.get_layer_mut(0) {
                let t = ((tm.time * 4.0) as i32) % 8;
                layer.set_tile_xy(0, 0, t);
            }
            */
        }

        const ACCELERATION: f32 = 15.0;
        const DECELERATION: f32 = 8.0;
        const FORCE_MAX: f32 = 8.0;
        const FORCE_MIN: f32 = -FORCE_MAX;

        let mut force_x = 0.0;
        let mut force_y = 0.0;

        if (keyboard_state & Input::KEYFLAG_LEFT) != 0 {
            force_x = -ACCELERATION;
        } else if (keyboard_state & Input::KEYFLAG_RIGHT) != 0 {
            force_x = ACCELERATION;
        }

        if (keyboard_state & Input::KEYFLAG_UP) != 0 {
            force_y = -ACCELERATION;
        } else if (keyboard_state & Input::KEYFLAG_DOWN) != 0 {
            force_y = ACCELERATION;
        }

        if force_x != 0.0 {
            appdata.scroll_speed.x = (appdata.scroll_speed.x + delta * force_x).clamp(FORCE_MIN, FORCE_MAX);
        } else {
            appdata.scroll_speed.x *=  (1.0 - delta * DECELERATION).clamp(0.0, 1.0);
        }

        if force_y != 0.0 {
            appdata.scroll_speed.y = (appdata.scroll_speed.y + delta * force_y).clamp(FORCE_MIN, FORCE_MAX);
        } else {
            appdata.scroll_speed.y *=  (1.0 - delta * DECELERATION).clamp(0.0, 1.0);
        }

        appdata.scroll_pos.x += appdata.scroll_speed.x;
        if appdata.scroll_pos.x >= appdata.scroll_limits.x {
            appdata.scroll_pos.x = appdata.scroll_limits.x;
            appdata.scroll_speed.x = 0.0;
        } else if  appdata.scroll_pos.x <= 0.0 {
            appdata.scroll_pos.x = 0.0;
            appdata.scroll_speed.x = 0.0;
        };

        appdata.scroll_pos.y += appdata.scroll_speed.y;
        if appdata.scroll_pos.y >= appdata.scroll_limits.y {
            appdata.scroll_pos.y = appdata.scroll_limits.y;
            appdata.scroll_speed.y = 0.0;
        } else if  appdata.scroll_pos.y <= 0.0 {
            appdata.scroll_pos.y = 0.0;
            appdata.scroll_speed.y = 0.0;
        };


    }

    fn on_draw(&mut self) {
        //trace!("Application::on_draw");

        if self.appdata.is_none() {
            return;
        }

        let renderer = crate::api::renderer_mut();
        let appdata = self.appdata.as_mut().unwrap();

        // set clipping
        let metrics = crate::api::metrics();
        renderer.set_scissor(
            metrics.view_x,
            metrics.view_y,
            metrics.view_width * metrics.view_scaling,
            metrics.view_height * metrics.view_scaling
        );

        // draw background
        let background_offset_x = if appdata.scroll_limits.x != 0.0 { appdata.scroll_pos.x / appdata.scroll_limits.x } else { 0.0 };
        let background_offset_y = if appdata.scroll_limits.y != 0.0 { appdata.scroll_pos.y / appdata.scroll_limits.y } else { 0.0 };
        appdata.background.draw(background_offset_x, background_offset_y);

        appdata.counter += 1;
        if appdata.counter >= 10000 {
            appdata.counter = 0;
        }

        if let Ok(mut map) = appdata.map.lock() {
            map.draw(
                appdata.level,
                -appdata.scroll_pos.x,
                -appdata.scroll_pos.y
            );
        }

        // reset clipping
        renderer.reset_scissor();
    }

}

impl Runnable for App {}

fn main() {
    gamekit::main!();
}
