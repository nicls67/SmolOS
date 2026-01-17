use crate::KernelResult;
use heapless::Vec;

mod app_config;

pub use self::app_config::{AppConfig, AppStatus, CallMethod, CallPeriodicity};

const K_MAX_APPS: usize = 32;

pub struct AppsManager {
    apps: Vec<AppConfig, K_MAX_APPS>,
}

impl AppsManager {
    /// Creates a new `AppsManager` instance with an empty application registry.
    ///
    /// # Returns
    ///
    /// A new `AppsManager` with no registered applications.
    pub fn new() -> AppsManager {
        Self { apps: Vec::new() }
    }

    /// Registers a new application with the manager.
    ///
    /// The application is added to the internal registry in a stopped state, ready to be
    /// started later via [`AppsManager::start_app`]. Any existing `app_status` and `id`
    /// values in the provided configuration are reset.
    ///
    /// # Parameters
    ///
    /// * `app` - The application configuration to register. The `app_status` will be
    ///   set to [`AppStatus::Stopped`] and `id` will be cleared to `None`.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the application was successfully registered.
    ///
    /// * `Err(KernelError::CannotAddNewPeriodicApp)` - If the application registry is
    ///   full (maximum of 32 applications).
    pub fn add_app(&mut self, mut p_app: AppConfig) -> KernelResult<()> {
        p_app.app_status = AppStatus::Stopped;
        p_app.id = None;

        match self.apps.push(p_app) {
            Ok(_) => Ok(()),
            Err(_) => Err(crate::KernelError::CannotAddNewPeriodicApp(p_app.name)),
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
    pub fn start_app(&mut self, p_app_name: &str) -> KernelResult<u32> {
        self.apps
            .iter_mut()
            .find(|l_app| l_app.name == p_app_name)
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
    pub fn stop_app(&mut self, p_app_id: u32) -> KernelResult<()> {
        self.apps
            .iter_mut()
            .find(|l_app| l_app.id == Some(p_app_id))
            .ok_or(crate::KernelError::AppNotFound)?
            .stop()
    }
}
