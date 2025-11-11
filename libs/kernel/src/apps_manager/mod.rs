use crate::KernelResult;
use crate::Milliseconds;
use crate::apps_manager::app_config::{AppConfig, AppStatus, CallMethod, CallPeriodicity};
use heapless::{String, Vec};

mod app_config;
mod led_blink;
mod shell_cmd;

const MAX_APPS: usize = 32;
const DEFAULT_APPS: [AppConfig; 2] = [
    AppConfig {
        name: "LED Blink",
        periodicity: CallPeriodicity::Periodic(Milliseconds(1000)),
        app_fn: CallMethod::Call(led_blink::led_blink),
        init_fn: Some(led_blink::init_led_blink),
        end_fn: None,
        app_status: AppStatus::Stopped,
        id: None,
        app_id_storage: Some(led_blink::led_blink_id_storage),
    },
    AppConfig {
        name: "reboot",
        periodicity: CallPeriodicity::Once,
        app_fn: CallMethod::Call(shell_cmd::reboot),
        init_fn: None,
        end_fn: None,
        app_status: AppStatus::Stopped,
        id: None,
        app_id_storage: Some(shell_cmd::cmd_app_id_storage),
    },
];

const DEFAULT_APPS_START_LIST: [&str; 1] = ["LED Blink"];

pub struct AppsManager {
    apps: Vec<AppConfig, MAX_APPS>,
}

impl AppsManager {
    pub fn new() -> AppsManager {
        Self { apps: Vec::new() }
    }

    pub fn init_default_apps(&mut self) -> KernelResult<()> {
        for app in DEFAULT_APPS.iter() {
            // Check if the app is in the start list
            let mut app_tmp = *app;
            if DEFAULT_APPS_START_LIST.contains(&app.name) {
                app_tmp.start()?;
            }

            // Push it into the vector
            match self.apps.push(*app) {
                Ok(_) => {}
                Err(_) => return Err(crate::KernelError::CannotAddNewPeriodicApp(app.name)),
            }
        }

        Ok(())
    }

    pub fn add_app(&mut self, mut app: AppConfig) -> KernelResult<()> {
        app.app_status = AppStatus::Stopped;
        app.id = None;

        match self.apps.push(app) {
            Ok(_) => Ok(()),
            Err(_) => Err(crate::KernelError::CannotAddNewPeriodicApp(app.name)),
        }
    }

    pub fn start_app(&mut self, app_name: &str) -> KernelResult<()> {
        self.apps
            .iter_mut()
            .find(|app| app.name == app_name)
            .ok_or(crate::KernelError::AppNotFound)?
            .start()
    }
}
