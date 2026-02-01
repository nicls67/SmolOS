//! Error Generation application.

use core::sync::atomic::{AtomicU32, Ordering};

use spin::Mutex;

use heapless::{String, Vec};

use crate::{
    ConsoleFormatting, K_MAX_APP_PARAM_SIZE, K_MAX_APP_PARAMS, KernelError, KernelResult,
    syscall_terminal,
};

/// Last assigned scheduler ID for the err_gen app.
static G_ERR_GEN_ID_STORAGE: AtomicU32 = AtomicU32::new(0);
/// Captured parameters for the err_gen app.
static G_ERR_GEN_PARAM_STORAGE: Mutex<Vec<String<K_MAX_APP_PARAM_SIZE>, K_MAX_APP_PARAMS>> =
    Mutex::new(Vec::new());

/// Kernel app entry point for the err_gen command.
pub fn err_gen() -> KernelResult<()> {
    let l_storage = G_ERR_GEN_PARAM_STORAGE.lock();

    // If no parameters are provided, print a message and return early.
    if l_storage.is_empty() {
        syscall_terminal(
            ConsoleFormatting::StrNewLineBefore("No action given for err_gen"),
            G_ERR_GEN_ID_STORAGE.load(Ordering::Relaxed),
        )?;
        return Ok(());
    }

    if let Some(l_action) = l_storage.get(0) {
        match l_action.as_str() {
            "error" => {
                return Err(KernelError::TestError);
            }
            "critical" => {
                return Err(KernelError::TestCriticalError);
            }
            "fatal" => {
                return Err(KernelError::TestFatalError);
            }
            _ => {
                syscall_terminal(
                    ConsoleFormatting::StrNewLineBefore("Invalid action"),
                    G_ERR_GEN_ID_STORAGE.load(Ordering::Relaxed),
                )?;
            }
        }
    }

    Ok(())
}

/// Capture parameters and app id for the err_gen command.
pub fn err_gen_init(
    p_app_id: u32,
    p_param: Vec<String<K_MAX_APP_PARAM_SIZE>, K_MAX_APP_PARAMS>,
) -> KernelResult<()> {
    G_ERR_GEN_ID_STORAGE.store(p_app_id, core::sync::atomic::Ordering::Relaxed);
    let mut l_storage = G_ERR_GEN_PARAM_STORAGE.lock();
    *l_storage = p_param;
    Ok(())
}
