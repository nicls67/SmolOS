#![no_std]
mod apps_manager;
mod data;
mod errors_mgt;
mod ident;
mod scheduler;
mod syscall;
mod systick;
mod terminal;
mod types;

use crate::apps_manager::AppsManager;
use crate::data::Kernel;
pub use crate::data::KernelTimeData;
use crate::errors_mgt::ErrorsManager;
use crate::ident::{KERNEL_MASTER_ID, KERNEL_NAME, KERNEL_VERSION};
use crate::scheduler::Scheduler;
pub use crate::terminal::TerminalType;
use crate::terminal::{Terminal, TerminalFormatting};
pub use data::cortex_init;
use display::FontSize::Font24;
use display::{Colors, Display};
use hal_interface::Hal;
use heapless::{Vec, format};
pub use syscall::*;
pub use systick::init_systick;
pub use types::*;

pub struct BootConfig {
    pub sched_period: Milliseconds,
    pub kernel_time_data: KernelTimeData,
    pub hal: Hal,
    pub system_terminals: Vec<TerminalType, 8>,
    pub err_led_name: Option<&'static str>,
    pub display_name: Option<&'static str>,
}

/// Boot the kernel with the provided configuration.
///
/// This function initializes the kernel, sets up the terminal, configures
/// error management, and initializes system timers. It performs the following steps:
///
/// 1. **Kernel Initialization**:
///    - Creates a new scheduler with the specified scheduling period.
///    - Initializes kernel data, including hardware abstraction layer (HAL),
///      kernel time data, system terminal, scheduler, and error manager.
///
/// 2. **Terminal Initialization**:
///    - Starts the kernel terminal and transitions it to the kernel state.
///    - Clears the terminal screen and writes boot messages, including the
///      kernel name, version, and core frequency information.
///
/// 3. **Error Manager Setup**:
///    - Initializes the error manager with the specified error LED name to
///      handle hardware or system-related errors.
///
/// 4. **Timer Initialization**:
///    - Configures SysTick with the period provided in the kernel time data
///      for time
pub fn boot(config: BootConfig) {
    //////////////////////////
    // Kernel initialization
    //////////////////////////
    let sched = Scheduler::new(config.sched_period);
    Kernel::init_kernel_data(
        config.hal,
        Display::new(),
        config.kernel_time_data.clone(),
        Terminal::new(config.system_terminals),
        sched,
        ErrorsManager::new(),
        AppsManager::new(),
    );
    Kernel::hal().configure_locker(KERNEL_MASTER_ID).unwrap();

    ////////////////////////////////////
    // Errors Manager initialization
    ////////////////////////////////////
    Kernel::errors().init(config.err_led_name).unwrap();

    //////////////////////////
    // Display initialization
    //////////////////////////
    Kernel::display()
        .init(config.display_name.unwrap(), Kernel::hal(), Colors::Black)
        .unwrap();
    Kernel::display()
        .set_font(Font24, KERNEL_MASTER_ID)
        .unwrap();

    ////////////////////////////
    // Terminal start
    ////////////////////////////
    let terminal = Kernel::terminal();
    terminal.set_display_mode().unwrap();
    terminal.write(&TerminalFormatting::Clear).unwrap();
    terminal
        .write(&TerminalFormatting::StrNewLineAfter("Booting..."))
        .unwrap();
    terminal
        .write(&TerminalFormatting::StrNewLineAfter(
            format!(30; "{} version {}", KERNEL_NAME, KERNEL_VERSION)
                .unwrap()
                .as_str(),
        ))
        .unwrap();
    terminal
        .write(&TerminalFormatting::StrNewLineAfter(
            format!(30; "Core frequency is {} MHz", Kernel::time_data().core_frequency.to_u32() / 1_000_000)
                .unwrap()
                .as_str(),
        ))
        .unwrap();

    ////////////////////////////////////
    // Systick initialization
    ////////////////////////////////////
    init_systick(Some(config.kernel_time_data.systick_period));

    //Boot completed
    terminal.set_color(Colors::Green);
    terminal
        .write(&TerminalFormatting::StrNewLineBoth("Kernel ready !"))
        .unwrap();

    // Initialize default apps
    Kernel::apps().init_default_apps().unwrap();

    // Start scheduler
    Kernel::scheduler()
        .start(Kernel::time_data().clone().systick_period)
        .unwrap();

    // Set terminal in prompt mode
    terminal.set_prompt_mode().unwrap();
}
