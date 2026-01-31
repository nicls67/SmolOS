#![no_std]
mod apps;
mod boot;
mod console_output;
mod data;
mod devices;
mod errors_mgt;
mod ident;
mod kernel_apps;
mod scheduler;
mod syscall;
mod systick;
mod terminal;
mod types;

use crate::apps::AppsManager;
pub use crate::console_output::ConsoleOutput;
use crate::data::Kernel;
pub use crate::data::KernelTimeData;
pub use apps::{AppConfig, AppStatus, CallPeriodicity, K_MAX_APP_PARAM_SIZE, K_MAX_APP_PARAMS};
pub use boot::{BootConfig, boot};
pub use console_output::ConsoleFormatting;
pub use data::cortex_init;
pub use devices::{DeviceType, LockState};
pub use syscall::*;
pub use systick::init_systick;
pub use types::KernelResult;
pub use types::Milliseconds;
pub use types::*;

/// Returns a mutable reference to the global [`AppsManager`].
pub fn apps() -> &'static mut AppsManager {
    Kernel::apps()
}
