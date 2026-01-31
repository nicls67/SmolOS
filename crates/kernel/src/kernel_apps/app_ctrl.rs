//! Control application for managing other apps.
//!
//! This module provides a simple shell-like interface to list, start, and stop
//! registered applications.

use core::sync::atomic::{AtomicU32, Ordering};
use heapless::format;
use spin::Mutex;

use heapless::{String, Vec};

use crate::{
    ConsoleFormatting, K_MAX_APP_PARAM_SIZE, K_MAX_APP_PARAMS, KernelResult, data::Kernel,
    syscall_terminal,
};

/// Last assigned scheduler ID for the control app.
static G_APP_CTRL_ID_STORAGE: AtomicU32 = AtomicU32::new(0);
/// Captured parameters for the control app.
static G_APP_CTRL_PARAM_STORAGE: Mutex<Vec<String<K_MAX_APP_PARAM_SIZE>, K_MAX_APP_PARAMS>> =
    Mutex::new(Vec::new());

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
                // Print status of all apps
                for l_app in Kernel::apps().list_apps() {
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
            "start" => {
                // Start an app
                if let Some(l_app) = l_storage.get(1) {
                    match Kernel::apps().start_app(l_app) {
                        Ok(_) => {
                            syscall_terminal(
                                ConsoleFormatting::StrNewLineBefore("App started"),
                                G_APP_CTRL_ID_STORAGE.load(Ordering::Relaxed),
                            )?;
                        }
                        Err(e) => match e {
                            crate::KernelError::AppAlreadyScheduled(_) => {
                                syscall_terminal(
                                    ConsoleFormatting::StrNewLineBefore("App already running"),
                                    G_APP_CTRL_ID_STORAGE.load(Ordering::Relaxed),
                                )?;
                            }
                            _ => {
                                return Err(e);
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
                if let Some(l_app) = l_storage.get(1) {
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
