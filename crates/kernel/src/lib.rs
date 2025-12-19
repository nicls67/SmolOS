#![no_std]
mod apps;
mod boot;
mod data;
mod devices;
mod errors_mgt;
mod ident;
mod scheduler;
mod syscall;
mod systick;
mod terminal;
mod types;
mod console_output;

use crate::apps::AppsManager;
use crate::data::Kernel;
pub use crate::data::KernelTimeData;
pub use crate::console_output::ConsoleOutput;
pub use data::cortex_init;
pub use devices::{DeviceType, LockState};
pub use syscall::*;
pub use systick::init_systick;
pub use types::*;
pub use boot::{BootConfig, boot};
pub use apps::{AppConfig, CallPeriodicity,CallMethod,AppStatus};
pub use types::Milliseconds;
pub use types::KernelResult;
pub use console_output::ConsoleFormatting;

/// Returns a mutable reference to the global [`AppsManager`].
pub fn apps() -> &'static mut AppsManager {
    Kernel::apps()
}
