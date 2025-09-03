use cortex_m::Peripherals;
use hal_interface::Hal;

use crate::terminal::Terminal;
use panic_semihosting as _;

/// A mutable static instance of the `Kernel` structure.
///
/// # Overview
/// The `KERNEL_DATA` variable serves as a global instance of the `Kernel` structure.
/// It is mutable and uninitialized by default, allowing it to be configured
/// during runtime. This data structure holds necessary details regarding the
/// kernel's context, including its peripherals, HAL (Hardware Abstraction Layer),
/// core frequency, and terminal.
///
/// # Fields
/// - `cortex_peripherals`: (Optional) Represents the Cortex-M peripherals,
///   such as system control or NVIC. Initialized to `None` by default.
/// - `hal`: (Optional) Represents the Hardware Abstraction Layer instance for
///   interacting with hardware components. Initialized to `None` by default.
/// - `core_freq`: Represents the frequency of the core in Hz. Initialized
///   to `0` by default.
/// - `terminal`: (Optional) Represents the kernel's terminal instance,
///   typically used for I/O (Input/Output) operations. Initialized to `None`
///   by default.
///
/// # Safety
/// Since `KERNEL_DATA` is declared as `pub static mut`, it is inherently unsafe
/// due to potential data races or undefined behavior when accessed by multiple
/// threads. It must be used with caution, and any access to this variable
/// should be wrapped in appropriate synchronization mechanisms (e.g., critical
/// sections, mutexes) to ensure thread safety.
///
/// # Note
/// Modification of this static should always occur in a controlled environment
/// where no other threads can access the variable simultaneously. Mismanagement
/// may introduce undefined behavior or system instability.
///
/// # Context
/// This `Kernel` structure and its fields are essential for coordinating
/// hardware
pub static mut KERNEL_DATA: Kernel = Kernel {
    cortex_peripherals: None,
    hal: None,
    core_freq: 0,
    terminal: None,
};

/// The `Kernel` struct represents the core entity responsible for managing
/// hardware peripherals and hardware abstraction layer (HAL) components
/// within the application.
///
/// This structure encapsulates two key optional elements:
/// - `cortex_peripherals`: An optional field representing the peripherals
///   specific to the Cortex microcontroller, typically used for interacting
///   with hardware at a low level. This uses the `Peripherals` type.
/// - `hal`: An optional field encapsulating the hardware abstraction layer,
///   which provides higher-level functionality over the raw hardware peripherals.
///
/// # Fields
/// * `cortex_peripherals` - Optionally holds the `Peripherals` object,
///   enabling direct control over the underlying microcontroller features.
/// * `hal` - Optionally holds the `Hal` object, providing a layer of abstraction
///   for easier access to the microcontroller's features.
///
/// The `Kernel` struct is designed to allow modular and flexible initialization
/// for embedded systems, enabling the user to configure these components as required.
pub struct Kernel {
    cortex_peripherals: Option<Peripherals>,
    hal: Option<Hal>,
    core_freq: u32,
    terminal: Option<Terminal>,
}

impl Kernel {
    /// Initializes the kernel data with the given hardware abstraction layer, core frequency, and terminal instance.
    ///
    /// # Parameters
    /// - `hal`: An instance of the hardware abstraction layer (`Hal`) which provides access to hardware-specific functionality.
    /// - `core_freq`: The core frequency of the system, represented as a `u32`.
    /// - `terminal`: An instance of the `Terminal` structure, used for output or logging operations.
    ///
    /// # Safety
    /// This function modifies a global static structure (`KERNEL_DATA`) within an `unsafe` block. It relies on the assumption
    /// that:
    /// - `Peripherals::take()` is called only once during the lifetime of the program (i.e., peripherals are not re-initialized).
    /// - The provided `hal`, `core_freq`, and `terminal` arguments are valid and properly initialized before being passed to this function.
    ///
    /// # Side Effects
    /// - Updates the `KERNEL_DATA` structure with the provided `hal`, `core_freq`, and `terminal`.
    /// - Sets `KERNEL_DATA.cortex_peripherals` to the system's core peripherals using `Peripherals::take()`.
    ///
    /// **Warning:** Improper invocation of this function or passing invalid parameters may lead to undefined behavior.
    pub fn init_kernel_data(hal: Hal, core_freq: u32, terminal: Terminal) {
        unsafe {
            KERNEL_DATA.cortex_peripherals = Some(Peripherals::take().unwrap());
            KERNEL_DATA.hal = Some(hal);
            KERNEL_DATA.core_freq = core_freq;
            KERNEL_DATA.terminal = Some(terminal);
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
}
