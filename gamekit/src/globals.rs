//!
//! Globals
//!

use crate::{api::Disposable, device::Device, error::Error, instance::Instance, material::Materials, metrics::Metrics, options::Options, pipeline::Pipeline, renderer::Renderer, resources::Resources, state::State, task::{TaskTime, Tasks}, window::Window, audio::Audio, input::Input };

pub struct GlobalContext {
    pub options: Options,
    pub metrics: Option<Metrics>,

    pub entry: Option<ash::Entry>,
    pub window: Option<Window>,
    pub instance: Option<Instance>,
    pub device: Option<Device>,

    pub pipeline: Option<Pipeline>,
    pub resources: Resources,
    pub materials: Materials,
    pub state: State,

    pub renderer: Option<Renderer>,

    pub audio: Option<Audio>,
    pub input: Option<Input>,

    pub tasks: Tasks
}

impl Disposable for GlobalContext {
    fn dispose(&mut self) {

        self.tasks.dispose();

        if self.input.is_some() {
            self.input.as_mut().unwrap().dispose();
            self.input = None;
        }

        if self.audio.is_some() {
            self.audio.as_mut().unwrap().dispose();
            self.audio = None;
        }

        if self.renderer.is_some() {
            self.renderer.as_mut().unwrap().dispose();
            self.renderer = None;
        }

        if self.pipeline.is_some() {
            self.pipeline.as_mut().unwrap().dispose();
            self.pipeline = None;
        }

        if self.metrics.is_some() {
            self.metrics.as_mut().unwrap().dispose();
            self.metrics = None;
        }

        self.materials.dispose();
        self.resources.dispose();
        self.state.dispose();

    }
}

impl GlobalContext {

    pub fn new(options: Options) -> Result<Self, Error> {
        Ok(Self {
            options,
            metrics: None,

            entry: None,
            window: None,
            instance: None,
            device: None,
            pipeline: None,

            resources: Resources::default(),
            materials: Materials::default(),
            state: State::default(),

            renderer: None,

            audio: None,
            input: None,

            tasks: Tasks::default()
        })
    }

    pub fn init()-> Result<(), Error> {

        let globals = GlobalContext::instance_mut();

        let metrics = Metrics::new();
        globals.metrics = Some(metrics);

        let entry = ash::Entry::linked();
        globals.entry = Some(entry);

        let instance = Instance::new()?;
        globals.instance = Some(instance);

        let window = Window::new()?;
        globals.window = Some(window);

        let device = Device::new()?;
        globals.device = Some(device);

        let pipeline = Pipeline::new()?;
        globals.pipeline = Some(pipeline);

        let renderer = Renderer::new(globals.options.queue_size)?;
        globals.renderer = Some(renderer);

        let audio = Audio::new()?;
        globals.audio = Some(audio);

        let input = Input::new()?;
        globals.input = Some(input);

        Ok(())
    }

    pub fn instance_mut() -> &'static mut GlobalContext {
        unsafe {
            match GLOBAL_CONTEXT {
                Some(ref mut instance) => *instance,
                None => {
                    panic!("global context not allocated");
                }
            }
        }
    }

    pub fn instance() -> &'static GlobalContext {
        unsafe {
            match GLOBAL_CONTEXT {
                Some(ref mut instance) => *instance,
                None => {
                    panic!("global context not allocated");
                }
            }
        }
    }

    pub fn alloc(options: Options) -> Result<(), Error> {
        unsafe {
            let instance_box = Box::new(GlobalContext::new(options)?);
            let instance_raw = Box::into_raw(instance_box);
            GLOBAL_CONTEXT = Some(&mut *instance_raw);
        }

        Ok(())
    }

    #[allow(static_mut_refs)]
    pub fn delete() {

        unsafe {
            // destroy global objects
            if GLOBAL_CONTEXT.is_some() {
                GLOBAL_CONTEXT.as_mut().unwrap().dispose();
            }
        }

        unsafe {
            if let Some(global_context) = std::mem::replace(&mut GLOBAL_CONTEXT, None) {
                let _ = Box::from_raw(global_context);
            }
        }
    }

}

static mut GLOBAL_CONTEXT: Option<&'static mut GlobalContext> = Option::None;

pub fn options() -> &'static crate::options::Options {
    &GlobalContext::instance().options
}

pub fn metrics() -> &'static crate::metrics::Metrics {
    GlobalContext::instance().metrics.as_ref().unwrap()
}

pub fn metrics_mut() -> &'static mut crate::metrics::Metrics {
    GlobalContext::instance_mut().metrics.as_mut().unwrap()
}

pub fn renderer() -> &'static Renderer {
    GlobalContext::instance().renderer.as_ref().unwrap()
}

pub fn renderer_mut() -> &'static mut Renderer {
    GlobalContext::instance_mut().renderer.as_mut().unwrap()
}

pub fn entry() -> &'static ash::Entry {
    GlobalContext::instance().entry.as_ref().unwrap()
}

pub fn audio() -> &'static Audio {
    GlobalContext::instance().audio.as_ref().unwrap()
}

pub fn audio_mut() -> &'static mut Audio {
    GlobalContext::instance_mut().audio.as_mut().unwrap()
}

pub fn input() -> &'static Input {
    GlobalContext::instance().input.as_ref().unwrap()
}

pub fn input_mut() -> &'static mut Input {
    GlobalContext::instance_mut().input.as_mut().unwrap()
}

pub fn window() -> &'static Window {
    GlobalContext::instance().window.as_ref().unwrap()
}

pub fn window_mut() -> &'static mut Window {
    GlobalContext::instance_mut().window.as_mut().unwrap()
}

pub fn instance() -> &'static mut Instance {
    GlobalContext::instance_mut().instance.as_mut().unwrap()
}

pub fn device() -> &'static mut Device {
    GlobalContext::instance_mut().device.as_mut().unwrap()
}

pub fn pipeline() -> &'static Pipeline {
    GlobalContext::instance().pipeline.as_ref().unwrap()
}

pub fn pipeline_mut() -> &'static mut Pipeline {
    GlobalContext::instance_mut().pipeline.as_mut().unwrap()
}

pub fn resources() -> &'static Resources {
    &GlobalContext::instance().resources
}

pub fn resources_mut() -> &'static mut Resources {
    &mut GlobalContext::instance_mut().resources
}

pub fn materials() -> &'static Materials {
    &GlobalContext::instance().materials
}

pub fn materials_mut() -> &'static mut Materials {
    &mut GlobalContext::instance_mut().materials
}

pub fn state() -> &'static State {
    &GlobalContext::instance().state
}

pub fn state_mut() -> &'static mut State {
    &mut GlobalContext::instance_mut().state
}

pub fn time() -> &'static TaskTime {
    &GlobalContext::instance().state.time
}

pub fn tasks() -> &'static Tasks {
    &GlobalContext::instance().tasks
}

pub fn tasks_mut() -> &'static mut Tasks {
    &mut GlobalContext::instance_mut().tasks
}
