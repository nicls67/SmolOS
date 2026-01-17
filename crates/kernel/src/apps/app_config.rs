use crate::apps::app_config::AppStatus::{Running, Stopped};
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
            let l_period;
            let l_ends_in;
            match self.periodicity {
                CallPeriodicity::Once => {
                    l_period = Kernel::scheduler().get_period();
                    l_ends_in = Some(l_period);
                }
                CallPeriodicity::Periodic(l_p) => {
                    l_period = l_p;
                    l_ends_in = None;
                }
                CallPeriodicity::PeriodicUntil(l_p, l_e) => {
                    l_period = l_p;
                    l_ends_in = Some(l_e);
                }
            }

            let l_app_id = Kernel::scheduler().add_periodic_app(
                self.name,
                match self.app_fn {
                    CallMethod::Call(l_app) => AppCall::AppNoParam(l_app),
                    CallMethod::CallWithParam(l_app, l_param) => AppCall::AppParam(l_app, l_param),
                },
                self.init_fn,
                self.end_fn,
                l_period,
                l_ends_in,
            )?;
            self.id = Some(l_app_id);
            self.app_status = Running;

            if let Some(l_app_id_storage) = self.app_id_storage {
                l_app_id_storage(l_app_id);
            }
            Ok(l_app_id)
        } else {
            Err(KernelError::AppAlreadyScheduled(self.name))
        }
    }

    /// Stops (unschedules) this app if it is currently running.
    ///
    /// If the app is [`AppStatus::Running`], this function:
    /// - removes the corresponding periodic task from the scheduler,
    /// - notifies the terminal that the app exited (using the stored scheduler id),
    /// - updates `self.app_status` to [`AppStatus::Stopped`] and clears `self.id`.
    ///
    /// If the app is already stopped, this is a no-op.
    ///
    /// # Errors
    /// Returns any error produced by the terminal exit notifier.
    pub fn stop(&mut self) -> KernelResult<()> {
        if self.app_status == Running {
            syscall_scheduler(SysCallSchedulerArgs::RemovePeriodicTask(
                self.name,
                match self.app_fn {
                    CallMethod::Call(_) => None,
                    CallMethod::CallWithParam(_, l_param) => Some(l_param),
                },
            ))
            .unwrap_or(());
            Kernel::terminal().app_exit_notifier(self.id.unwrap())?;
            self.app_status = Stopped;
            self.id = None;
        }
        Ok(())
    }
}
