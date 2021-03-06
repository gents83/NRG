use std::{
    any::TypeId,
    collections::HashMap,
    path::PathBuf,
    sync::{
        mpsc::{channel, Receiver},
        Arc, Mutex, RwLock,
    },
};

use nrg_messenger::MessengerRw;
use nrg_platform::{InputState, Key, KeyEvent, WindowEvent};
use nrg_resources::SharedDataRw;

use crate::{Job, JobHandler, JobHandlerRw, Phase, PluginId, PluginManager, Scheduler, Worker};

const NUM_WORKER_THREADS: usize = 5;

pub struct App {
    is_profiling: bool,
    is_enabled: bool,
    shared_data: SharedDataRw,
    global_messenger: MessengerRw,
    plugin_manager: PluginManager,
    scheduler: Scheduler,
    workers: HashMap<String, Worker>,
    job_handler: Arc<RwLock<JobHandler>>,
    receiver: Arc<Mutex<Receiver<Job>>>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.stop_worker_threads();

        if self.is_profiling {
            nrg_profiler::write_profile_file!();
        }

        self.scheduler.uninit();

        let plugins_to_remove = self.plugin_manager.release();
        self.update_plugins(plugins_to_remove, false);
    }
}

impl App {
    pub fn new() -> Self {
        nrg_profiler::create_profiler!();

        let (sender, receiver) = channel();

        let mut app = Self {
            is_enabled: true,
            is_profiling: false,
            scheduler: Scheduler::new(),
            plugin_manager: PluginManager::new(),
            workers: HashMap::new(),
            job_handler: JobHandler::new(sender),
            receiver: Arc::new(Mutex::new(receiver)),
            shared_data: SharedDataRw::default(),
            global_messenger: MessengerRw::default(),
        };

        app.setup_worker_threads();

        app
    }

    fn setup_worker_threads(&mut self) {
        for i in 1..NUM_WORKER_THREADS + 1 {
            self.add_worker(format!("Worker{}", i).as_str());
        }
    }

    fn stop_worker_threads(&mut self) {
        for (_name, w) in self.workers.iter_mut() {
            w.stop();
        }
    }

    fn update_plugins(&mut self, plugins_to_remove: Vec<PluginId>, reload: bool) {
        for id in plugins_to_remove.iter() {
            if let Some(plugin_data) = self.plugin_manager.remove_plugin(id) {
                let lib_path = plugin_data.original_path.clone();
                PluginManager::clear_plugin_data(plugin_data, self);
                if reload {
                    let reloaded_plugin_data = PluginManager::create_plugin_data(lib_path, self);
                    self.plugin_manager.add_plugin(reloaded_plugin_data);
                }
            }
        }
    }

    fn update_events(&mut self) {
        nrg_profiler::scoped_profile!("app::update_events");

        let mut is_profiling = self.is_profiling;
        let mut is_enabled = self.is_enabled;
        self.global_messenger
            .read()
            .unwrap()
            .process_messages(|msg| {
                if msg.type_id() == TypeId::of::<KeyEvent>() {
                    let e = msg.as_any().downcast_ref::<KeyEvent>().unwrap();
                    if e.code == Key::F9 && e.state == InputState::JustPressed {
                        if !is_profiling {
                            nrg_profiler::start_profiler!();
                            is_profiling = true;
                        } else {
                            is_profiling = false;
                            nrg_profiler::stop_profiler!();
                            nrg_profiler::write_profile_file!();
                        }
                    }
                } else if msg.type_id() == TypeId::of::<WindowEvent>() {
                    let e = msg.as_any().downcast_ref::<WindowEvent>().unwrap();
                    match e {
                        WindowEvent::Show => {
                            is_enabled = true;
                        }
                        WindowEvent::Hide => {
                            is_enabled = false;
                        }
                        _ => {}
                    }
                }
            });
        self.is_profiling = is_profiling;
        if self.is_enabled && !is_enabled {
            self.stop_worker_threads();
        } else if !self.is_enabled && is_enabled {
            self.setup_worker_threads();
        }
        self.is_enabled = is_enabled;
    }

    pub fn run_once(&mut self) -> bool {
        nrg_profiler::scoped_profile!("app::run_frame");

        let can_continue = self
            .scheduler
            .run_once(self.is_enabled, self.get_job_handler());

        if !self.is_enabled {
            let plugins_to_remove = self.plugin_manager.update();
            self.update_plugins(plugins_to_remove, true);
        } else {
            self.job_handler.read().unwrap().clear_pending_jobs();
        }

        self.update_events();

        self.shared_data.write().unwrap().flush_resources();

        can_continue
    }

    pub fn run(&mut self) {
        loop {
            let can_continue = self.run_once();

            if !can_continue {
                break;
            }
        }
    }

    pub fn add_plugin(&mut self, lib_path: PathBuf) {
        let plugin_data = PluginManager::create_plugin_data(lib_path, self);
        self.plugin_manager.add_plugin(plugin_data);
    }

    pub fn remove_plugin(&mut self, plugin_id: &PluginId) {
        if let Some(plugin_data) = self.plugin_manager.remove_plugin(plugin_id) {
            PluginManager::clear_plugin_data(plugin_data, self);
        }
    }
    fn add_worker(&mut self, name: &str) -> &mut Worker {
        let key = String::from(name);
        let job_handler = self.get_job_handler();
        let w = self.workers.entry(key).or_insert_with(Worker::default);
        if !w.is_started() {
            w.start(name, job_handler, self.receiver.clone());
        }
        w
    }
    fn get_worker(&mut self, name: &str) -> &mut Worker {
        let key = String::from(name);
        self.workers.get_mut(&key).unwrap()
    }

    pub fn get_job_handler(&self) -> JobHandlerRw {
        self.job_handler.clone()
    }
    pub fn get_shared_data(&self) -> SharedDataRw {
        self.shared_data.clone()
    }
    pub fn get_global_messenger(&self) -> MessengerRw {
        self.global_messenger.clone()
    }
    pub fn create_phase_on_worker<P: Phase>(&mut self, phase: P, name: &'static str) {
        let w = self.add_worker(name);
        w.create_phase(phase);
    }
    pub fn destroy_phase_on_worker(&mut self, phase_name: &str, name: &str) {
        let w = self.get_worker(name);
        w.destroy_phase(phase_name);
    }
    pub fn create_phase_before<P: Phase>(&mut self, phase: P, previous_phase_name: &str) {
        self.scheduler
            .create_phase_before(phase, previous_phase_name);
    }
    pub fn create_phase<P: Phase>(&mut self, phase: P) {
        self.scheduler.create_phase(phase);
    }
    pub fn destroy_phase(&mut self, phase_name: &str) {
        self.scheduler.destroy_phase(phase_name);
    }
    pub fn get_phase<P: Phase>(&mut self, phase_name: &str) -> &P {
        self.scheduler.get_phase(phase_name)
    }
    pub fn get_phase_mut<P: Phase>(&mut self, phase_name: &str) -> &mut P {
        self.scheduler.get_phase_mut(phase_name)
    }
}
