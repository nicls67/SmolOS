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

/// Store the control app scheduler ID for later inspection.
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
                            format!(50;"{} -> {}", l_app, l_status.as_str())
                                .unwrap()
                                .as_str(),
                        ),
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
