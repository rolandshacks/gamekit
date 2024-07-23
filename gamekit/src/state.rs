//!
//! State
//!

use crate::{api::Disposable, task::TaskTime};

pub struct State {
    pub time: TaskTime,
}

impl Disposable for State {
    fn dispose(&mut self) {
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            time: TaskTime::default()
        }
    }
}

impl State {
}
