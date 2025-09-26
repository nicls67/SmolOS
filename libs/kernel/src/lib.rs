#![no_std]
mod data;
mod errors_mgt;
mod except;
mod ident;
mod kernel_apps;
mod scheduler;
mod syscall;
mod terminal;
mod types;

use crate::data::Kernel;
pub use crate::data::KernelTimeData;
use crate::errors_mgt::ErrorsManager;
use crate::ident::{KERNEL_NAME, KERNEL_VERSION};
use crate::scheduler::Scheduler;
pub use crate::terminal::{Terminal, TerminalFormatting, TerminalType};
use cortex_m::peripheral::syst::SystClkSource;
use hal_interface::Hal;
use heapless::format;
pub use syscall::*;
pub use types::*;

pub struct BootConfig {
    pub sched_period: Milliseconds,
    pub kernel_time_data: KernelTimeData,
    pub hal: Hal,
    pub system_terminal_name: &'static str,
    pub system_terminal_type: TerminalType,
    pub err_led_name: Option<&'static str>,
}

/**
 * Boots the system using the provided boot configuration.
 *
 * This function performs system initialization tasks such as setting up the kernel,
 * configuring timers, and initializing the system scheduler. It is designed to bring
 * the system into a state ready for operation.
 *
 * # Parameters
 * - `config`: A `BootConfig` struct containing the hardware abstraction layer (HAL),
 *   the core frequency, and timing parameters for system initialization.
 *
 * # Initialization Steps
 * 1. **Kernel Initialization**:
 *    - Prepares the kernel by calling `Kernel::init_kernel_data` and passing the HAL
 *      and core frequency settings to initialize kernel-specific data.
 *
 * 2. **Timers Initialization**:
 *    - Configures the SysTick timer to generate periodic interrupts at a rate of 1 millisecond.
 *      - Sets the clock source of the SysTick timer to the core clock.
 *      - Configures the reload value based on the core clock frequency and desired tick period.
 *      - Enables SysTick timer interrupts.
 *    - Sets up the system scheduler's periodic interrupt using the configured scheduler period.
 *      - Adjusts the priority for the PendSV system handler.
 *      - Calculates and sets the scheduler tick target based on the system period configuration.
 *
 * 3. **Start SysTick**:
 *    - Enables the SysTick counter to begin generating interrupts according to the timer configuration.
 *
 * # Safety
 * - Unsafe code is used to configure the priority of the PendSV system handler and set the tick
 *   target directly. The caller must ensure that the configuration complies with the system's
 *   requirements and does not introduce undefined behavior.
 *
 */
pub fn boot(config: BootConfig) {
    /////////////////////////
    // Kernel initialization
    /////////////////////////

    let sched = Scheduler::new(config.sched_period);
    Kernel::init_kernel_data(
        config.hal,
        config.kernel_time_data.clone(),
        Terminal::new(config.system_terminal_name, config.system_terminal_type),
        sched,
        ErrorsManager::new(),
    );

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
    // Timers initialization
    ////////////////////////////////////

    // Initialize Systick at 1ms
    let cortex_p = Kernel::cortex_peripherals();
    cortex_p.SYST.set_clock_source(SystClkSource::Core);
    cortex_p.SYST.clear_current();
    cortex_p.SYST.set_reload(
        config.kernel_time_data.core_frequency.to_u32()
            * config.kernel_time_data.systick_period.to_u32()
            / 1000,
    );
    cortex_p.SYST.enable_interrupt();

    //Boot completed
    terminal
        .write(&TerminalFormatting::StrNewLineBoth("Kernel ready !"))
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
