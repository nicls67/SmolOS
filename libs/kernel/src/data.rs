use crate::scheduler::Scheduler;
use crate::terminal::Terminal;
use crate::{Mhz, Milliseconds};
use cortex_m::Peripherals;
use hal_interface::Hal;

pub static mut KERNEL_DATA: Kernel = Kernel {
    cortex_peripherals: None,
    hal: None,
    kernel_time_data: None,
    terminal: None,
    scheduler: None,
};

/// A data structure representing timing-related configuration for the system kernel.
///
/// This structure contains information regarding the core processor's frequency
/// and the system tick (systick) period, which are essential for coordinating time-sensitive
/// operations within the kernel.
///
/// # Fields
///
/// * `core_frequency` (`Mhz`):
///   The operating frequency of the core processor in megahertz. This value defines the
///   speed at which the processor operates and is used for timing calculations.
///
/// * `systick_period` (`Milliseconds`):
///   The period of the system tick in milliseconds. This value represents the interval
///   between systick interrupts, which are used for task scheduling, kernel timing,
///   and system timekeeping.
///
/// Both fields must be configured appropriately to ensure proper kernel operation,
/// particularly for accurate timing and synchronization.

#[derive(Clone)]
pub struct KernelTimeData {
    pub core_frequency: Mhz,
    pub systick_period: Milliseconds,
}

/// Represents the central structure of the operating system kernel, containing various components
/// required for its operation.
///
/// The `Kernel` struct encapsulates the core components of the system, utilizing optional fields
/// for flexibility during initialization and configuration. Each field governs a critical aspect
/// of kernel functionality, ranging from low-level peripherals to high-level scheduling.
///
/// Fields:
/// - `cortex_peripherals`: Optionally holds the Cortex-M microcontroller's peripherals,
///   providing access to essential hardware features such as system timers, interrupts, and
///   other on-chip components.
/// - `hal`: Optionally stores the Hardware Abstraction Layer (HAL) implementation, offering
///   a consistent interface to underlying hardware interactions and abstractions.
/// - `kernel_time_data`: Optionally contains timing-related data and utilities crucial for
///   measuring and managing system time, delays, or scheduling operations.
/// - `terminal`: Optionally provides a text-based terminal interface for user interaction,
///   logging, or debugging purposes.
/// - `scheduler`: Optionally manages task scheduling and context switching, ensuring efficient
///   execution and multitasking within the kernel.
///
/// The `Kernel` struct is designed to allow lazy initialization of its components, enabling
/// modular development and customization of the kernel according to specific use cases or hardware
/// configurations.
pub struct Kernel {
    cortex_peripherals: Option<Peripherals>,
    hal: Option<Hal>,
    kernel_time_data: Option<KernelTimeData>,
    terminal: Option<Terminal>,
    scheduler: Option<Scheduler>,
}

impl Kernel {
    /// Initializes the kernel data structure with the provided components.
    ///
    /// This function sets up essential components of the kernel by assigning
    /// values to the global `KERNEL_DATA` structure. It is used to configure
    /// and prepare the kernel for operation. The function leverages unsafe
    /// blocks to directly modify static data, which requires caution to ensure
    /// proper synchronization and safety.
    ///
    /// # Parameters
    ///
    /// - `hal`: The hardware abstraction layer (HAL) used to interface with the
    ///   hardware of the system.
    /// - `kernel_time_data`: Data related to the kernel's timekeeping mechanisms.
    /// - `terminal`: The terminal interface used for input/output operations in
    ///   the kernel environment.
    /// - `scheduler`: The task
    pub fn init_kernel_data(
        hal: Hal,
        kernel_time_data: KernelTimeData,
        terminal: Terminal,
        scheduler: Scheduler,
    ) {
        unsafe {
            KERNEL_DATA.cortex_peripherals = Some(Peripherals::take().unwrap());
            KERNEL_DATA.hal = Some(hal);
            KERNEL_DATA.kernel_time_data = Some(kernel_time_data);
            KERNEL_DATA.terminal = Some(terminal);
            KERNEL_DATA.scheduler = Some(scheduler);
        }
    }

