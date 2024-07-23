//!
//! Public application programming interface.
//!

pub struct Api {}

impl Api {
}

/// Application trait
pub trait Application {
    fn new() -> Result<Self, Error> where Self: Sized;
    fn on_init(&mut self);
    fn on_shutdown(&mut self) {}
    fn on_async_update(&mut self, _task_context: &TaskContext) {}
    fn on_ready(&mut self) {}
    fn on_update(&mut self) {}
    fn on_draw(&mut self) {}
    fn on_metrics(&mut self) {}
    fn on_keystate_change(&mut self, _keystate: u32, _oldstate: u32) {}
}

/// Runnable to be used for task callbacks
pub trait Runnable: Send {
    fn start(&mut self) {}
    fn stop(&mut self) {}
    fn run(&mut self) {}
    fn run_delta(&mut self, _task_context: &TaskContext) {}
    fn is_running(&self) -> bool { return true; }
}

/// Disposable application resources
pub trait Disposable {
    fn dispose(&mut self);
}

/// Error type
pub type Error = crate::error::Error;

/// Default logger implementation
pub type DefaultLogger = crate::logger::DefaultLogger;

/// Definition of the log level filter values
pub type LogLevel = log::LevelFilter;

/// Get the default logger
pub const fn default_logger() -> DefaultLogger {
    crate::logger::default()
}

/// Initialize logger
pub fn init_logger(logger: &'static dyn log::Log, log_level: LogLevel) {
    crate::logger::init(logger, log_level)
}

/// Generic atomic reference counting read-write lock
pub type LockRef<T> = std::sync::Arc<std::sync::Mutex<T>>;

/// Application options
pub type Options = crate::options::Options;

/// Blend mode
pub type BlendMode = crate::material::BlendMode;

/// Material
pub type Material = crate::material::Material;

/// Shared material reference
pub type MaterialLockRef = crate::material::MaterialLockRef;

/// Quadric
pub type Quad = crate::primitives::Quad;

/// Vertex queue
pub type VertexQueue = crate::primitives::VertexQueue;

/// Typed uniform buffer
pub type Uniform<T> = crate::buffer::Uniform<T>;

/// Typed push constants
pub type PushConstants<T> = crate::buffer::PushConstants<T>;

/// Random number generator
pub type Random = crate::random::Random;

/// Application metrics
pub type Metrics = crate::metrics::Metrics;

/// Task time information
pub type TaskTime = crate::task::TaskTime;

/// Task context information
pub type TaskContext = crate::task::TaskContext;

/// Sprite base data
pub type SpriteData = crate::sprite::SpriteData;

/// Typed sprite
pub type Sprite<T=crate::sprite::DefaultSpriteMeta> = crate::sprite::Sprite<T>;

/// Animator mode
pub type AnimatorMode = crate::animator::AnimatorMode;

/// Animator
pub type Animator = crate::animator::Animator;

/// Font
pub type Font = crate::font::Font;

/// Shared font reference
pub type FontLockRef = crate::font::FontLockRef;

/// Bitmap
pub type Bitmap = crate::bitmap::Bitmap;

/// Shared bitmap reference
pub type BitmapLockRef = crate::bitmap::BitmapLockRef;

/// Static data
pub type StaticData = crate::data::StaticData;

/// Shared static data reference
pub type StaticDataLockRef = crate::data::StaticDataLockRef;

/// Audio
pub type Audio = crate::audio::Audio;

/// Music
pub type Music = crate::audio::Music;

/// Shared music reference
pub type MusicLockRef = crate::audio::MusicLockRef;

/// Sample
pub type Sample = crate::audio::Sample;

/// Shared sample reference
pub type SampleLockRef = crate::audio::SampleLockRef;

/// Input
pub type Input = crate::input::Input;

/// Sprite meta data encoder
pub trait SpriteMeta {
    fn update(&mut self, _data: &mut SpriteData, _step: f32) {}
    fn encode(&mut self, _data: &mut SpriteData) {}
}

// math

/// 2D vector
pub type Vec2 = crate::math::Vec2;

/// 3D vector
pub type Vec3 = crate::math::Vec3;

/// 4D vector
pub type Vec4 = crate::math::Vec4;

// exec

/// Generic main executable lifecycle
pub type Exec<T> = crate::exec::Exec<T>;

// global access

/// Get global metrics
pub fn metrics() -> &'static crate::metrics::Metrics {
    crate::globals::metrics()
}

/// Get global resources
pub fn resources() -> &'static crate::resources::Resources {
    crate::globals::resources()
}

/// Get global materials
pub fn materials() -> &'static crate::material::Materials {
    crate::globals::materials()
}

/// Get global materials as mutable
pub fn materials_mut() -> &'static mut crate::material::Materials {
    crate::globals::materials_mut()
}

/// Get global time
pub fn time() -> &'static crate::task::TaskTime {
    crate::globals::time()
}

/// Get global state
pub fn state() -> &'static crate::state::State {
    crate::globals::state()
}

/// Get global state as mutable
pub fn state_mut() -> &'static mut crate::state::State {
    crate::globals::state_mut()
}

/// Get global renderer
pub fn renderer() -> &'static crate::renderer::Renderer {
    crate::globals::renderer()
}

/// Get global renderer as mutable
pub fn renderer_mut() -> &'static mut crate::renderer::Renderer {
    crate::globals::renderer_mut()
}

/// Get global options
pub fn options() -> &'static crate::options::Options {
    crate::globals::options()
}

/// Get global audio
pub fn audio() -> &'static crate::audio::Audio {
    crate::globals::audio()
}

/// Get global audio as mutable
pub fn audio_mut() -> &'static mut crate::audio::Audio {
    crate::globals::audio_mut()
}

/// Get global input
pub fn input() -> &'static crate::input::Input {
    crate::globals::input()
}

/// Get global input as mutable
pub fn input_mut() -> &'static mut crate::input::Input {
    crate::globals::input_mut()
}
