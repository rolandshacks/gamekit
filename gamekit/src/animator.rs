//!
//! Animator
//!

#[derive(PartialEq, Default)]
pub enum AnimatorMode {
    #[default]
    ForwardLoop,
    Forward,
    BackwardLoop,
    Backward,
    PingPong,
    PingPongLoop,
    Idle
}

pub struct Animator {
    pub active: bool,
    pub mode: AnimatorMode,
    pub value: f32,
    pub start: f32,
    pub end: f32,
    pub step: f32,
    step_sign: f32
}

impl Default for Animator {
    fn default() -> Self {
        Self::new(0.0, 1.0, 0.0, 0.1, AnimatorMode::ForwardLoop)
    }
}

impl Animator {
    pub fn idle() -> Self {
        Self::new(0.0, 1.0, 0.0, 0.1, AnimatorMode::Idle)
    }

    pub fn new(start: f32, end: f32, value: f32, step: f32, mode: AnimatorMode) -> Self {
        Self {
            active: true,
            mode,
            value,
            start,
            end,
            step,
            step_sign: 1.0
        }
    }

    pub fn set(&mut self, start: f32, end: f32, value: f32, step: f32, mode: AnimatorMode) {
        self.start = start;
        self.end = end;
        self.value = value;
        self.step = step;
        self.mode = mode;
    }

    pub fn update(&mut self, delta: f32) {

        if !self.active || self.mode == AnimatorMode::Idle {
            return;
        }

        let step = self.step * self.step_sign * delta;

        match self.mode {
            AnimatorMode::Forward => {
                self.value += step;
                if self.value < self.start || self.value >= self.end {
                    self.value = self.end;
                    self.active = false;
                }
            },
            AnimatorMode::ForwardLoop => {
                self.value += step;
                if self.value < self.start || self.value > self.end {
                    self.value = self.start;
                }
            },
            AnimatorMode::Backward => {
                self.value -= step;
                if self.value <= self.start || self.value > self.end {
                    self.value = self.start;
                    self.active = false;
                }
            },
            AnimatorMode::BackwardLoop => {
                self.value -= step;
                if self.value < self.start || self.value > self.end {
                    self.value = self.end;
                }
            },
            AnimatorMode::PingPong => {
                self.value += step;
                if self.value < self.start || self.value > self.end {
                    self.value = self.value.clamp(self.start, self.end);
                    if self.step_sign >= 0.0 {
                        self.step_sign = -self.step_sign;
                    } else {
                        self.active = false;
                    }
                }
            },
            AnimatorMode::PingPongLoop => {
                self.value += step;
                if self.value < self.start || self.value > self.end {
                    self.value = self.value.clamp(self.start, self.end);
                    self.step_sign = -self.step_sign;
                }
            },
            _ => {}
        }

    }
}
