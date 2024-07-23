//!
//! Gamekit Compiler
//! 
//! This executable is mainly used as a frontend for testing purposes.
//! Usually, the compiler is directly used through a custom build.rs file.
//!

extern crate gamekit;

use std::env;
use std::process::ExitCode;
use std::path::Path;

fn main() -> ExitCode {

    let cwd = env::current_dir().unwrap();
    let out_dir = Path::new(&cwd).join("target");

    env::set_var("OUT_DIR", out_dir);
    env::set_var("DEBUG", "true");
    env::set_var("OPT_LEVEL", "0");

    gamekit::compiler::compile()
}
