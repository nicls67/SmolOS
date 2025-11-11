use crate::apps_manager::app_config::AppStatus::{Running, Stopped};
use crate::data::Kernel;
use crate::scheduler::{App, AppCall, AppParam};
use crate::{KernelResult, Milliseconds, Syscall, syscall};

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
    pub fn start(&mut self) -> KernelResult<()> {
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

            let mut app_id: u32 = 0;

            syscall(
                Syscall::AddPeriodicTask(
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
                    &mut app_id,
                ),
                0,
            )?;
            self.id = Some(app_id);
            self.app_status = Running;

            if let Some(app_id_storage) = self.app_id_storage {
                app_id_storage(app_id);
            }
        }
        Ok(())
    }

    pub fn stop(&mut self) -> KernelResult<()> {
        if self.app_status == Running {
            syscall(
                Syscall::RemovePeriodicTask(
                    self.name,
                    match self.app_fn {
                        CallMethod::Call(_) => None,
                        CallMethod::CallWithParam(_, param) => Some(param),
                    },
                ),
                self.id.unwrap(),
            )?;
            self.app_status = Stopped;
            self.id = None;
        }
        Ok(())
    }
}
