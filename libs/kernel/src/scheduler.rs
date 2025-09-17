use crate::KernelError::CannotAddNewPeriodicApp;
use crate::data::Kernel;
use crate::except::set_ticks_target;
use crate::{KernelError, KernelResult, Milliseconds, TerminalFormatting};
use cortex_m::peripheral::scb::SystemHandler;
use heapless::{String, Vec};

pub type App = fn() -> KernelResult<()>;

struct AppWrapper {
    name: String<32>,
    app: App,
    app_period: u32,
    active: bool,
}
pub struct Scheduler {
    tasks: Vec<AppWrapper, 128>,
    cycle_counter: u32,
    sched_period: Milliseconds,
    pub started: bool,
    current_task_id: Option<usize>,
    current_task_has_error: bool,
}

impl Scheduler {
    pub fn new(period: Milliseconds) -> Scheduler {
        Scheduler {
            tasks: Vec::new(),
            cycle_counter: 0,
            sched_period: period,
            started: false,
            current_task_id: None,
            current_task_has_error: false,
        }
    }

    pub fn start(&mut self, systick_period: Milliseconds) -> KernelResult<()> {
        let cortex_p = Kernel::cortex_peripherals();

        // Initialize scheduler periodic IT
        unsafe {
            cortex_p.SCB.set_priority(SystemHandler::PendSV, 0xFF);
            set_ticks_target(self.sched_period.to_u32() / systick_period.to_u32())
        }

        cortex_p.SYST.enable_counter();
        self.started = true;
        Kernel::terminal().write(&TerminalFormatting::StrNewLineBoth("Scheduler started !"))
    }

    pub fn add_periodic_app(
        &mut self,
        name: &str,
        app: App,
        init: App,
        period: Milliseconds,
    ) -> KernelResult<()> {
        let app_name = String::from(name.parse().unwrap());

        // Try to initialize the app
        init().map_err(|_| KernelError::AppInitError(app_name.clone()))?;

        // Register app in the scheduler
        self.tasks
            .push(AppWrapper {
                name: app_name.clone(),
                app,
                app_period: period.to_u32() / self.sched_period.to_u32(),
                active: true,
            })
            .map_err(|_| CannotAddNewPeriodicApp(app_name))
    }

    pub fn periodic_task(&mut self) {
        // Run all tasks
        for (id, task) in self.tasks.iter_mut().enumerate() {
            if self.cycle_counter % task.app_period == 0 && task.active {
                self.current_task_id = Some(id);
                self.current_task_has_error = false;
                match (task.app)() {
                    Ok(..) => {}
                    Err(e) => {
                        if !self.current_task_has_error {
                            Kernel::errors().error_handler(&e);
                        }
                    }
                }
                self.current_task_id = None;
            }
        }

        self.cycle_counter += 1;
    }

    pub fn abort_task(&mut self) {
        // Set the current task as inactive
        if let Some(id) = self.current_task_id {
            self.tasks[id].active = false;
            self.current_task_has_error = true;
        }
    }
}
