//! Control application for managing other apps.
//!
//! This module provides a simple shell-like interface to list, start, and stop
//! registered applications.

use core::sync::atomic::{AtomicU32, Ordering};
use heapless::format;
use spin::Mutex;

use heapless::{String, Vec};

use crate::{
    CallPeriodicity, ConsoleFormatting, K_MAX_APP_PARAM_SIZE, K_MAX_APP_PARAMS, KernelResult,
    data::Kernel, syscall_terminal,
};

/// Last assigned scheduler ID for the control app.
static G_APP_CTRL_ID_STORAGE: AtomicU32 = AtomicU32::new(0);
/// Captured parameters for the control app.
static G_APP_CTRL_PARAM_STORAGE: Mutex<Vec<String<K_MAX_APP_PARAM_SIZE>, K_MAX_APP_PARAMS>> =
    Mutex::new(Vec::new());

/// Checks if an app has one-shot periodicity and displays an error if so.
///
/// # Arguments
/// * `p_app` - App name to check.
///
/// # Returns
/// `true` if the app has `CallPeriodicity::Once` (error message displayed),
/// `false` if the app can be started/stopped.
///
/// # Errors
/// Returns [`crate::KernelError::AppNotFound`] if no registered app matches `p_app`.
fn reject_one_shot_app(p_app: &str) -> KernelResult<bool> {
    if Kernel::apps().get_app_periodicity(p_app)? == CallPeriodicity::Once {
        syscall_terminal(
            ConsoleFormatting::StrNewLineBefore("One-shot apps cannot be controlled"),
            G_APP_CTRL_ID_STORAGE.load(Ordering::Relaxed),
        )?;
        return Ok(true);
    }
    Ok(false)
}

/// Kernel app entry point for the control command.
///
/// Supported actions:
/// - `status`: list registered apps and their status.
/// - `start <app>`: start a registered app by name.
/// - `stop <app>`: stop a running app by name.
pub fn app_ctrl() -> KernelResult<()> {
    let l_storage = G_APP_CTRL_PARAM_STORAGE.lock();

    // If no parameters are provided, print a message and return early.
    if l_storage.is_empty() {
        syscall_terminal(
            ConsoleFormatting::StrNewLineBefore("No action given"),
            G_APP_CTRL_ID_STORAGE.load(Ordering::Relaxed),
        )?;
        return Ok(());
    }

    if let Some(l_action) = l_storage.get(0) {
        match l_action.as_str() {
            "status" => {
                // Check for optional '-a' parameter to show all apps
                let l_show_all = match l_storage.get(1) {
                    Some(l_param) if l_param == "-a" => true,
                    Some(l_param) => {
                        syscall_terminal(
                            ConsoleFormatting::StrNewLineBefore(
                                format!(50; "Invalid parameter: {}", l_param)
                                    .unwrap()
                                    .as_str(),
                            ),
                            G_APP_CTRL_ID_STORAGE.load(Ordering::Relaxed),
                        )?;
                        return Ok(());
                    }
                    None => false,
                };

                // Print status of all apps
                for l_app in Kernel::apps().list_apps() {
                    let l_periodicity = Kernel::apps().get_app_periodicity(l_app)?;

                    if l_show_all || l_periodicity != CallPeriodicity::Once {
                        let l_status = Kernel::apps().get_app_status(l_app)?;

                        syscall_terminal(
                            ConsoleFormatting::StrNewLineBefore(
                                format!(50; "{} -> {}", l_app, l_status.as_str())
                                    .unwrap()
                                    .as_str(),
                            ),
                            G_APP_CTRL_ID_STORAGE.load(Ordering::Relaxed),
                        )?;
                    }
                }
            }
            "start" => {
                // Start an app
                if l_storage.len() > 2 {
                    syscall_terminal(
                        ConsoleFormatting::StrNewLineBefore("Too many parameters"),
                        G_APP_CTRL_ID_STORAGE.load(Ordering::Relaxed),
                    )?;
                    return Ok(());
                }

                if let Some(l_app) = l_storage.get(1) {
                    // Check periodicity - only allow Periodic and PeriodicUntil
                    if reject_one_shot_app(l_app)? {
                        return Ok(());
                    }

                    match Kernel::apps().start_app(l_app) {
                        Ok(_) => {
                            syscall_terminal(
                                ConsoleFormatting::StrNewLineBefore("App started"),
                                G_APP_CTRL_ID_STORAGE.load(Ordering::Relaxed),
                            )?;
                        }
                        Err(l_e) => match l_e {
                            crate::KernelError::AppAlreadyScheduled(_) => {
                                syscall_terminal(
                                    ConsoleFormatting::StrNewLineBefore("App already running"),
                                    G_APP_CTRL_ID_STORAGE.load(Ordering::Relaxed),
                                )?;
                            }
                            _ => {
                                return Err(l_e);
                            }
                        },
                    }
                } else {
                    syscall_terminal(
                        ConsoleFormatting::StrNewLineBefore("No app specified"),
                        G_APP_CTRL_ID_STORAGE.load(Ordering::Relaxed),
                    )?;
                }
            }
            "stop" => {
                // Stop an app
                if l_storage.len() > 2 {
                    syscall_terminal(
                        ConsoleFormatting::StrNewLineBefore("Too many parameters"),
                        G_APP_CTRL_ID_STORAGE.load(Ordering::Relaxed),
                    )?;
                    return Ok(());
                }

                if let Some(l_app) = l_storage.get(1) {
                    // Check periodicity - only allow Periodic and PeriodicUntil
                    if reject_one_shot_app(l_app)? {
                        return Ok(());
                    }

                    if let Some(l_id) = Kernel::apps().get_app_id(l_app)? {
                        Kernel::apps().stop_app(l_id)?;
                        syscall_terminal(
                            ConsoleFormatting::StrNewLineBefore("App stopped"),
                            G_APP_CTRL_ID_STORAGE.load(Ordering::Relaxed),
                        )?;
                    } else {
                        syscall_terminal(
                            ConsoleFormatting::StrNewLineBefore("App not running"),
                            G_APP_CTRL_ID_STORAGE.load(Ordering::Relaxed),
                        )?;
                    }
                } else {
                    syscall_terminal(
                        ConsoleFormatting::StrNewLineBefore("No app specified"),
                        G_APP_CTRL_ID_STORAGE.load(Ordering::Relaxed),
                    )?;
                }
            }
            _ => {
                syscall_terminal(
                    ConsoleFormatting::StrNewLineBefore("Invalid action"),
                    G_APP_CTRL_ID_STORAGE.load(Ordering::Relaxed),
                )?;
            }
        }
    }

    Ok(())
}

/// Capture parameters and app id for the control command.
///
/// # Parameters
/// - `app_id`: Scheduler id assigned to this app.
/// - `param`: Parsed parameters for the command.
pub fn app_ctrl_init(
    p_app_id: u32,
    p_param: Vec<String<K_MAX_APP_PARAM_SIZE>, K_MAX_APP_PARAMS>,
) -> KernelResult<()> {
    G_APP_CTRL_ID_STORAGE.store(p_app_id, core::sync::atomic::Ordering::Relaxed);
    let mut l_storage = G_APP_CTRL_PARAM_STORAGE.lock();
    *l_storage = p_param;
    Ok(())
}
