//!
//! Exec
//!

use crate::api::Disposable;
use crate::api::Application;
use crate::api::Options;
use crate::api::Runnable;
use crate::error::Error;
use crate::globals::{self, GlobalContext};
use crate::input::InputEventListener;
use crate::manifest::ApplicationDescriptorTable;
use crate::material::Materials;
use crate::resources::Resources;
use crate::task::TaskDispatcher;
use crate::task::Tasks;

use std::sync::Arc;
use std::sync::Mutex;

use log::{*};

/// Exec
pub struct Exec<T: Application + Runnable + Disposable + 'static> {
    running: bool,
    dispatcher: TaskDispatcher,
    application: Arc<Mutex<T>>
}

impl <T: Application + Runnable + Disposable + 'static> Disposable for Exec<T> {
    fn dispose(&mut self) {
        trace!("Application::dispose");

        {
            let pipeline = crate::globals::pipeline_mut();
            pipeline.dispose();
        }

        self.application.lock().unwrap().dispose();
        GlobalContext::delete();
    }
}

impl <T: Application + Runnable + Disposable> InputEventListener for Exec<T> {
    fn on_keystate_change(&mut self, keystate: u32, oldstate: u32) {
        //trace!("Exec::on_keystate_change : {}", keystate);
        self.application.lock().unwrap().on_keystate_change(keystate, oldstate);
    }
}

impl <T: Application + Runnable + Disposable + 'static> Exec<T> {
    pub fn new(descriptors: &'static ApplicationDescriptorTable) -> Result<Self, Error> {
        trace!("Exec::new");

        let options = Options::from_static(descriptors.options);

        GlobalContext::alloc(options)?;
        GlobalContext::init()?;

        Resources::build(descriptors)?;
        Materials::build(descriptors.materials)?;

        let application= Arc::new(Mutex::new(T::new()?));

        Tasks::build(application.clone(), descriptors.tasks)?;

        let cycle_time_micros = 1000000u64 / (globals::options().fps as u64);
        let dispatcher = TaskDispatcher::new(cycle_time_micros);

        Self::init(&application)?;

        Ok(Self {
            running: true,
            dispatcher,
            application
        })
    }

    pub fn init(application: &Arc<Mutex<T>>) -> Result<(), Error> {
        trace!("Exec::init");

        application.lock().unwrap().on_init();
        application.lock().unwrap().on_metrics();

        let materials = globals::materials();
        if materials.len() < 1 {
            return Err(Error::from("no material defined"));
        }

        materials.compile();

        {
            // use material 0 as initial material
            let renderer = crate::globals::renderer_mut();
            renderer.set_material(&materials.get_default());
        }

        application.lock().unwrap().on_ready();

        Ok(())
    }

    fn shutdown(application: &Arc<Mutex<T>>) {
        trace!("Exec::shutdown");
        application.lock().unwrap().on_shutdown();
    }

    fn metrics_changed(application: &Arc<Mutex<T>>) {
        application.lock().unwrap().on_metrics();
    }

    fn update(application: &Arc<Mutex<T>>) {
        application.lock().unwrap().on_update();
    }

    fn draw(application: &Arc<Mutex<T>>) {
        application.lock().unwrap().on_draw();
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    fn on_event(&mut self) {

    }

    fn process_events(&mut self) -> bool {
        let window = globals::window_mut();
        return window.process_events(self);
    }

    pub fn run(&mut self) {
        trace!("Exec::run");

        {
            let tasks = globals::tasks_mut();
            tasks.start();
        }

        self.running = true;

        while self.is_running() {

            if false == self.process_events() {
                break;
            }

            self.dispatcher.sync();

            if self.dispatcher.statistics().is_updated() && globals::options().show_statistics == true {
                let stat = self.dispatcher.statistics();
                stat.print("main");
            }

            {
                // copy time to global state
                let state = crate::globals::state_mut();
                state.time = self.dispatcher.time().clone();
            }

            let reinitialized = {
                let renderer = crate::globals::renderer_mut();
                match renderer.begin_frame() {
                    Ok(reinitialized) => reinitialized,
                    Err(e) => {
                        error!("runtime failure: {}", e.message());
                        break;
                    }
                }
            };

            {
                if reinitialized {
                    Self::metrics_changed(&self.application);
                }

                Self::update(&self.application);
                Self::draw(&self.application);
            }

            {
                let renderer = crate::globals::renderer_mut();
                match renderer.end_frame() {
                    Ok(_) => {},
                    Err(e) => {
                        error!("runtime failure: {}", e.message());
                        break;
                    }
                }
            }

        }

        self.running = false;

        {
            let tasks = globals::tasks_mut();
            tasks.stop();
        }

        Self::shutdown(&self.application);

    }

}
