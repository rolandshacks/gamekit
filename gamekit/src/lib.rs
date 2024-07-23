//!
//! Gamekit is a lightweight gaming framework with the focus on 2D graphics.
//!
//! The Gamekit core is based on
//!
//! - [Vulkan] Graphics and compute API
//! - [Ash] Vulkan bindings
//! - [SDL] Simple DirectMedia Layer
//! - [Serde] parser framework with json5
//! - [cgmath] linear algebra and mathematics library
//!
//! [Vulkan]: https://vulkan.lunarg.com
//! [Ash]: https://github.com/ash-rs/ash
//! [SDL]: https://www.libsdl.org
//! [Serde]: https://serde.rs/
//! [cgmath]: https://github.com/rustgd/cgmath

#![allow(dead_code)]

mod thirdparty;

mod constants;
mod globals;
mod macros;
mod error;
mod state;
mod window;
mod exec;
mod renderer;
mod options;
mod metrics;
mod task;
mod instance;
mod device;
mod swapchain;
mod pipeline;
mod types;
mod buffer;
mod resources;
mod image;
mod texture;
mod shader;
mod material;
mod primitives;
mod random;
mod logger;
mod animator;
mod sprite;
mod bitmap;
mod font;
mod data;
mod blitter;
mod audio;
mod input;
mod maps;
mod builtin;

pub mod api;
//mod manifest;
//pub mod compiler;
pub mod math;

use std::process::ExitCode;

use api::Disposable;
use gamebuilder::manifest::ApplicationDescriptorTable;
use log::{*};

/// Default application main function that implements a basic
/// init-run-dispose application lifecycle.
pub fn default_main<T: api::Application + api::Runnable + api::Disposable + 'static>(
    descriptors: &'static ApplicationDescriptorTable,
    logger: &'static api::DefaultLogger) {

    api::init_logger(logger, api::LogLevel::Trace);

    trace!("start");

    println!("=============");
    println!("=  GAMEKIT  =");
    println!("=============");

    trace!("create application");

    let mut exec: api::Exec<T> = match api::Exec::new(descriptors) {
        Ok(app) => app,
        Err(e) => {
            error!("initialization failed: {}", e.message());
            return;
        }
    };

    trace!("run application");
    exec.run();

    trace!("shutdown application");
    exec.dispose();

    trace!("exit");

}

pub fn build() -> ExitCode {
    gamebuilder::build()
}

#[macro_export]
macro_rules! build {
    () => {
        extern crate gamekit;
        fn main() -> std::process::ExitCode {
            gamekit::build()
        }
    };
}

#[macro_export]
macro_rules! load {
    () => {
        include!(concat!(env!("OUT_DIR"), "/manifest.rs"));
    };
}

#[macro_export]
macro_rules! main {
    () => {
        static LOGGER: gamekit::api::DefaultLogger = gamekit::api::default_logger();
        gamekit::default_main::<App>(DESCRIPTOR_TABLE, &LOGGER);
    };
}
