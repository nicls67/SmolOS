use crate::KernelResult;
use heapless::Vec;

mod app_config;

pub use self::app_config::{AppConfig, AppStatus, CallMethod, CallPeriodicity};

const MAX_APPS: usize = 32;

pub struct AppsManager {
    apps: Vec<AppConfig, MAX_APPS>,
}

impl AppsManager {
    pub fn new() -> AppsManager {
        Self { apps: Vec::new() }
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