    /// Provides a static reference to the `Hal` instance.
    ///
    /// # Returns
    /// A static reference (`&'static`) to the `Hal` object if it's initialized.
    ///
    /// # Panics
    /// This function will panic with the message `"Hal not initialized"` if the `Hal`
    /// instance has not been set in `KERNEL_DATA`.
    ///
    /// # Safety
    /// This function uses unsafe code to access the static mutable `KERNEL_DATA.hal` value.
    /// The unsafe block assumes that access to `KERNEL_DATA.hal` has been properly
    /// synchronized and initialized before calling this function.
    ///
    /// # Allowance
    /// The `#[allow(static_mut_refs)]` attribute is used to suppress the warning for
    /// accessing mutable statics, as this pattern relies on proper internal synchronization
    /// to ensure safety when manipulating `KERNEL_DATA.hal`.
    ///
    /// # Usage
    /// Ensure that the `Hal` instance is initialized in `KERNEL_DATA.hal` before invoking this function:
    ///
    /// If `KERNEL_DATA.hal` is uninitialized, calling this function will result in a panic.
    ///
    #[allow(static_mut_refs)]
    pub fn hal() -> &'static mut Hal {
        unsafe {
            if KERNEL_DATA.hal.is_some() {
                KERNEL_DATA.hal.as_mut().unwrap()
            } else {
                panic!("Hal not initialized");
            }
        }
    }

    /// Retrieves a mutable reference to the Cortex-M peripherals if they have been initialized.
    ///
    /// # Returns
    /// A mutable reference to the `Peripherals` structure that represents Cortex-M peripherals.
    ///
    /// # Panics
    /// This function will panic if the Cortex-M peripherals have not been initialized before calling this function.
    ///
    /// # Safety
    /// This function involves unsafe operations as it accesses mutable static data. The caller must ensure
    /// that this function is used in a thread-safe manner to avoid data races.
    ///
    /// # Features
    /// - The function allows static mutable references by leveraging `#[allow(static_mut_refs)]`, which is
    ///   inherently unsafe. Use with caution in concurrent environments.
    /// - Accessing the peripherals is protected by an `Option`, ensuring that the code only proceeds
    ///   if the peripherals are initialized.
    ///
    #[allow(static_mut_refs)]
    pub fn cortex_peripherals() -> &'static mut Peripherals {
        unsafe {
            if KERNEL_DATA.cortex_peripherals.is_some() {
                KERNEL_DATA.cortex_peripherals.as_mut().unwrap()
            } else {
                panic!("Cortex-M peripherals not initialized");
            }
        }
    }

    /// Provides mutable access to the global `Terminal` instance safely.
    ///
    /// # Returns
    /// A mutable reference to the global `Terminal` instance, if it has been initialized successfully.
    ///
    /// # Panics
    /// This function will panic if the `terminal` field in `KERNEL_DATA` is not initialized.
    /// Ensure that the `terminal` field is properly set up before calling this function.
    ///
    /// # Safety
    /// This function internally uses unsafe blocks to access a static mutable reference,
    /// which can potentially lead to undefined behavior if improperly used.
    /// The caller must ensure synchronization and prevent concurrent access to this data
    /// to avoid data races in a multithreaded context.
    ///
    /// # Note
    /// The improper usage of static mutable references is usually considered unsafe in Rust.
    /// However, this function makes use of `#[allow(static_mut_refs)]` to suppress warnings
    /// related to static mutable references
    #[allow(static_mut_refs)]
    pub fn terminal() -> &'static mut Terminal {
        unsafe {
            if KERNEL_DATA.terminal.is_some() {
                KERNEL_DATA.terminal.as_mut().unwrap()
            } else {
                panic!("Terminal not initialized");
            }
        }
    }

    /// Returns a mutable reference to the global `Scheduler` instance if it is initialized.
    ///
    /// # Safety
    /// This function uses an unsafe block to access and return a mutable reference
    /// to a static variable. This introduces the risk of undefined behavior if improper
    /// access occurs, for example, if the `scheduler` is accessed concurrently without
    /// proper synchronization. Ensure that this function is only called in a single-threaded
    /// context or that proper synchronization mechanisms are in place.
    ///
    /// # Panics
    /// This function will panic if the global `Scheduler` is not initialized (i.e., if
    /// `KERNEL_DATA.scheduler` is `None`).
    ///
    /// # Returns
    /// * A mutable reference to the global `Scheduler` instance.
    ///
    #[allow(static_mut_refs)]
    pub fn scheduler() -> &'static mut Scheduler {
        unsafe {
            if KERNEL_DATA.scheduler.is_some() {
                KERNEL_DATA.scheduler.as_mut().unwrap()
            } else {
                panic!("Scheduler not initialized");
            }
        }
    }

    /// Returns a static reference to the `KernelTimeData` if it has been initialized.
    ///
    /// # Safety
    /// This function performs an unsafe block to obtain a mutable reference to a static
    /// instance, which is then converted into an immutable reference. This is safe only
    /// under the assumption that no other part of the code violates Rust's aliasing rules
    /// by attempting to modify the static data concurrently.
    ///
    /// # Panics
    /// This function will panic if the `kernel_time_data` field in `KERNEL_DATA`
    /// is not initialized (`None`).
    ///
    /// # Notes
    /// - The `#[allow(static_mut_refs)]` attribute is used to suppress warnings for the
    ///   unsafe
    #[allow(static_mut_refs)]
    pub fn time_data() -> &'static KernelTimeData {
        unsafe {
            if KERNEL_DATA.kernel_time_data.is_some() {
                KERNEL_DATA.kernel_time_data.as_mut().unwrap()
            } else {
                panic!("Time data not initialized");
            }
        }
    }
}
