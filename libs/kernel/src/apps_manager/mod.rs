use crate::KernelResult;
use crate::Milliseconds;
use crate::apps_manager::app_config::{AppConfig, AppStatus, CallMethod, CallPeriodicity};
use crate::apps_manager::shell_cmd::REBOOT_DELAY;
use heapless::Vec;

mod app_config;
mod led_blink;
mod shell_cmd;

const MAX_APPS: usize = 32;
const DEFAULT_APPS: [AppConfig; 2] = [
    AppConfig {
        name: "led_blink",
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
        periodicity: CallPeriodicity::PeriodicUntil(
            Milliseconds(1000),
            Milliseconds((REBOOT_DELAY + 1) as u32 * 1000),
        ),
        app_fn: CallMethod::Call(shell_cmd::reboot_periodic),
        init_fn: None,
        end_fn: Some(shell_cmd::reboot_end),
        app_status: AppStatus::Stopped,
        id: None,
        app_id_storage: Some(shell_cmd::cmd_app_id_storage),
    },
];

const DEFAULT_APPS_START_LIST: [&str; 1] = ["led_blink"];

pub struct AppsManager {
    apps: Vec<AppConfig, MAX_APPS>,
}

impl AppsManager {
    pub fn new() -> AppsManager {
        Self { apps: Vec::new() }
    }

    /// Initialize and register the compile-time `DEFAULT_APPS` list into this [`AppsManager`].
    ///
    /// For each app defined in [`DEFAULT_APPS`], this function:
    /// - Clones the [`AppConfig`] entry (it is `Copy`) into a temporary value,
    /// - Starts it immediately if its name is present in [`DEFAULT_APPS_START_LIST`],
    /// - Pushes it into the internal apps list.
    ///
    /// # Errors
    /// Returns an error if:
    /// - Starting a default app fails (via [`AppConfig::start`]), or
    /// - The internal apps list is full and the app cannot be added.
    pub fn init_default_apps(&mut self) -> KernelResult<()> {
        for app in DEFAULT_APPS.iter() {
            // Check if the app is in the start list
            let mut app_tmp = *app;
            if DEFAULT_APPS_START_LIST.contains(&app.name) {
                app_tmp.start()?;
            }

            // Push it into the vector
            match self.apps.push(app_tmp) {
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

    /// Start a registered app by name.
    ///
    /// This searches the internal apps list for an app whose [`AppConfig::name`]
    /// matches `app_name` and invokes [`AppConfig::start`] on it.
    ///
    /// # Arguments
    /// * `app_name` - The name of the app to start.
    ///
    /// # Returns
    /// On success, returns the started app's ID (as returned by [`AppConfig::start`]).
    ///
    /// # Errors
    /// Returns [`crate::KernelError::AppNotFound`] if no registered app matches `app_name`,
    /// or propagates any error returned by [`AppConfig::start`].
    pub fn start_app(&mut self, app_name: &str) -> KernelResult<u32> {
        self.apps
            .iter_mut()
            .find(|app| app.name == app_name)
            .ok_or(crate::KernelError::AppNotFound)?
            .start()
    }
    
    /// Stop a running registered app by its ID.
    ///
    /// This searches the internal apps list for an app whose [`AppConfig::id`]
    /// matches `app_id` and invokes [`AppConfig::stop`] on it.
    ///
    /// # Arguments
    /// * `app_id` - The ID of the app to stop.
    ///
    /// # Returns
    /// Returns `Ok(())` if the app was found and successfully stopped.
    ///
    /// # Errors
    /// Returns [`crate::KernelError::AppNotFound`] if no registered app matches `app_id`,
    /// or propagates any error returned by [`AppConfig::stop`].
    pub fn stop_app(&mut self, app_id: u32) -> KernelResult<()> {
        self.apps
            .iter_mut()
            .find(|app| app.id == Some(app_id))
            .ok_or(crate::KernelError::AppNotFound)?
            .stop()
    }
}
