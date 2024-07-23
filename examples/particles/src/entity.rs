//!
//! Entity
//!

use gamekit::{api::Metrics, *};
use gamekit::api;

use cgmath::InnerSpace;
use math::{Vec2, Vec4};

pub struct Entity {
    pub position: Vec2,
    pub size: Vec2,
    pub color: Vec4,
    pub texture_coords: Vec4,
    pub texture_mask: u32,
    pub flags: u32,
    pub target: Vec2,
    pub velocity: Vec2,
    pub time_to_live: f32,
    pub batch_index: usize
}

impl Entity {
    pub fn new() -> Self {
        Self {
            position: Vec2::new(0.0, 0.0),
            size: Vec2::new(24.0, 24.0),
            color: Vec4::new(0.5, 1.0, 0.0, 0.4),
            texture_coords: Vec4::new(0.0, 0.0, 1.0, 1.0),
            texture_mask: 0x1,
            flags: 0x0,
            target: Vec2::new(0.0, 0.0),
            velocity: Vec2::new(0.0, 0.0),
            time_to_live: 0.0,
            batch_index: 0
        }
    }

    pub fn initialize(&mut self, _frame: usize, metrics: &Metrics) {
        self.time_to_live = api::Random::get_float_range(2.0, 5.0);

        let x = api::Random::get_float_range(0.0, metrics.window_width as f32);
        let y = api::Random::get_float_range(0.0, metrics.window_height as f32);

        self.target.x = x;
        self.target.y = y;

        //let k = Random::get_float();
        self.texture_mask = 2;
    }

    pub fn update(&mut self, delta_time: f32, metrics: &Metrics) {

        let mut distance = Vec2::new(self.target.x - self.position.x, self.target.y - self.position.y);
        let len = distance.magnitude();

        // normalize in-place
        distance.x /= len;
        distance.y /= len;

        if self.time_to_live <= 0.0 || len < 100.0 {
            self.initialize(1, metrics);
            return;
        }

        if self.time_to_live > 0.0 {
            self.time_to_live -= delta_time;
        }

        self.velocity.x += distance.x * 5.0 * delta_time;
        self.velocity.y += distance.y * 5.0 * delta_time + 5.0 * delta_time;
        self.velocity = self.velocity.normalize();

        let speed = 350.0;

        self.position.x += self.velocity.x * speed * delta_time;
        self.position.y += self.velocity.y * speed * delta_time;

        let c = 1.0 - (self.position.y / metrics.view_width).clamp(0.0, 1.0);
        self.color.x = 1.0 - self.velocity.y.clamp(0.0, 1.0);
        self.color.y = c;
        self.color.z = 1.0 - c;

        let min_x = 0.0f32;
        let max_x = (metrics.view_width as f32) - self.size.x;
        let min_y = 0.0f32;
        let max_y = (metrics.view_height as f32) - self.size.y;

        if self.position.y >= max_y {
            self.position.y = max_y;
            self.velocity.y = - (self.velocity.y * 2.0).abs();
        } else if  self.position.y <= min_y {
            self.position.y = min_y;
            self.velocity.y = self.velocity.y.abs();
        }

        if self.position.x >= max_x {
            self.position.x = max_x;
            self.velocity.x = - self.velocity.x.abs();
        } else if  self.position.x <= min_x {
            self.position.x = min_x;
            self.velocity.x = self.velocity.x.abs();
        }


    }

}
