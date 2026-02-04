use crate::KernelResult;
use heapless::Vec;

mod app_config;

pub use self::app_config::{
    AppConfig, AppStatus, CallPeriodicity, K_MAX_APP_PARAM_SIZE, K_MAX_APP_PARAMS,
};

const K_MAX_APPS: usize = 32;

/// Manages the registration and lifecycle of user applications.
pub struct AppsManager {
    /// Internal list of registered application configurations.
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
    /// matches the first token of `p_app` and invokes [`AppConfig::start`] on it.
    ///
    /// # Arguments
    /// * `p_app` - The full app invocation string (name plus optional parameters).
    ///
    /// # Returns
    /// On success, returns the started app's ID (as returned by [`AppConfig::start`]).
    ///
    /// # Errors
    /// Returns [`crate::KernelError::AppNotFound`] if no registered app matches the parsed name,
    /// or propagates any error returned by [`AppConfig::start`].
    pub(crate) fn start_app(&mut self, p_app: &str) -> KernelResult<u32> {
        // App name is the first argument
        let l_app_name = p_app.split_ascii_whitespace().next().unwrap_or_default();

        self.apps
            .iter_mut()
            .find(|l_app| l_app.name == l_app_name)
            .ok_or(crate::KernelError::AppNotFound)?
            .start(p_app)
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
    pub(crate) fn stop_app(&mut self, p_app_id: u32) -> KernelResult<()> {
        self.apps
            .iter_mut()
            .find(|l_app| l_app.id == Some(p_app_id))
            .ok_or(crate::KernelError::AppNotFound)?
            .stop()
    }

    /// Returns the list of registered app names.
    ///
    /// # Returns
    /// A vector of app name slices in registration order.
    pub(crate) fn list_apps(&self) -> Vec<&str, K_MAX_APPS> {
        self.apps.iter().map(|l_app| l_app.name).collect()
    }

    /// Returns the current status for a given app name.
    ///
    /// # Arguments
    /// * `p_app` - App name to query.
    ///
    /// # Returns
    /// The current [`AppStatus`] for the matching app.
    ///
    /// # Errors
    /// Returns [`crate::KernelError::AppNotFound`] if no registered app matches `p_app`.
    pub(crate) fn get_app_status(&self, p_app: &str) -> KernelResult<AppStatus> {
        Ok(self
            .apps
            .iter()
            .find(|l_app| l_app.name == p_app)
            .ok_or(crate::KernelError::AppNotFound)?
            .app_status)
    }

    /// Returns the current scheduler id for a given app name.
    ///
    /// # Arguments
    /// * `p_app` - App name to query.
    ///
    /// # Returns
    /// `Some(id)` if the app is running, `None` if it is stopped.
    ///
    /// # Errors
    /// Returns [`crate::KernelError::AppNotFound`] if no registered app matches `p_app`.
    pub(crate) fn get_app_id(&self, p_app: &str) -> KernelResult<Option<u32>> {
        Ok(self
            .apps
            .iter()
            .find(|l_app| l_app.name == p_app)
            .ok_or(crate::KernelError::AppNotFound)?
            .id)
    }

    /// Returns the call periodicity for a given app name.
    ///
    /// # Arguments
    /// * `p_app` - App name to query.
    ///
    /// # Returns
    /// The [`CallPeriodicity`] configured for the matching app.
    ///
    /// # Errors
    /// Returns [`crate::KernelError::AppNotFound`] if no registered app matches `p_app`.
    pub(crate) fn get_app_periodicity(&self, p_app: &str) -> KernelResult<CallPeriodicity> {
        Ok(self
            .apps
            .iter()
            .find(|l_app| l_app.name == p_app)
            .ok_or(crate::KernelError::AppNotFound)?
            .periodicity)
    }
}
