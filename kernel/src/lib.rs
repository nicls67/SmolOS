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
mod kernel_apps;

pub use crate::data::KernelTimeData;
pub use crate::console_output::ConsoleOutput;
pub use data::cortex_init;
pub use devices::{DeviceType, LockState};
pub use syscall::*;
pub use systick::init_systick;
pub use types::*;
pub use boot::{BootConfig, boot};