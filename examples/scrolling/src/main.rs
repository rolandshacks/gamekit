//!
//! Gamekit scrolling tile map demo.
//!

#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/manifest.rs"));

use log::{*};
use gamekit::api::{*};
use gamekit::api;

mod tilemap;

#[repr(C)]
#[derive(Default)]
struct ShaderParams {
    time: f32,
    time_delta: f32,
    frame: i32,
    offset_left: f32,
    offset_top: f32,
    window_width: f32,
    window_height: f32,
    view_width: f32,
    view_height: f32,
    view_x: f32,
    view_y: f32,
    view_scaling: f32
}

struct ApplicationData {
    background_material: MaterialLockRef,
    background: Quad,
    tilemap_material: MaterialLockRef,
    tilemap: VertexQueue,
    tilemap_params: Uniform::<ShaderParams>,
    tilemap_scroll: Vec2,
    tilemap_width: usize,
    tilemap_height: usize
}

impl Disposable for ApplicationData {
    fn dispose(&mut self) {
        self.tilemap.dispose();
    }
}

impl ApplicationData {
    pub fn new() -> Result<Self, Error> {
        let metrics = crate::api::metrics();
        let resources = crate::api::resources();

        /////

        let shader_params = Uniform::<ShaderParams>::new(0, 1)?;

        /////

        let materials = crate::api::materials();
        let background_material_ref = materials.get("background");
        background_material_ref.lock().unwrap()
            .add_uniform(&shader_params);

        let tilemap_material_ref = materials.get("tilemap");
        tilemap_material_ref.lock().unwrap()
            .add_uniform(&shader_params);

        /////

        let tilemap_texture = resources.get_texture("tiles");

        let tilemap_width;
        let tilemap_height;

        let tilemap = {
            let tilemap_data = tilemap::TILEMAP;

            let num_tiles = tilemap_data.len();
            let rows = 7usize;
            let tiles_per_row = num_tiles / rows;
            let cols = (metrics.view_width as usize + 7) / 8 + tiles_per_row + 1;

            tilemap_width = cols * 8;
            tilemap_height = rows * 8;

            // this includes repeating tiles to enable seamless scrolling
            let num_tiles_absolute = rows * cols;

            let mut tilemap = VertexQueue::new(num_tiles_absolute);

            tilemap.begin();

            let w = 8.0f32;
            let h = 8.0f32;

            let texture = tilemap_texture.lock().unwrap();

            let tile_rows = texture.height / 8;
            let tile_cols = texture.width / 8;

            let th = 1.0f32 / (tile_rows as f32);
            let tw = 1.0f32 / (tile_cols as f32);

            for row in 0..rows {
                let y = (row as f32) * w;

                let ofs = row * tiles_per_row;

                for col in 0..cols {
                    let x = (col as f32) * h;

                    let idx = tilemap_data[ofs + (col % tiles_per_row)] as u32;
                    let mx = idx % tile_cols;
                    let my = idx / tile_cols;

                    let ty = (my as f32) * th;
                    let tx = (mx as f32) * tw;

                    tilemap.push(
                        x, y, w, h,
                        1.0, 1.0, 1.0, 1.0,
                        tx, ty, tw, th,
                        1, 0x0
                    );

                }
            }

            tilemap.end();

            tilemap
        };

        /////

        Ok(Self {
            background_material: background_material_ref,
            background: Quad::new(),
            tilemap,
            tilemap_params: shader_params,
            tilemap_material: tilemap_material_ref,
            tilemap_scroll: Vec2::new(0.0, 0.0),
            tilemap_width,
            tilemap_height
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

        let tm = crate::api::time();
        let appdata = self.appdata.as_mut().unwrap();

        let metrics = crate::api::metrics();

        { // set background position
            appdata.background.set_position(0.0, 0.0);
            appdata.background.set_size(metrics.view_width, metrics.view_height);
        }

        { // do scrolling
            appdata.tilemap_scroll.y = 0.0;
            appdata.tilemap_scroll.x -= 2.0;

            if appdata.tilemap_scroll.x < -(appdata.tilemap_width as f32 + metrics.view_width) {
                appdata.tilemap_scroll.x += appdata.tilemap_width as f32;
            };

        }

        {
            let shader_params = &mut appdata.tilemap_params;
            let params = shader_params.data_mut();

            params.time = tm.time;
            params.time_delta = tm.delta;
            params.frame += 1;
            params.offset_left = 0.0;
            params.offset_top = 0.0;
            params.window_width = metrics.window_width;
            params.window_height = metrics.window_height;
            params.view_width = metrics.view_width;
            params.view_height = metrics.view_height;
            params.view_x = metrics.view_x;
            params.view_y = metrics.view_y;
            params.view_scaling = metrics.view_scaling;

            shader_params.set_array_index(0);
            shader_params.update().unwrap();
        }


    }

    fn on_draw(&mut self) {
        //trace!("Application::on_draw");

        if self.appdata.is_none() {
            return;
        }

        let appdata = self.appdata.as_mut().unwrap();
        let renderer = crate::api::renderer_mut();
        let metrics = crate::api::metrics();

        {
            renderer.reset_scissor();
            renderer.set_material(&appdata.background_material);
            appdata.background.draw();
        }

        {
            renderer.set_scissor(metrics.view_x + 8.0 * metrics.view_scaling, metrics.view_y, metrics.view_width * metrics.view_scaling - 16.0 * metrics.view_scaling, metrics.view_height * metrics.view_scaling);
            renderer.set_material(&appdata.tilemap_material);

            let shader_params = &mut appdata.tilemap_params;
            let params = shader_params.data_mut();

            params.offset_left = appdata.tilemap_scroll.x;
            params.offset_top = metrics.view_height - appdata.tilemap_height as f32;

            let old_idx = shader_params.set_array_index(1);
            shader_params.update().unwrap();

            appdata.tilemap_material.lock().unwrap().bind_uniforms();
            shader_params.set_array_index(old_idx);

            let tilemap = &mut appdata.tilemap;
            tilemap.draw();
        }

    }

}

impl Runnable for App {}

fn main() {
    gamekit_main();
}
