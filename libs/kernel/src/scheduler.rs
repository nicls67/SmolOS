use crate::KernelError::CannotAddNewPeriodicApp;
use crate::data::Kernel;
use crate::except::set_ticks_target;
use crate::{KernelError, KernelErrorLevel, KernelResult, Milliseconds, TerminalFormatting};
use cortex_m::peripheral::scb::{FpuAccessMode, SystemHandler};
use heapless::{String, Vec};

pub type App = fn() -> KernelResult<()>;

struct AppWrapper {
    name: String<32>,
    app: App,
    app_period: u32,
}
pub struct Scheduler {
    tasks: Vec<AppWrapper, 128>,
    cycle_counter: u32,
    sched_period: Milliseconds,
}

impl Scheduler {
    pub fn new(period: Milliseconds) -> Scheduler {
        Scheduler {
            tasks: Vec::new(),
            cycle_counter: 0,
            sched_period: period,
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
            })
            .map_err(|_| CannotAddNewPeriodicApp(app_name))
    }

    pub fn periodic_task(&mut self) -> KernelResult<()> {
        for task in self.tasks.iter() {
            if self.cycle_counter % task.app_period == 0 {
                (task.app)()?;
            }
        }
        self.cycle_counter += 1;
        Ok(())
    }
}
