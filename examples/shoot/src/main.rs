//!
//! Gamekit "Shooter" demo.
//!

#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/manifest.rs"));

use log::{*};
use gamekit::api;
use gamekit::api::{*};

const PLAY_MUSIC: bool = false;

#[repr(C)]
#[derive(Default)]
struct ShaderParams {
    window_width: f32,
    window_height: f32,
    view_width: f32,
    view_height: f32,
    view_x: f32,
    view_y: f32,
    view_scaling: f32,
    time: f32,
    time_delta: f32,
    frame: i32
}

impl ShaderParams {
    pub fn new(index: u32, dynamic_array_elements: usize) -> Result<Uniform::<Self>, Error> {
        Uniform::<Self>::new(index, dynamic_array_elements)
    }
}

#[repr(C)]
#[derive(Default)]
struct PushParams {
    offset_x: f32,
    offset_y: f32
}

impl PushParams {
    pub fn new() -> Result<PushConstants::<Self>, Error> {
        PushConstants::<Self>::new()
    }
}

#[derive(Default)]
struct SpriteAttributes {
    vx: f32,
    vy: f32
}

impl SpriteMeta for SpriteAttributes {
    fn update(&mut self, _data: &mut SpriteData, _step: f32) {
    }
}

struct ApplicationData {
    material_sprites_ref: MaterialLockRef,
    sprite: Sprite<SpriteAttributes>,
    shader_params: Uniform::<ShaderParams>,
    push_params: PushConstants::<PushParams>,
    fire_pressed: bool,
    shoot_sample: SampleLockRef,
    shoot_timer: f32
}

impl Disposable for ApplicationData {
    fn dispose(&mut self) {
    }
}

impl ApplicationData {
    pub fn new() -> Result<Self, Error> {

        //let metrics = crate::api::metrics();

        let push_params = PushParams::new()?;
        let shader_params = ShaderParams::new(0, 0)?;

        let materials = crate::api::materials();
        let material_sprites_ref = materials.get("default");
        material_sprites_ref.lock().unwrap()
            .add_push_constants(&push_params)
            .add_uniform(&shader_params);

        let renderer = crate::api::renderer_mut();

        renderer.generate_sprite_sheet(448, 32, 32, 32);
        let mut sprite: Sprite<SpriteAttributes> = Sprite::default();
        sprite.set_position(0.0, 0.0);
        sprite.set_size(32.0, 32.0);
        //sprite.set_size(32.0 * metrics.scaling, 32.0 * metrics.scaling);
        sprite.set_frame(0.0);

        let resources = crate::api::resources();
        let sample = resources.get_sample("sample");

        Ok(Self {
            material_sprites_ref,
            sprite,
            shader_params,
            push_params,
            fire_pressed: false,
            shoot_sample: sample,
            shoot_timer: 0.0
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

        let resources = crate::api::resources();

        if PLAY_MUSIC {
            let audio = crate::api::audio();
            let music = resources.get_music("music");
            audio.play_music(&music, 0.25);
        }

        let appdata = ApplicationData::new()?;

        Ok(Self {
            appdata: Some(appdata)
        })
    }

    fn on_init(&mut self) {
        trace!("Application::on_init");
    }

    fn on_ready(&mut self) {
        trace!("Application::on_ready");

        let appdata = self.appdata.as_mut().unwrap();

        {
            let metrics = crate::api::metrics();
            let params = appdata.shader_params.data_mut();
            params.window_width = metrics.window_width;
            params.window_height = metrics.window_height;
            params.view_width = metrics.view_width;
            params.view_height = metrics.view_height;
            params.view_x = metrics.view_x;
            params.view_y = metrics.view_y;
            params.view_scaling = metrics.view_scaling;
            params.time = 0.0;
            params.time_delta = 0.0;
            params.frame = 0;
            let _ = appdata.shader_params.update_all();
        }

        let renderer = crate::api::renderer_mut();
        renderer.set_material(&appdata.material_sprites_ref);

    }

    fn on_shutdown(&mut self) {
        trace!("Application::on_shutdown");

        if PLAY_MUSIC {
            let audio = crate::api::audio();
            audio.stop_music();
        }
    }

    fn on_update(&mut self) {
        //trace!("Application::on_update");

        let metrics = crate::api::metrics();

        let input = crate::api::input();
        let keyboard_state = input.keyboard_state();

        let tm = crate::api::time();
        let appdata = self.appdata.as_mut().unwrap();
        let _ = appdata.push_params.update();
        let delta = tm.step;


        if appdata.shoot_timer > 0.0 {
            appdata.shoot_timer -= delta;
        }

        if appdata.shoot_timer <= 0.0 && appdata.fire_pressed {

            appdata.fire_pressed = false;

            let audio = crate::api::audio();
            let _ = audio.play_sample(&appdata.shoot_sample, 0, 1.0);

            appdata.shoot_timer = 0.1;
        }

        {
            const ACCELERATION: f32 = 7500.0;
            const DECELERATION: f32 = 10.0;

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

            let sprite = &mut appdata.sprite;

            let sprite_width = sprite.size().x;
            let sprite_height = sprite.size().y;

            let vx;
            let vy;

            {
                let meta = &mut sprite.meta;

                if force_x != 0.0 {
                    meta.vx = (meta.vx + delta * force_x).clamp(-100.0, 100.0);
                } else {
                    meta.vx =  meta.vx * (1.0 - delta * DECELERATION).clamp(0.0, 1.0);
                }

                if force_y != 0.0 {
                    meta.vy = (meta.vy + delta * force_y).clamp(-100.0, 100.0);
                } else {
                    meta.vy =  meta.vy * (1.0 - delta * DECELERATION).clamp(0.0, 1.0);
                }

                vx = meta.vx;
                vy = meta.vy;
            }

            {
                let data = sprite.data_mut();
                data.position.x = (data.position.x + delta * vx).clamp(0.0, metrics.view_width - sprite_width);
                data.position.y = (data.position.y + delta * vy).clamp(0.0, metrics.view_height - sprite_height);
            }

            sprite.update(delta);
        }

    }

    fn on_draw(&mut self) {
        //trace!("Application::on_draw");

        if self.appdata.is_none() {
            return;
        }

        //let metrics = crate::api::metrics();

        let appdata = self.appdata.as_mut().unwrap();
        let renderer = crate::api::renderer_mut();

        {
            renderer.begin();

            let sprite = &mut appdata.sprite;
            renderer.draw_sprite(sprite);

            renderer.end();
        }

    }

    fn on_async_update(&mut self, task_context: &TaskContext) {
        trace!("Application::on_async_update [#{}:{}]", task_context.id(), task_context.name());
    }

    fn on_keystate_change(&mut self, keystate: u32, oldstate: u32) {
        trace!("Application::on_keystate_change [{}]", keystate);

        if (keystate & Input::KEYFLAG_BUTTON1) != 0 && (oldstate & Input::KEYFLAG_BUTTON1) == 0 {
            if self.appdata.is_none() {
                return;
            }

            let appdata = self.appdata.as_mut().unwrap();

            appdata.fire_pressed = true;
        }
    }

}

impl Runnable for App {
    fn run_delta(&mut self, task_context: &TaskContext) {
        self.on_async_update(task_context);
    }
}

fn main() {
    gamekit_main();
}
