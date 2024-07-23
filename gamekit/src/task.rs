//!
//! Task
//!

use log::{*};

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::MutexGuard;
use std::thread;
use std::sync::Arc;
use std::sync::Mutex;

use crate::api::Disposable;
use crate::api::LockRef;
use crate::api::Runnable;
use crate::error::Error;
use crate::globals;
use crate::manifest::StaticTaskDescriptor;

// even if there is a timer overrun, sleep at least 1 millisecond
//const MIN_SLEEP_DURATION: std::time::Duration = std::time::Duration::from_micros(1000u64);

/// Task info
#[derive(Clone, Debug)]
pub struct TaskInfo {
    name: String,
    id: u32
}

impl Default for TaskInfo {
    fn default() -> Self {
        Self {
            name: String::from(""),
            id: 0
        }
    }
}

impl TaskInfo {
    pub fn new(name: &str, id: u32) -> Self {
        Self {
            name: String::from(name),
            id
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}

/// Task time
#[derive(Clone, Debug)]
pub struct TaskTime {
    pub time: f32,
    pub delta: f32,
    pub step: f32
}

impl Default for TaskTime {
    fn default() -> Self {
        Self {
            time: 0.0,
            delta: 0.0,
            step: 0.0
        }
    }
}

impl TaskTime {
    pub fn set(&mut self, task_time: &TaskTime) {
        self.time = task_time.time;
        self.delta = task_time.delta;
        self.step = task_time.step;
    }
}

/// Task context
#[derive(Clone, Debug)]
pub struct TaskContext {
    info: TaskInfo,
    time: TaskTime
}

impl Default for TaskContext {
    fn default() -> Self {
        Self {
            info: TaskInfo::default(),
            time: TaskTime::default()
        }
    }
}

impl TaskContext {
    pub fn new(name: &str, id: u32) -> Self {
        Self {
            info: TaskInfo::new(name, id),
            time: TaskTime::default()
        }
    }

    pub fn name(&self) -> &str {
        self.info.name()
    }

    pub fn id(&self) -> u32 {
        self.info.id()
    }

    pub fn time(&self) -> &TaskTime {
        &self.time
    }

    pub fn set_time(&mut self, task_time: &TaskTime) {
        self.time.set(task_time);
    }
}


/// Task statistics
pub struct TaskStatistics {
    updated: bool,

    pub last_update_time: std::time::Instant,

    pub update_counter: u64,
    pub last_update_counter: u64,
    pub avg_updates_per_second: f64,

    pub usage_counter: u64,
    pub last_usage_counter: u64,
    pub avg_frame_time: u64
}

impl TaskStatistics {
    pub fn new() -> Self {
        Self {
            updated: false,
            last_update_time: std::time::Instant::now(),
            update_counter: 0,
            last_update_counter: 0,
            avg_updates_per_second: 0.0,
            usage_counter: 0,
            last_usage_counter: 0,
            avg_frame_time: 0
        }
    }

    pub fn is_updated(&self) -> bool {
        self.updated
    }

    pub fn print(&self, label: &str) {
        if label.len() > 0 {
            debug!("[{}] {:.1} updates/sec ({}us avg. step time)", label, self.avg_updates_per_second, self.avg_frame_time);
        } else {
            debug!("{:.1} updates/sec ({}us avg. step time)", self.avg_updates_per_second, self.avg_frame_time);
        }
    }

    pub fn print_named(&self, name: &str, id: u32) {
        let label = if name.len() > 0 {
            format!("#{}:{}", id, name)
        } else {
            format!("{}", id)
        };

        self.print(&label);
    }
}

/// Task dispatcher
pub struct TaskDispatcher {
    first: bool,
    t_abs_start: std::time::Instant,
    t_cycle: std::time::Duration,
    t_start: std::time::Instant,
    t_delta: std::time::Duration,
    t_frame_last: std::time::Instant,
    t_frame_start: std::time::Instant,
    t_frame_delta: std::time::Duration,
    time: TaskTime,
    statistics: TaskStatistics,
}

impl TaskDispatcher {
    pub fn new(cycle_time_micros: u64) -> Self {
        let t_abs_start = std::time::Instant::now();
        let t_cycle = std::time::Duration::from_micros(cycle_time_micros);
        let t_start = t_abs_start.clone();
        let t_delta = std::time::Duration::from_micros(0u64);
        let t_frame_last = t_start.clone();
        let t_frame_start = t_frame_last.clone();
        let t_frame_delta = std::time::Duration::from_micros(0u64);

        Self {
            first: true,
            t_abs_start,
            t_cycle,
            t_start,
            t_delta,
            t_frame_last,
            t_frame_start,
            t_frame_delta,
            time: TaskTime::default(),
            statistics: TaskStatistics::new(),
        }
    }

    pub fn statistics(&self) -> &TaskStatistics {
        return &self.statistics;
    }

    fn update_statistics(stat: &mut TaskStatistics, current_time: std::time::Instant, frame_time: std::time::Duration) {

        stat.update_counter += 1;
        stat.usage_counter += frame_time.as_micros() as u64;

        let elapsed_duration = (current_time - stat.last_update_time).as_secs_f64();

        if elapsed_duration >= 3.0 && stat.update_counter > stat.last_update_counter {

            stat.updated = true;

            stat.last_update_time = current_time;

            let delta = stat.update_counter - stat.last_update_counter;
            stat.last_update_counter = stat.update_counter;
            stat.avg_updates_per_second = (delta as f64) / elapsed_duration;

            let delta = stat.usage_counter - stat.last_usage_counter;
            stat.last_usage_counter = stat.usage_counter;

            let num_frames_f = stat.avg_updates_per_second.max(1.0);
            stat.avg_frame_time = ((delta as f64) / elapsed_duration / num_frames_f) as u64;
        } else {
            stat.updated = false;
        }

    }

    fn begin(&mut self) {
        let now = std::time::Instant::now();
        self.t_delta = now - self.t_abs_start;
        self.t_frame_start = now;
        self.t_frame_delta = self.t_frame_start - self.t_frame_last;
        self.update_time();
    }

    fn end(&mut self) {
        if self.first {
            self.first = false;
            return;
        }

        let t_now = std::time::Instant::now();
        let t_elapsed = t_now - self.t_frame_start;
        self.t_frame_last = self.t_frame_start;

        Self::update_statistics(&mut self.statistics, t_now, t_elapsed);

        let t_next = self.t_start + self.t_cycle;

        if t_now < t_next {
            let t_sleep = t_next - t_now;
            std::thread::sleep(t_sleep);
            self.t_start = t_next;
        } else {
            let t_overrun = t_now - t_next;
            let t_overrun_micros = t_overrun.as_micros();

            if t_overrun_micros > 3000 {
                self.t_start = t_now; // skip frames
                // warn if a 3 millisecond threshold is exceeded
                warn!("frame time overrun by {t_overrun_micros}us");
            } else {
                self.t_start = t_next;
            }

            // make sure a bit of time is reserved for other threads
            //std::thread::sleep(MIN_SLEEP_DURATION);
            std::thread::yield_now();
        }

    }

    pub fn sync(&mut self) -> &std::time::Duration {
        self.end();
        self.begin();
        return &self.t_frame_delta;
    }

    pub fn update_time(&mut self) {
        self.time.time = self.t_delta.as_secs_f32();
        self.time.delta = self.t_frame_delta.as_secs_f32();

        let step = self.t_frame_delta.min(self.t_cycle);
        self.time.step = step.as_secs_f32();
    }

    pub fn time(&self) -> &TaskTime {
        &self.time
    }

}

/// Task
pub struct Task {
    info: TaskInfo,
    handle: Option<std::thread::JoinHandle<()>>,
    running: Arc<Mutex<AtomicBool>>,
    runnable: Arc<Mutex<dyn Runnable>>,
    dispatcher: Arc<Mutex<TaskDispatcher>>
}

pub type TaskRef = std::sync::Arc<Task>;
pub type TaskLockRef = LockRef<Task>;

impl Disposable for Task {
    fn dispose(&mut self) {
        self.stop();
    }
}

impl Task {
    fn build(runnable: Arc<Mutex<dyn Runnable>>, cycle_time_micros: u64, name: &str, id: u32) -> Self {
        trace!("Task::build");

        let info = TaskInfo::new(name, id);

        let dispatcher = TaskDispatcher::new(cycle_time_micros);
        let dispatcher_ref = Arc::new(Mutex::new(dispatcher));

        Self {
            info,
            handle: None,
            running: Arc::new(Mutex::new(AtomicBool::new(false))),
            runnable: runnable.clone(),
            dispatcher: dispatcher_ref
        }
    }

    pub fn new(runnable: Arc<Mutex<dyn Runnable>>, cycle_time_micros: u64) -> Self {
        trace!("Task::new");
        Self::build(runnable, cycle_time_micros, "", 0)
    }

    pub fn from_static(runnable: Arc<Mutex<dyn Runnable>>, descriptor: &StaticTaskDescriptor) -> Self {
        trace!("Task::from_static");
        Self::build(
            runnable,
            descriptor.interval,
            &descriptor.name,
            descriptor.id
        )
    }

    pub fn to_lockref(task: Self) -> TaskLockRef {
        Arc::new(Mutex::new(task))
    }

    pub fn name(&self) -> &str {
        self.info.name()
    }

    pub fn set_name(&mut self, name: &str) {
        self.info.name = String::from(name);
    }

    pub fn id(&self) -> u32 {
        self.info.id()
    }

    pub fn set_id(&mut self, id: u32) {
        self.info.id = id;
    }

    pub fn start(&mut self) {

        trace!("Task::start");

        let running_ref = self.running.clone();
        let runnable_ref = self.runnable.clone();
        let dispatcher_ref = self.dispatcher.clone();

        let task_context = TaskContext::new(self.info.name(), self.info.id());

        self.handle = Some(thread::spawn(move || {
            Self::thread_loop(task_context, running_ref, runnable_ref, dispatcher_ref);
        }));

    }

    pub fn stop(&mut self) {

        trace!("Task::stop");

        let mut handle: Option<std::thread::JoinHandle<()>> = None;

        trace!("Task::stop - lock state");
        self.running.lock().unwrap().store(false, Ordering::Relaxed);

        trace!("Task::stop - take handle");
        if self.handle.is_some() {
            handle = self.handle.take();
        }

        if handle.is_none() {
            return;
        }

        trace!("Task::stop - join");
        let _ = handle.unwrap().join();
    }

    fn thread_step(
        task_context: &mut TaskContext,
        runnable_ref: &Arc<Mutex<dyn Runnable>>,
        dispatcher_ref: &Arc<Mutex<TaskDispatcher>>
    ) -> bool {

        let running: bool;

        {
            let mut dispatcher = dispatcher_ref.lock().unwrap();
            dispatcher.sync();
            task_context.set_time(dispatcher.time());

            if globals::options().show_statistics == true {
                if dispatcher.statistics().is_updated() {
                    let stat = dispatcher.statistics();
                    stat.print_named(task_context.name(), task_context.id());
                }
            }
        }

        {
            let mut runnable = runnable_ref.lock().unwrap();
            runnable.run();
            runnable.run_delta(task_context);
            running = runnable.is_running();
        }

        return running;
    }

    fn thread_loop(
        mut task_context: TaskContext,
        running_ref: Arc<Mutex<AtomicBool>>,
        runnable_ref: Arc<Mutex<dyn Runnable>>,
        dispatcher_ref: Arc<Mutex<TaskDispatcher>>
    ) {

        trace!("Task::thread_loop enter");

        let mut running_flag = true;

        running_ref.lock().unwrap().store(running_flag, Ordering::Relaxed);
        runnable_ref.lock().unwrap().start();

        while running_flag {

            if running_ref.lock().unwrap().load(Ordering::Relaxed) == false {
                break;
            }

            if !running_flag {
                break;
            }

            running_flag = Self::thread_step(&mut task_context, &runnable_ref, &dispatcher_ref);
        }

        running_ref.lock().unwrap().store(false, Ordering::Relaxed);
        runnable_ref.lock().unwrap().stop();

        trace!("Task::thread_loop exit");

    }

    pub fn is_running(&self) -> bool {
        return self.running.lock().unwrap().load(Ordering::Relaxed);
    }

}

/// Synchronous task
pub struct SyncTask {
    context: TaskContext,
    running: bool,
    runnable: Box<dyn Runnable>,
    dispatcher: TaskDispatcher
}

impl SyncTask {
    fn build(runnable: Box<dyn Runnable>, cycle_time_micros: u64, name: &str, id: u32) -> Self {
        trace!("SyncTask::build");
        Self {
            context: TaskContext::new(name, id),
            running: false,
            runnable,
            dispatcher: TaskDispatcher::new(cycle_time_micros)
        }
    }

    pub fn new(runnable: Box<dyn Runnable>, cycle_time_micros: u64) -> Self {
        trace!("SyncTask::new");
        Self::build(runnable, cycle_time_micros, "", 0)
    }

    pub fn name(&self) -> &str {
        self.context.name()
    }

    pub fn set_name(&mut self, name: &str) {
        self.context.info.name = String::from(name);
    }

    pub fn id(&self) -> u32 {
        self.context.id()
    }

    pub fn set_id(&mut self, id: u32) {
        self.context.info.id = id;
    }

    pub fn run(&mut self) {

        trace!("SyncTask::run loop enter");

        self.running = true;

        while self.is_running() {
            self.step()
        }

        self.running = false;

    }

    pub fn step(&mut self) {
        self.dispatcher.sync();

        if globals::options().show_statistics == true {
            if self.dispatcher.statistics().is_updated() {
                let stat = self.dispatcher.statistics();
                stat.print_named(self.context.name(), self.context.id());
            }
        }

        self.runnable.run();

        self.context.set_time(self.dispatcher.time());
        self.runnable.run_delta(&self.context);
    }

    pub fn get_time(&self) -> &TaskTime {
        self.dispatcher.time()
    }

    pub fn is_running(&self) -> bool {
        return self.running && self.runnable.is_running();
    }

}

/// Asynchrounous caller
pub struct AsyncCaller {
    info: TaskInfo,
    handle: Option<std::thread::JoinHandle<()>>,
    running: Arc<Mutex<AtomicBool>>,
    callee: Arc<Mutex<fn()>>,
    dispatcher: Arc<Mutex<TaskDispatcher>>
}

pub type AsyncCallerRef = std::sync::Arc<AsyncCaller>;
pub type AsyncCallerLockRef = LockRef<AsyncCaller>;

impl AsyncCaller {
    fn build(callee: Arc<Mutex<fn()>>, cycle_time_micros: u64, name: &str, id: u32) -> Arc<Mutex<Self>> {
        trace!("AsyncCaller::build");

        let dispatcher = TaskDispatcher::new(cycle_time_micros);
        let dispatcher_ref = Arc::new(Mutex::new(dispatcher));

        Arc::new(Mutex::new(Self {
            info: TaskInfo::new(name, id),
            running: Arc::new(Mutex::new(AtomicBool::new(false))),
            handle: None,
            callee,
            dispatcher: dispatcher_ref
        }))
    }

    pub fn new(callee: Arc<Mutex<fn()>>, cycle_time_micros: u64) -> Arc<Mutex<Self>> {
        trace!("AsyncCaller::new");
        Self::build(callee, cycle_time_micros, "", 0)
    }

    pub fn name(&self) -> &str {
        self.info.name()
    }

    pub fn set_name(&mut self, name: &str) {
        self.info.name = String::from(name);
    }

    pub fn id(&self) -> u32 {
        self.info.id()
    }

    pub fn set_id(&mut self, id: u32) {
        self.info.id = id;
    }

    pub fn start(&mut self) {

        trace!("AsyncCaller::start");

        let running_ref = self.running.clone();
        let callee_ref = self.callee.clone();
        let dispatcher_ref = self.dispatcher.clone();

        let info = self.info.clone();

        self.handle = Some(thread::spawn(move || {
            Self::thread_loop(info, running_ref, callee_ref, dispatcher_ref);
        }));

    }

    pub fn stop(&mut self) {

        trace!("AsyncCaller::stop");

        let mut handle: Option<std::thread::JoinHandle<()>> = None;

        trace!("AsyncCaller::lock state");
        self.running.lock().unwrap().store(false, Ordering::Relaxed);

        trace!("AsyncCaller::take handle");
        if self.handle.is_some() {
            handle = self.handle.take();
        }

        if handle.is_none() {
            return;
        }

        trace!("AsyncCaller::join");
        let _ = handle.unwrap().join();

    }

    fn thread_loop(
        info: TaskInfo,
        running_ref: Arc<Mutex<AtomicBool>>,
        callee_ref: Arc<Mutex<fn()>>,
        dispatcher_ref: Arc<Mutex<TaskDispatcher>>
    ) {

        trace!("AsyncCaller::thread_loop enter");

        running_ref.lock().unwrap().store(true, Ordering::Relaxed);
    
        loop {

            if running_ref.lock().unwrap().load(Ordering::Relaxed) == false {
                break;
            }

            {
                let mut dispatcher = dispatcher_ref.lock().unwrap();
                dispatcher.sync();

                if globals::options().show_statistics == true {
                    if dispatcher.statistics().is_updated() {
                        let stat = dispatcher.statistics();
                        stat.print_named(info.name(), info.id());
                    }
                }                
            }

            {
                let callee = callee_ref.lock().unwrap();
                callee();
            }

        }

        running_ref.lock().unwrap().store(false, Ordering::Relaxed);

        trace!("AsyncCaller::thread_loop exit");

    }

    pub fn is_running(&self) -> bool {
        return self.running.lock().unwrap().load(Ordering::Relaxed);
    }

}


pub struct Tasks {
    tasks: HashMap<String, TaskLockRef>
}

impl Disposable for Tasks {
    fn dispose(&mut self) {
        trace!("Tasks::dispose");

        for (_, task) in &mut self.tasks {
            task.lock().unwrap().dispose();
        }

        self.tasks.clear();
    }
}

impl Default for Tasks {
    fn default() -> Self {
        Self {
            tasks: HashMap::new()
        }
    }
}

impl Tasks {

    pub fn build(runnable: Arc<Mutex<dyn Runnable>>, descriptors: &'static [StaticTaskDescriptor]) -> Result<(), Error> {

        let tasks = crate::globals::tasks_mut();

        for descriptor in descriptors {
            let task = Task::from_static(runnable.clone(), descriptor);
            tasks.add_task(descriptor.name, task);
        }

        Ok(())
    }

    pub fn start(&mut self) {
        for (_, task) in &self.tasks {
            task.lock().unwrap().start();
        }
    }

    pub fn stop(&mut self) {
        for (_, task) in &self.tasks {
            task.lock().unwrap().stop();
        }
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn get(&self, id: &str) -> TaskLockRef {
        let material_ref = self.tasks.get(id).expect("material not found");
        material_ref.clone()
    }

    pub fn get_lock(&self, id: &str) -> MutexGuard<Task> {
        let material_ref = self.tasks.get(id).expect("material not found");
        material_ref.lock().unwrap()
    }

    pub fn add_task(&mut self, name: &str, task: Task) -> TaskLockRef {
        let task_ref = Task::to_lockref(task);
        self.tasks.insert(name.to_string(), task_ref.clone());
        task_ref
    }

}
