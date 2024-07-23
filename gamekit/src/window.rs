//!
//! Window
//!

use crate::{api::Disposable, error::Error, input::InputEventListener, types::Surface};

use ash::vk::Handle;
use log::{*};

pub struct Window {
    video_subsystem: sdl2::VideoSubsystem,
    window: sdl2::video::Window,
    event_pump: sdl2::EventPump,
    pub surface_instance: ash::khr::surface::Instance,
    pub surface: Surface
}

impl Disposable for Window {
    fn dispose(&mut self) {
        trace!("Window::dispose");

        if !self.surface.obj.is_null() {
            unsafe {
                self.surface_instance.destroy_surface(self.surface.obj, None);
            }
            self.surface.obj = ash::vk::SurfaceKHR::null();
            self.surface.handle = 0u64;
        }
    }
}

impl Window {

    pub fn new() -> Result<Self, Error> {

        trace!("create window");

        let options = crate::globals::options();
        let entry = crate::globals::entry();
        let instance = crate::globals::instance();

        let surface_instance = ash::khr::surface::Instance::new(entry, &instance.obj);

        let sdl = &instance.sdl;
        let event_pump = sdl.event_pump().unwrap();
        let video_subsystem = sdl.video().unwrap();

        let mut win_x = if options.window_x == i32::MAX { sdl2::sys::SDL_WINDOWPOS_UNDEFINED_MASK as i32 } else { options.window_x as i32 };
        let mut win_y = if options.window_y == i32::MAX { sdl2::sys::SDL_WINDOWPOS_UNDEFINED_MASK as i32 } else { options.window_y as i32 };
        let win_width = options.window_width;
        let win_height = options.window_height;

        if win_x < 0 || win_y < 0 {

            let bounds = match video_subsystem.display_bounds(0) {
                Ok(bounds) => bounds,
                Err(s) => { return Err(Error::from(s)); }
            };

            let dpi = match video_subsystem.display_dpi(0) {
                Ok(dpi) => dpi,
                Err(s) => { return Err(Error::from(s)); }
            };

            let scale_x = if dpi.1 > 144.0 { dpi.1 / 144.0 } else { 1.0 };
            let scale_y = if dpi.2 > 144.0 { dpi.2 / 144.0 } else { 1.0 };

            if win_x < 0 { win_x += 1 + ((bounds.x + bounds.w) as f32 * scale_x).floor() as i32 - win_width as i32 };
            if win_y < 0 { win_y += 1 + ((bounds.y + bounds.h) as f32 * scale_y).floor() as i32  - win_height as i32 };

        }

        let window = video_subsystem
            .window(&options.title, win_width, win_height)
            .position(win_x, win_y)
            .vulkan()
            .resizable()
            .build()
            .unwrap();

        let surface_handle = window.vulkan_create_surface(instance.obj.handle().as_raw() as usize).unwrap();
        let surface_obj = ash::vk::SurfaceKHR::from_raw(surface_handle);

        let surface = Surface {
            handle: surface_handle,
            obj: surface_obj
        };

        Ok(Self {
            video_subsystem,
            window,
            surface_instance,
            surface,
            event_pump
        })
    }

    pub fn process_events<T: InputEventListener>(&mut self, input_event_listener: &mut T) -> bool {

        let mut viewport_changed = false;

        let input = crate::globals::input_mut();

        for event in self.event_pump.poll_iter() {

            input.dispatch_event(&event, input_event_listener);

            match event {
                sdl2::event::Event::Quit {..} => { return false },
                sdl2::event::Event::KeyUp { keycode: Some(sdl2::keyboard::Keycode::Escape), .. } => { return false },
                sdl2::event::Event::Window {timestamp: _, window_id: _, win_event} => {
                    match win_event {
                        sdl2::event::WindowEvent::Resized(..) => { viewport_changed = true; },
                        _ => {}
                    }
                },
                _ => {},
            }
        }

        if viewport_changed {
            // handle if needed
        }

        return true;

    }

}
