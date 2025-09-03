#![no_std]
mod data;
mod except;
mod ident;
mod types;

use crate::data::Kernel;
use crate::except::set_ticks_target;
use crate::ident::{KERNEL_NAME, KERNEL_VERSION};
use cortex_m::peripheral::scb::SystemHandler;
use cortex_m::peripheral::syst::SystClkSource;
use hal_interface::Hal;
use heapless::format;
pub use types::*;

pub struct BootConfig {
    pub sched_period: Milliseconds,
    pub systick_period: Milliseconds,
    pub core_freq: u32,
    pub hal: Hal,
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
    Kernel::init_kernel_data(config.hal, config.core_freq);

    let serial_id = Kernel::hal().get_interface_id("SERIAL_MAIN").unwrap();

    // Clear console
    Kernel::hal()
        .interface_write(
            serial_id,
            hal_interface::InterfaceWriteActions::UartWrite(
                hal_interface::UartWriteActions::SendString("\x1B[2J\x1B[H"),
            ),
        )
        .unwrap();

    Kernel::hal()
        .interface_write(
            serial_id,
            hal_interface::InterfaceWriteActions::UartWrite(
                hal_interface::UartWriteActions::SendString("Booting...\r\n"),
            ),
        )
        .unwrap();

    Kernel::hal()
        .interface_write(
            serial_id,
            hal_interface::InterfaceWriteActions::UartWrite(
                hal_interface::UartWriteActions::SendString(
                    format!(30; "{} version {}\r\n", KERNEL_NAME, KERNEL_VERSION)
                        .unwrap()
                        .as_str(),
                ),
            ),
        )
        .unwrap();

    ////////////////////////////////////
    // Timers initialization
    ////////////////////////////////////

    // Initialize Systick at 1ms
    let cortex_p = Kernel::cortex_peripherals();
    cortex_p.SYST.set_clock_source(SystClkSource::Core);
    cortex_p.SYST.clear_current();
    cortex_p
        .SYST
        .set_reload(config.core_freq * config.systick_period.to_u32() / 1000);
    cortex_p.SYST.enable_interrupt();

    // Initialize scheduler periodic IT
    unsafe {
        cortex_p.SCB.set_priority(SystemHandler::PendSV, 0xFF);
        set_ticks_target(config.sched_period.to_u32() / config.systick_period.to_u32())
    }

    // Start Systick
    cortex_p.SYST.enable_counter();

    Kernel::hal()
        .interface_write(
            serial_id,
            hal_interface::InterfaceWriteActions::UartWrite(
                hal_interface::UartWriteActions::SendString("Kernel ready !\r\n"),
            ),
        )
        .unwrap();
}
