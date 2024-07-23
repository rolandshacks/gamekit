//!
//! State
//!

use crate::{api::Disposable, task::TaskTime};

#[derive(Default)]
pub struct State {
    pub time: TaskTime,
}

impl Disposable for State {
    fn dispose(&mut self) {
    }
}

impl State {
}
