//!
//! Input
//!

use crate::api::Disposable;
use crate::error::Error;

extern crate sdl2;

use log::{*};

pub trait InputEventListener {
    fn on_keystate_change(&mut self, _keystate: u32, _oldstate: u32) {}
}

pub struct Input {
    keyboard_state: u32
}

impl Disposable for Input {
    fn dispose(&mut self) {
        trace!("Input::dispose");
    }
}

impl Input {
    pub const KEYFLAG_NONE: u32 = 0x0;
    pub const KEYFLAG_LEFT: u32 = 0x1;
    pub const KEYFLAG_RIGHT: u32 = 0x2;
    pub const KEYFLAG_UP: u32 = 0x4;
    pub const KEYFLAG_DOWN: u32 = 0x8;
    pub const KEYFLAG_BUTTON1: u32 = 0x10;
    pub const KEYFLAG_BUTTON2: u32 = 0x20;
    pub const KEYFLAG_BUTTON3: u32 = 0x40;
    pub const KEYFLAG_BUTTON4: u32 = 0x80;

    pub fn new() -> Result<Self, Error> {
        trace!("initialized input subsystem");
        Ok(Self {
            keyboard_state: Self::KEYFLAG_NONE
        })
    }

    pub fn dispatch_event<T: InputEventListener>(&mut self, event: &sdl2::event::Event, input_event_listener: &mut T) {
        let (keycode, key_down) = match event {
            sdl2::event::Event::KeyDown { keycode: Some(keycode), .. } => {
                (keycode, true)
            },
            sdl2::event::Event::KeyUp { keycode: Some(keycode), .. } => {
                (keycode, false)
            },
            _ => { return; },
        };

        let mut mask = Self::KEYFLAG_NONE;

        match *keycode {
            sdl2::keyboard::Keycode::LEFT => { mask |= Self::KEYFLAG_LEFT; },
            sdl2::keyboard::Keycode::RIGHT => { mask |= Self::KEYFLAG_RIGHT; }
            sdl2::keyboard::Keycode::UP => { mask |= Self::KEYFLAG_UP; },
            sdl2::keyboard::Keycode::DOWN => { mask |= Self::KEYFLAG_DOWN; }
            sdl2::keyboard::Keycode::LCTRL => { mask |= Self::KEYFLAG_BUTTON1; }
            sdl2::keyboard::Keycode::LSHIFT => { mask |= Self::KEYFLAG_BUTTON2; }
            _ => {
                trace!("Input::keyboard_event : {} {}", keycode, if key_down { "down" } else { "up" });
            }
        }

        if mask != Self::KEYFLAG_NONE {

            let old_state = self.keyboard_state;

            if key_down {
                self.keyboard_state |= mask;
            } else {
                self.keyboard_state &= !mask;
            }

            if old_state != self.keyboard_state {
                //trace!("changed keyboard state: {}", self.keyboard_state);
                input_event_listener.on_keystate_change(self.keyboard_state, old_state);
            }

        }

    }

    pub fn keyboard_state(&self) -> u32 {
        self.keyboard_state
    }

}
