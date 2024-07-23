//!
//! Terminal logger
//!
//! # Example
//!
//! ```
//! use log::{*};
//! static LOGGER: api::DefaultLogger = api::default_logger();
//! fn main() {
//!     api::init_logger(logger, api::LogLevel::Trace);
//!     trace!("trace");
//!     debug!("debug");
//!     info!("info");
//!     warn!("warn");
//!     error!("error");
//! }
//! ```
//!

use log::{Log, Metadata, Record};

pub struct DefaultLogger;

static SHOW_TIMESTAMP: bool = false;
static mut START_TIME: Option<std::time::Instant> = None;

impl log::Log for DefaultLogger {

    fn enabled(&self, metadata: &Metadata) -> bool {
        let max_level = log::max_level();
        metadata.level() <= max_level
    }

    fn log(&self, record: &Record) {

        if !self.enabled(record.metadata()) {
            return;
        }

        let mut timestamp: u128 = 0;

        if SHOW_TIMESTAMP {
            unsafe {
                timestamp = START_TIME.unwrap().elapsed().as_millis();
            }
        }

        let level = record.level();

        let module = record.module_path().unwrap_or_default();

        let location = match (record.file(), record.line()) {
            (Some(s), Some(l)) => format!("{}({}) ", s, l),
            (Some(s), None) => format!("{} ", s),
            _ => String::from("")
        };

        let (level_tag, record_cols) = match level {
            log::Level::Trace => ("\x1b[0;102m T ", "\x1b[37m"),
            log::Level::Debug => ("\x1b[0;102m D ", "\x1b[37m"),
            log::Level::Info => ("\x1b[47m I ", "\x1b[37m"),
            log::Level::Warn => ("\x1b[43m W ", "\x1b[0;93m"),
            log::Level::Error => ("\x1b[41m E ", "\x1b[0;91m")
        };

        if SHOW_TIMESTAMP {
            println!("{:<6} {:<48.48} \x1b[97m{:<24.24}\x1b[0m{}\x1b[0m {}{}\x1b[0m", timestamp, location, module, level_tag, record_cols, record.args());
        } else {
            println!("{:<48.48} \x1b[97m{:<24.24}\x1b[0m{}\x1b[0m {}{}\x1b[0m", location, module, level_tag, record_cols, record.args());
        }

    }

    fn flush(&self) {}
}

pub const fn default() -> DefaultLogger {
    DefaultLogger{}
}

pub fn init(logger: &'static dyn Log, log_level: log::LevelFilter) {
    unsafe {
        START_TIME = Some(std::time::Instant::now());
    }

    let _ = log::set_logger(logger)
                .map(|()| log::set_max_level(log_level));
}

#[cfg(test)]
mod tests {
    //use std::ptr::null;

    //use super::*;
    use log::{*};

    #[test]
    fn test_logging() {

        static LOGGER: crate::logger::DefaultLogger = crate::logger::default();

        {
            crate::logger::init(&LOGGER, crate::LevelFilter::Trace);
            trace!("Trace log");
            debug!("Debug log");
            info!("Info log");
            warn!("Warn log");
            error!("Error log");
        }

        //assert_eq!(2, 3);
    }

}
