//!
//! Gamekit "Hello, world!" demo.
//!
//! This is the most simplistic application setup
//! to get started with. It does not produce any graphics,
//! but shows how to set up the gamekit application and
//! custom build.
//!

#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/manifest.rs"));

use log::{*};
use gamekit::api::{*};

struct ApplicationData {
}

impl Disposable for ApplicationData {
    fn dispose(&mut self) {
    }
}

impl ApplicationData {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {})
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

    fn on_update(&mut self) {
        //let tm = crate::api::time();
        //trace!("Application::on_update");
    }

    fn on_draw(&mut self) {
        //trace!("Application::on_draw");
    }

    fn on_async_update(&mut self, task_context: &TaskContext) {
        trace!("Application::on_async_update [#{}:{}]", task_context.id(), task_context.name());
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
