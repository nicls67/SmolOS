use crate::apps_manager::app_config::AppStatus::{Running, Stopped};
use crate::data::Kernel;
use crate::scheduler::{App, AppCall, AppParam};
use crate::{KernelError, KernelResult, Milliseconds, SysCallSchedulerArgs, syscall_scheduler};

#[derive(Copy, Clone)]
pub enum CallPeriodicity {
    Once,
    Periodic(Milliseconds),
    PeriodicUntil(Milliseconds, Milliseconds),
}

#[derive(Copy, Clone)]
pub enum CallMethod {
    Call(App),
    CallWithParam(AppParam, u32),
}

#[derive(PartialEq, Copy, Clone)]
pub enum AppStatus {
    Running,
    Stopped,
}

impl AppStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Running => "Running",
            Stopped => "Stopped",
        }
    }
}

#[derive(Copy, Clone)]
pub struct AppConfig {
    pub name: &'static str,
    pub periodicity: CallPeriodicity,
    pub app_fn: CallMethod,
    pub init_fn: Option<App>,
    pub end_fn: Option<App>,
    pub app_status: AppStatus,
    pub id: Option<u32>,
    pub app_id_storage: Option<fn(u32)>,
}

impl AppConfig {
    /// Starts (schedules) this app if it is currently stopped.
    ///
    /// This registers the configured app with the kernel scheduler according to its
    /// [`CallPeriodicity`] and [`CallMethod`].
    ///
    /// - [`CallPeriodicity::Once`]: schedules the app to run once (using the scheduler period).
    /// - [`CallPeriodicity::Periodic`]: schedules the app to run indefinitely at the given period.
    /// - [`CallPeriodicity::PeriodicUntil`]: schedules the app to run at the given period until
    ///   the provided duration elapses.
    ///
    /// On success, this function:
    /// - stores the returned scheduler id in `self.id`,
    /// - updates `self.app_status` to [`AppStatus::Running`],
    /// - calls `self.app_id_storage` (if provided) with the assigned id.
    ///
    /// # Returns
    /// The scheduler id assigned to the app.
    ///
    /// # Errors
    /// Returns [`KernelError::AppAlreadyScheduled`] if the app is already running/scheduled.
    pub fn start(&mut self) -> KernelResult<u32> {
        if self.app_status == Stopped {
            let period;
            let ends_in;
            match self.periodicity {
                CallPeriodicity::Once => {
                    period = Kernel::scheduler().get_period();
                    ends_in = Some(period);
                }
                CallPeriodicity::Periodic(p) => {
                    period = p;
                    ends_in = None;
                }
                CallPeriodicity::PeriodicUntil(p, e) => {
                    period = p;
                    ends_in = Some(e);
                }
            }

            let app_id = Kernel::scheduler().add_periodic_app(
                self.name,
                match self.app_fn {
                    CallMethod::Call(app) => AppCall::AppNoParam(app, self.end_fn),
                    CallMethod::CallWithParam(app, param) => {
                        AppCall::AppParam(app, param, self.end_fn)
                    }
                },
                self.init_fn,
                period,
                ends_in,
            )?;
            self.id = Some(app_id);
            self.app_status = Running;

            if let Some(app_id_storage) = self.app_id_storage {
                app_id_storage(app_id);
            }
            Ok(app_id)
        } else {
            Err(KernelError::AppAlreadyScheduled(self.name))
        }
    }

    /// Stops (unschedules) this app if it is currently running.
    ///
    /// If the app is [`AppStatus::Running`], this removes the periodic task from the scheduler
    /// (matching by `self.name` and, when applicable, the configured parameter), then:
    /// - sets `self.app_status` to [`AppStatus::Stopped`],
    /// - clears `self.id`.
    ///
    /// If the app is already stopped, this is a no-op.
    ///
    /// # Errors
    /// Propagates any scheduler error encountered while removing the task.
    pub fn stop(&mut self) -> KernelResult<()> {
        if self.app_status == Running {
            syscall_scheduler(SysCallSchedulerArgs::RemovePeriodicTask(
                self.name,
                match self.app_fn {
                    CallMethod::Call(_) => None,
                    CallMethod::CallWithParam(_, param) => Some(param),
                },
            ))?;
            self.app_status = Stopped;
            self.id = None;
        }
        Ok(())
    }
}
