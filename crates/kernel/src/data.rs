use crate::apps::AppsManager;
use crate::devices::DevicesManager;
use crate::errors_mgt::ErrorsManager;
use crate::scheduler::Scheduler;
use crate::terminal::Terminal;
use crate::{Mhz, Milliseconds};
use cortex_m::Peripherals;
use display::Display;
use hal_interface::Hal;

pub static mut KERNEL_DATA: Kernel = Kernel {
    cortex_peripherals: None,
    hal: None,
    kernel_time_data: None,
    terminal: None,
    scheduler: None,
    errors: None,
    display: None,
    apps: None,
    devices: None,
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

/// The `Kernel` struct represents the core of the embedded operating system,
/// managing and coordinating various system components and functionalities.
///
/// # Fields
///
/// * `cortex_peripherals` - An optional field that contains core Cortex-M peripherals,
///   such as NVIC, SysTick, and others, required for low-level system operations.
///   This field is wrapped in an `Option` to allow deferred initialization or possible absence
///   in certain configurations.
///
/// * `hal` - An optional field for accessing the Hardware Abstraction Layer (HAL)
///   to interact with the underlying hardware peripherals, such as GPIO, I2C, SPI, etc.
///   Allows for hardware abstraction and easier portability between various microcontrollers.
///
/// * `kernel_time_data` - An optional field containing the timekeeping data required by the kernel
///   for scheduling, delays, or other time-sensitive operations. Typically includes timing mechanisms
///   like system ticks or RTC access.
///
/// * `terminal` - An optional field representing the user interface through a terminal,
///   which may handle input and output operations for system communication or debugging purposes.
///
/// * `scheduler` - An optional field for the kernel's task scheduler, which is responsible for managing
///   and orchestrating the execution of tasks or threads. Handles process prioritization and switching.
///
/// * `errors` - An optional field for the error manager, which tracks and manages system errors
///   or exceptions. Provides mechanisms for error logging or recovery during runtime.
///
/// # Usage
///
/// The `Kernel` struct serves as a container for all critical system components. Each field
/// is optional, allowing for greater flexibility in struct initialization and enabling configurations
/// where certain components might not be present. For example, a minimal system might not require
/// a terminal or a scheduler but still depends on HAL and timing functionalities.
///
/// Instances of `Kernel` are typically initialized during system startup and provide a central
/// point of access for key functionalities and resources throughout the lifecycle of the system.
/// Ensure proper initialization of required fields before usage to prevent runtime errors.
///
pub struct Kernel {
    cortex_peripherals: Option<Peripherals>,
    hal: Option<Hal>,
    kernel_time_data: Option<KernelTimeData>,
    terminal: Option<Terminal>,
    scheduler: Option<Scheduler>,
    errors: Option<ErrorsManager>,
    display: Option<Display>,
    apps: Option<AppsManager>,
    devices: Option<DevicesManager>,
}

impl Kernel {
    /// Initializes the global kernel data structure with the provided components.
    ///
    /// # Arguments
    ///
    /// * `hal` - A hardware abstraction layer (`Hal`) instance for interacting with low-level hardware features.
    /// * `display` - A `Display` instance to handle graphical or textual output.
    /// * `kernel_time_data` - A `KernelTimeData` instance to manage kernel-related timing and scheduling.
    /// * `terminal` - A `Terminal` instance to handle terminal input/output interactions.
    /// * `scheduler` - A `Scheduler` instance responsible for managing task scheduling.
    /// * `errors` - An `ErrorsManager` instance for managing and reporting errors throughout the kernel.
    /// * `apps_manager` - An `AppsManager` instance for managing kernel applications.
    ///
    /// # Safety
    ///
    /// This function directly writes to the global `KERNEL_DATA` structure using `unsafe` code. It is the caller's
    /// responsibility to ensure that:
    /// 1. The provided components are properly initialized before calling this function.
    /// 2. The function is not called more than once, as it overwrites existing global data, which could lead to
    ///    undefined behavior.
    ///
    /// # Notes
    ///
    /// This function is meant to be called during the kernel initialization stage to set up all the required
    /// components for kernel operation.
    ///
    pub fn init_kernel_data(
        hal: Hal,
        display: Display,
        kernel_time_data: KernelTimeData,
        terminal: Terminal,
        scheduler: Scheduler,
        errors: ErrorsManager,
        apps_manager: AppsManager,
        devices: DevicesManager,
    ) {
        unsafe {
            KERNEL_DATA.hal = Some(hal);
            KERNEL_DATA.display = Some(display);
            KERNEL_DATA.kernel_time_data = Some(kernel_time_data);
            KERNEL_DATA.terminal = Some(terminal);
            KERNEL_DATA.scheduler = Some(scheduler);
            KERNEL_DATA.errors = Some(errors);
            KERNEL_DATA.apps = Some(apps_manager);
            KERNEL_DATA.devices = Some(devices);
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

    /// Provides a mutable reference to the global display driver.
    ///
    /// This function retrieves a mutable reference to the global `Display` object stored within
    /// the `KERNEL_DATA` structure. If the `Display` driver has already been initialized,
    /// it safely accesses the `Display`. If the driver is not initialized, it panics with an error message.
    ///
    /// # Safety
    /// - The function uses `unsafe` to access a static mutable reference. Static mutable references
    ///   can lead to undefined behavior if improperly used. Ensure no simultaneous mutable and immutable
    ///   borrows occur to maintain memory safety.
    /// - This function assumes that the global `KERNEL_DATA.display` has been properly initialized
    ///
    #[allow(static_mut_refs)]
    pub fn display() -> &'static mut Display {
        unsafe {
            if KERNEL_DATA.display.is_some() {
                KERNEL_DATA.display.as_mut().unwrap()
            } else {
                panic!("Display driver not initialized");
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

    /// Provides access to the global `ErrorsManager` instance.
    ///
    /// This function returns a static reference to the `ErrorsManager`. It ensures that the
    /// global `ErrorsManager` instance is properly initialized before providing access to it.
    /// If the `ErrorsManager` has not been initialized, the function will panic.
    ///
    /// # Safety
    ///
    /// This function uses unsafe code to dereference a potentially mutable static reference.
    /// While the `#[allow(static_mut_refs)]` attribute suppresses the warning for mutable
    /// references to a static variable, care must be taken to ensure this function is used
    /// correctly to avoid undefined behavior.
    ///
    /// # Panics
    ///
    /// This function will panic if the global `ErrorsManager` instance has not been
    /// initialized. Ensure that the `ErrorsManager` is initialized before calling this function.
    ///
    /// # Returns
    ///
    /// A static reference to the `ErrorsManager` instance.
    ///
    #[allow(static_mut_refs)]
    pub fn errors() -> &'static mut ErrorsManager {
        unsafe {
            if KERNEL_DATA.errors.is_some() {
                KERNEL_DATA.errors.as_mut().unwrap()
            } else {
                panic!("Errors manager is not initialized");
            }
        }
    }

    /// Provides mutable access to the global `AppsManager` instance.
    ///
    /// This function retrieves a mutable reference to the global instance of the
    /// `AppsManager` by accessing the `KERNEL_DATA.apps` field. If the `apps`
    /// field is not initialized (i.e., it contains `None`), the function will panic.
    ///
    /// # Safety
    /// This function uses `unsafe` code to dereference and return a mutable reference
    /// to a static variable. Since it allows mutable access to a static reference,
    /// this can lead to undefined behavior if multiple mutable references are created
    /// and used simultaneously. Use this function with caution and ensure that
    /// no data races or aliasing occur.
    ///
    /// # Panics
    /// This function will panic if the `KERNEL_DATA.apps` field is not initialized
    /// (i.e., contains `None`).
    ///
    /// # Returns
    /// A mutable reference to the global `AppsManager` instance.
    ///
    #[allow(static_mut_refs)]
    pub fn apps() -> &'static mut AppsManager {
        unsafe {
            if KERNEL_DATA.apps.is_some() {
                KERNEL_DATA.apps.as_mut().unwrap()
            } else {
                panic!("Apps manager is not initialized");
            }
        }
    }
    
    /// Provides mutable access to the global `DevicesManager` instance.
    ///
    /// This function retrieves a mutable reference to the global instance of the
    /// `DevicesManager` by accessing the `KERNEL_DATA.devices` field. If the `devices`
    /// field is not initialized (i.e., it contains `None`), the function will panic.
    ///
    /// # Safety
    /// This function uses `unsafe` code to dereference and return a mutable reference
    /// to a static variable. Since it allows mutable access to a static reference,
    /// this can lead to undefined behavior if multiple mutable references are created
    /// and used simultaneously. Use this function with caution and ensure that
    /// no data races or aliasing occur.
    ///
    /// # Panics
    /// This function will panic if the `KERNEL_DATA.devices` field is not initialized
    /// (i.e., contains `None`).
    ///
    /// # Returns
    /// A mutable reference to the global `DevicesManager` instance.
    ///
    #[allow(static_mut_refs)]
    pub fn devices() -> &'static mut DevicesManager {
        unsafe {
            if KERNEL_DATA.devices.is_some() {
                KERNEL_DATA.devices.as_mut().unwrap()
            } else {
                panic!("Devices manager is not initialized");
            }
        }
    }
}

/// Initializes the Cortex-M peripherals used by the kernel.
///
/// This function is responsible for initializing the peripherals of the Cortex-M microcontroller
/// that the kernel depends on. It accesses the global `KERNEL_DATA` structure and assigns the
/// retrieved peripherals object to the `cortex_peripherals` field.
///
/// # Safety
///
/// This function performs an unsafe operation to directly modify the global `KERNEL_DATA` structure.
/// It assumes exclusive access to this data structure and relies on the safe initialization of
/// `KERNEL_DATA` and the presence of Cortex-M peripherals.
///
/// Calling this function multiple times without proper synchronization or in an invalid state
/// may result in undefined behavior.
///
/// # Panics
///
/// This function will panic if it fails to retrieve the Cortex-M peripherals via `Peripherals::take()`,
/// which occurs if the peripherals have already been taken elsewhere in the program.
///
pub fn cortex_init() {
    unsafe {
        KERNEL_DATA.cortex_peripherals = Some(Peripherals::take().unwrap());
    }
}
