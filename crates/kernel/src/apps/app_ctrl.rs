use core::sync::atomic::AtomicU32;
use spin::Once;

use heapless::{String, Vec};

use crate::{
    AppConfig, AppStatus, CallPeriodicity, K_MAX_APP_PARAM_SIZE, K_MAX_APP_PARAMS, KernelResult,
};

/// App configuration for the kernel control application.
pub const K_APP_CTRL_CONFIG: AppConfig = AppConfig {
    name: "app_ctrl",
    periodicity: CallPeriodicity::Once,
    app_fn: app_ctrl,
    init_fn: None,
    end_fn: None,
    app_status: AppStatus::Stopped,
    id: None,
    app_id_storage: Some(id_storage),
    param_storage: Some(param_storage),
};

/// Last assigned scheduler ID for the control app.
static G_APP_CTRL_ID_STORAGE: AtomicU32 = AtomicU32::new(0);
/// Captured parameters for the control app (set once).
static G_APP_CTRL_PARAM_STORAGE: Once<Vec<String<K_MAX_APP_PARAM_SIZE>, K_MAX_APP_PARAMS>> =
    Once::new();

/// Store the control app scheduler ID for later inspection.
pub fn id_storage(p_id: u32) {
    G_APP_CTRL_ID_STORAGE.store(p_id, core::sync::atomic::Ordering::Relaxed);
}

/// Store app parameters for later inspection by the control app.
///
/// Parameters are already owned heapless strings and can be stored directly.
pub fn param_storage(p_param: Vec<String<K_MAX_APP_PARAM_SIZE>, K_MAX_APP_PARAMS>) {
    G_APP_CTRL_PARAM_STORAGE.call_once(|| p_param);
}

/// No-op control app entry point.
pub fn app_ctrl() -> KernelResult<()> {
    Ok(())
}
