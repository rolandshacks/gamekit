//!
//! Custom Build
//!

extern crate gamekit;

use std::process::ExitCode;
fn main() -> ExitCode {
    gamekit::compiler::compile()
}
