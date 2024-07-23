/// ```no_run
///
/// Manifest and resource compiler and code generator.
///
/// # Build Setup
///
/// Add *gamekit* to your build dependencies in the `Cargo.toml` file similar
/// to this:
///
/// ```
/// [build-dependencies]
/// gamekit = { version = "x.y.z", path = "..." }
/// ```
///
/// Then create a custom build file `build.rs` and call the `compile()`
/// function:
///
/// ```
/// gamekit::build!();
/// ```
///
/// # Code Usage
///
/// In your application main module, include the generated application manifest:
///
/// ```
/// gamekit::load!();
/// ```
///
/// And to use the default the gamekit main entry function:
///
/// ```
/// gamekit::main!();
/// ```
///

#[allow(dead_code)]

mod compiler;
pub mod manifest;

use std::process::ExitCode;
use std::env;
use std::path::Path;

pub fn build() -> ExitCode {
    build_ex(false)
}

pub fn build_ex(overwrite_env: bool) -> ExitCode {

    //println!("cargo:warning=running gamebuilder");

    if overwrite_env {
        let cwd = env::current_dir().unwrap();
        let out_dir = Path::new(&cwd).join("target");

        unsafe {
            env::set_var("OUT_DIR", out_dir);
            env::set_var("DEBUG", "true");
            env::set_var("OPT_LEVEL", "0");
        }
    }

    match crate::compiler::compile() {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE
    }
}

#[macro_export]
macro_rules! build {
    () => {
        extern crate gamebuilder;
        fn main() -> std::process::ExitCode {
            gamebuilder::build()
        }
    };
}
