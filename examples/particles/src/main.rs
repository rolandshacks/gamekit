//!
//! Gamekit particles, spite and font demo.
//!

#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/manifest.rs"));

use std::f32::consts::PI;

use log::{*};
use gamekit::api;
use gamekit::api::{*};

mod entity;
use entity::Entity;

const NUM_ENTITIES: usize = 250;

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

#[repr(C)]
#[derive(Default)]
struct PushParams {
    offset_x: f32,
    offset_y: f32
}

struct Explosion {
    anim: Animator
}

impl SpriteMeta for Explosion {

    fn update(&mut self, data: &mut SpriteData, step: f32) {
        self.anim.update(step);
        data.frame = self.anim.value;
    }
}

impl Default for Explosion {
    fn default() -> Self {
        Self {
            anim: Animator::new(0.0, 7.999, 0.0, 10.0, AnimatorMode::ForwardLoop)
        }
    }
}

struct ApplicationData {
    material_logo_ref: MaterialLockRef,
    material_particles_ref: MaterialLockRef,
    material_sprites_ref: MaterialLockRef,
    material_text_ref: MaterialLockRef,
    entities: Vec<Entity>,
    logo: Quad,
    logo_ofs_x: f32,
    logo_ofs_y: f32,
    vertex_queue: VertexQueue,
    shader_params: Uniform::<ShaderParams>,
    push_params: PushConstants::<PushParams>,
    sprite: Sprite<Explosion>,
}

impl Disposable for ApplicationData {
    fn dispose(&mut self) {
        self.vertex_queue.dispose();
        self.logo.dispose();
    }
}

impl ApplicationData {
    pub fn new() -> Result<Self, Error> {

        let push_params = PushConstants::<PushParams>::new()?;
        let shader_params = Uniform::<ShaderParams>::new(0, 0)?;

        /////

        let materials = crate::api::materials();
        let material_logo_ref = materials.get("logo");
        material_logo_ref.lock().unwrap()
            .add_push_constants(&push_params)
            .add_uniform(&shader_params);

        let material_particles_ref = materials.get("particles");
        material_particles_ref.lock().unwrap()
            .add_push_constants(&push_params)
            .add_uniform(&shader_params);

        let material_sprites_ref = materials.get("sprites");
        material_sprites_ref.lock().unwrap()
            .add_push_constants(&push_params)
            .add_uniform(&shader_params);

        let material_text_ref = materials.get("text");
        material_text_ref.lock().unwrap()
            .add_push_constants(&push_params)
            .add_uniform(&shader_params);

        /////

        let mut quad = Quad::new();

        let metrics = crate::api::metrics();

        let screen_size = (
            metrics.window_width as f32,
            metrics.window_height as f32
        );

        {
            let mut quad_width = screen_size.0;
            let mut quad_height = screen_size.1;
            let quad_ratio = quad_width / quad_height;

            if quad_width > screen_size.0 as f32 {
                quad_width = screen_size.0 as f32;
                quad_height = quad_width / quad_ratio;
            }

            if quad_height > metrics.window_height as f32 {
                quad_height = metrics.window_height as f32;
                quad_width = quad_height * quad_ratio;
            }

            quad.set_coords((screen_size.0  - quad_width)/2.0, (screen_size.1-quad_height)/2.0, quad_width, quad_height);
            quad.set_texture_mask(0x1);
        }

        let vertex_queue = VertexQueue::new(NUM_ENTITIES);

        let mut entities = vec![];

        for _ in 0..NUM_ENTITIES {
            let mut entity = Entity::new();
            entity.initialize(0, metrics);
            entities.push(entity);
        }

        /////

        let renderer = crate::api::renderer_mut();

        renderer.generate_sprite_sheet(128*7, 128, 128, 128);
        let mut sprite: Sprite<Explosion> = Sprite::default();
        sprite.set_position(100.0, 100.0);
        sprite.set_size(100.0, 100.0);
        sprite.meta.anim.set(0.0, 7.999, 0.0, 10.0, AnimatorMode::ForwardLoop);

        Ok(Self {
            material_logo_ref,
            material_particles_ref,
            material_sprites_ref,
            material_text_ref,
            entities,
            logo: quad,
            logo_ofs_x: 0.0,
            logo_ofs_y: 0.0,
            vertex_queue,
            shader_params,
            push_params,
            sprite
        })
    }
}

