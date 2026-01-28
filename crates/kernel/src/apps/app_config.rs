use heapless::{String, Vec};

use crate::apps::app_config::AppStatus::{Running, Stopped};
use crate::data::Kernel;
use crate::scheduler::App;
use crate::{KernelError, KernelResult, Milliseconds};

/// Maximum number of parameters accepted after the app name.
pub const K_MAX_APP_PARAMS: usize = 8;
/// Maximum byte length for each parameter (ASCII expected).
pub const K_MAX_APP_PARAM_SIZE: usize = 16;

#[derive(Copy, Clone)]
pub enum CallPeriodicity {
    Once,
    Periodic(Milliseconds),
    PeriodicUntil(Milliseconds, Milliseconds),
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
    pub app_fn: App,
    /// Optional initialization hook invoked before scheduling the app.
    /// Receives the assigned scheduler id and parsed parameters.
    pub init_fn:
        Option<fn(u32, Vec<String<K_MAX_APP_PARAM_SIZE>, K_MAX_APP_PARAMS>) -> KernelResult<()>>,
    pub end_fn: Option<App>,
    pub app_status: AppStatus,
    pub id: Option<u32>,
}

impl AppConfig {
    /// Starts (schedules) this app if it is currently stopped.
    ///
    /// This registers the configured app with the kernel scheduler according to its
    /// [`CallPeriodicity`] and `app_fn`.
    ///
    /// - [`CallPeriodicity::Once`]: schedules the app to run once (using the scheduler period).
    /// - [`CallPeriodicity::Periodic`]: schedules the app to run indefinitely at the given period.
    /// - [`CallPeriodicity::PeriodicUntil`]: schedules the app to run at the given period until
    ///   the provided duration elapses.
    ///
    /// On success, this function:
    /// - stores the returned scheduler id in `self.id`,
    /// - updates `self.app_status` to [`AppStatus::Running`],
    /// - calls `self.init_fn` (if provided) before scheduling the app, passing the assigned id
    ///   and parsed parameters.
    ///
    /// # Arguments
    /// * `p_app_param` - The full app parameter string captured at launch time. Parameters are
    ///   parsed by ASCII whitespace and the first token (app name) is ignored.
    ///
    /// # Returns
    /// The scheduler id assigned to the app.
    ///
    /// # Errors
    /// Returns [`KernelError::AppAlreadyScheduled`] if the app is already running/scheduled.
    /// Returns [`KernelError::AppParamTooLong`] if any parameter exceeds
    /// [`K_MAX_APP_PARAM_SIZE`], [`KernelError::TooManyAppParams`] if the
    /// parameter count exceeds [`K_MAX_APP_PARAMS`], or
    /// [`KernelError::AppNeedsNoParam`] if parameters are provided while no
    /// no `init_fn` hook is configured.
    pub fn start(&mut self, p_app_param: &str) -> KernelResult<u32> {
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
                self.app_fn,
                self.end_fn,
                l_period,
                l_ends_in,
            )?;
            self.id = Some(l_app_id);
            self.app_status = Running;

            // Store app parameters in a Vec
            let mut l_param_vec: Vec<String<K_MAX_APP_PARAM_SIZE>, K_MAX_APP_PARAMS> = Vec::new();

            for l_param in p_app_param.split_ascii_whitespace().skip(1) {
                let mut l_entry = String::<K_MAX_APP_PARAM_SIZE>::new();
                l_entry.push_str(l_param).map_err(|_| {
                    Kernel::scheduler().remove_periodic_app(self.name).unwrap();
                    self.id = None;
                    self.app_status = Stopped;
                    KernelError::AppParamTooLong
                })?;
                l_param_vec.push(l_entry).map_err(|_| {
                    Kernel::scheduler().remove_periodic_app(self.name).unwrap();
                    self.id = None;
                    self.app_status = Stopped;
                    KernelError::TooManyAppParams
                })?;
            }

            // Call initialization function if provided
            if let Some(l_init_func) = self.init_fn {
                match l_init_func(l_app_id, l_param_vec) {
                    Ok(_) => (),
                    Err(_l_err) => {
                        Kernel::scheduler().remove_periodic_app(self.name).unwrap();
                        self.id = None;
                        self.app_status = Stopped;
                        return Err(KernelError::AppInitError(self.name));
                    }
                };
            }
            // No param is expected but received some
            else if !l_param_vec.is_empty() {
                Kernel::scheduler().remove_periodic_app(self.name).unwrap();
                self.id = None;
                self.app_status = Stopped;
                return Err(KernelError::AppNeedsNoParam(self.name));
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
            Kernel::scheduler().remove_periodic_app(self.name)?;
            Kernel::terminal().app_exit_notifier(self.id.unwrap())?;
            self.app_status = Stopped;
            self.id = None;
        }
        Ok(())
    }
}
