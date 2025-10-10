#![no_std]
mod data;
mod errors_mgt;
mod ident;
mod kernel_apps;
mod scheduler;
mod syscall;
mod systick;
mod terminal;
mod types;

use crate::data::Kernel;
pub use crate::data::KernelTimeData;
use crate::errors_mgt::ErrorsManager;
use crate::ident::{KERNEL_NAME, KERNEL_VERSION};
use crate::scheduler::Scheduler;
pub use crate::terminal::{Terminal, TerminalFormatting, TerminalType};
pub use data::cortex_init;
use display::FontSize::Font24;
use display::{Colors, Display, FontSize};
use hal_interface::Hal;
use heapless::format;
pub use syscall::*;
pub use systick::init_systick;
pub use types::*;

pub struct BootConfig {
    pub sched_period: Milliseconds,
    pub kernel_time_data: KernelTimeData,
    pub hal: Hal,
    pub system_terminal_name: &'static str,
    pub system_terminal_type: TerminalType,
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
        Terminal::new(config.system_terminal_name, config.system_terminal_type),
        sched,
        ErrorsManager::new(),
    );

    //////////////////////////
    // Display initialization
    //////////////////////////
    Kernel::display()
        .init(config.display_name.unwrap(), Kernel::hal(), Colors::Black)
        .unwrap();
    Kernel::display().set_font(Font24);
    Kernel::display()
        .draw_string_at_cursor("Booting...\n", Colors::White)
        .unwrap();
    Kernel::display()
        .draw_string_at_cursor(
            format!(30; "{} version {}\n", KERNEL_NAME, KERNEL_VERSION)
                .unwrap()
                .as_str(),
            Colors::White,
        )
        .unwrap();
    Kernel::display().draw_string_at_cursor(format!(30; "Core frequency is {} MHz\n", Kernel::time_data().core_frequency.to_u32() / 1_000_000)
        .unwrap()
        .as_str(), Colors::White).unwrap();

    ////////////////////////////
    // Terminal start
    ////////////////////////////
    let terminal = Kernel::terminal();
    terminal.set_kernel_state().unwrap();
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
    // Errors Manager initialization
    ////////////////////////////////////
    Kernel::errors().init(config.err_led_name).unwrap();

    ////////////////////////////////////
    // Systick initialization
    ////////////////////////////////////
    init_systick(Some(config.kernel_time_data.systick_period));

    //Boot completed
    terminal
        .write(&TerminalFormatting::StrNewLineBoth("Kernel ready !"))
        .unwrap();
    Kernel::display()
        .draw_string_at_cursor(
            format!(30; "\nKernel ready !\n").unwrap().as_str(),
            Colors::Green,
        )
        .unwrap();
}

/// Starts the system scheduler.
///
/// This function initializes and starts the kernel's scheduler. It retrieves the scheduler
/// instance from the `Kernel`, and then attempts to start it. The `unwrap()` is used on the result
/// of the `start()` method, meaning that this function will panic if the scheduler fails to start.
///
/// # Panics
///
/// This function will panic if the `start()` method of the scheduler returns an error.
///
/// Ensure that the system is properly set up and ready for scheduling before calling this function.
pub fn start_scheduler() {
    Kernel::scheduler()
        .start(Kernel::time_data().clone().systick_period)
        .unwrap();
}

/// Starts the kernel applications for the system.
///
/// This function initializes all kernel-level applications by invoking the
/// `initialize_kernel_apps` method provided by the `kernel_apps` module.
/// It ensures that the kernel apps are correctly set up and ready to use
/// during the system's runtime.
///
/// # Panics
///
/// This function will panic if the initialization of kernel apps fails,
/// as it propagates the error using `unwrap`. Ensure that proper setup and
/// error handling are performed in `initialize_kernel_apps` to avoid runtime
/// panics.
///
/// Make sure to call this function at the appropriate point during system
/// initialization to correctly set up kernel applications.
pub fn start_kernel_apps() {
    kernel_apps::initialize_kernel_apps().unwrap()
}