struct App {
    screen_size: Vec2,
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
        let metrics = crate::api::metrics();
        let appdata = ApplicationData::new()?;

        Ok(Self {
            screen_size: Vec2 { x: metrics.window_width as f32, y: metrics.window_height as f32 },
            appdata: Some(appdata)
        })
    }

    fn on_init(&mut self) {
        trace!("MyExec::on_init");
    }

    fn on_shutdown(&mut self) {
        trace!("MyExec::on_shutdown");
    }

    fn on_update(&mut self) {
        //trace!("MyExec::on_udpate");

        if self.appdata.is_none() {
            return;
        }

        let tm = crate::api::time();
        let appdata = self.appdata.as_mut().unwrap();
        let metrics = crate::api::metrics();

        {
            let shader_params = &mut appdata.shader_params;
            let params = shader_params.data_mut();
            params.window_width = self.screen_size.x;
            params.window_height = self.screen_size.y;
            params.view_width = self.screen_size.x;
            params.view_height = self.screen_size.y;
            params.view_x = 0.0;
            params.view_y = 0.0;
            params.view_scaling = 1.0;
            params.time = tm.time;
            params.time_delta = tm.step;
            params.frame += 1;
            shader_params.update().unwrap();
        }

        {
            let vertex_queue = &mut appdata.vertex_queue;
            vertex_queue.begin();
            for entity in &mut appdata.entities {
                entity.update(tm.step, metrics);
                vertex_queue.push(
                    entity.position.x, entity.position.y,
                    entity.size.x, entity.size.y,
                    entity.color.x, entity.color.y, entity.color.z, entity.color.w,
                    entity.texture_coords.x,entity.texture_coords.y, entity.texture_coords.z, entity.texture_coords.w,
                    entity.texture_mask, entity.flags
                );
            }
            vertex_queue.end();
        }

        {
            appdata.logo_ofs_x += tm.step * 400.0;
            if appdata.logo_ofs_x > 3000.0 {
                appdata.logo_ofs_x = 0.0;
            }

            appdata.logo_ofs_y += tm.step * 6.0;
            if appdata.logo_ofs_y > 2.0*PI {
                appdata.logo_ofs_y -= 2.0*PI;
            }

        }

        {
            let sprite = &mut appdata.sprite;
            sprite.update(tm.step);
        }


    }

    fn on_draw(&mut self) {
        //trace!("MyExec::on_draw");

        if self.appdata.is_none() {
            return;
        }

        let appdata = self.appdata.as_mut().unwrap();
        let renderer = crate::api::renderer_mut();

        {
            renderer.set_material(&appdata.material_particles_ref);

            {
                let data = appdata.push_params.data_mut();
                data.offset_x = 0.0;
                data.offset_y = 0.0;
            }

            let _ = appdata.push_params.update();

            let vertex_queue = &mut appdata.vertex_queue;
            vertex_queue.draw();
        }

        {
            renderer.set_material(&appdata.material_sprites_ref);

            let sprite = &mut appdata.sprite;

            renderer.begin();
            renderer.draw_sprite(sprite);
            renderer.end();
        }

        {
            renderer.set_material(&appdata.material_text_ref);

            renderer.begin();
            renderer.draw_text_scaled(10.0, 10.0, 4.0, 4.0, "Hello, world!");
            renderer.end();
        }

        {
            renderer.set_material(&appdata.material_logo_ref);

            let metrics = crate::api::metrics();

            let quad = &mut appdata.logo;

            {
                let data = appdata.push_params.data_mut();
                data.offset_x = appdata.logo_ofs_x - 1000.0;
                data.offset_y = 0.0 + f32::sin(appdata.logo_ofs_y) * 90.0;
            }
            let _ = appdata.push_params.update();
            quad.draw();

            {
                let data = appdata.push_params.data_mut();
                data.offset_x = (metrics.window_width as f32) - appdata.logo_ofs_x;
                data.offset_y = 0.0 - f32::sin(appdata.logo_ofs_y) * 90.0
            }
            let _ = appdata.push_params.update();
            quad.draw();
        }


    }

}

impl Runnable for App {}

fn main() {
    gamekit_main();
}
