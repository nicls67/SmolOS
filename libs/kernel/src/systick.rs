use crate::Milliseconds;
use crate::data::Kernel;
use core::sync::atomic::{AtomicU32, Ordering};
use cortex_m::peripheral::SCB;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::exception;

static SCHED_TICKS_COUNTER: AtomicU32 = AtomicU32::new(0);
static SCHED_TICKS_TARGET: AtomicU32 = AtomicU32::new(0);

/// Initializes the system timer (Systick) with a specified or default period.
///
/// This function configures the SysTick timer to generate periodic interrupts
/// based on the provided time period. If no period is specified, it assumes
/// a default period of 1 millisecond based on a core frequency of 16 MHz.
///
/// # Arguments
///
/// * `period` - An optional `Milliseconds` value specifying the period for the SysTick timer.
///              If `None` is provided, a default period of 1 millisecond is used.
///
/// # Behavior
///
/// - The SysTick timer is configured to use the core clock as its clock source.
/// - The current value of the SysTick counter is cleared before initialization.
/// - If a period is specified, the reload value for the SysTick timer is calculated
///   based on the core frequency and the given period. If no period is specified,
///   a default reload value corresponding to 1 millisecond is used.
/// - Enables the SysTick interrupt and starts the SysTick counter.
///
/// # Assumptions
///
/// - The default core frequency is assumed to be 16 MHz unless a specific period
///   dependent on another core frequency is explicitly provided.
///
/// # Notes
///
/// - Ensure that the provided core frequency value in `Kernel::time_data().core_frequency`
///   matches the actual system clock frequency for correct timer behavior.
/// - Ensure that `Kernel::cortex_peripherals()` is properly set up before invoking this function.
///
pub fn init_systick(period: Option<Milliseconds>) {
    // Initialize Systick at 1ms
    let cortex_p = Kernel::cortex_peripherals();
    cortex_p.SYST.set_clock_source(SystClkSource::Core);
    cortex_p.SYST.clear_current();

    if let Some(period) = period {
        cortex_p
            .SYST
            .set_reload(Kernel::time_data().core_frequency.to_u32() * period.0 / 1000);
    } else {
        // The default core frequency is 16 MHz, so 1 ms is 16,000 ticks
        cortex_p.SYST.set_reload(16_000);
    }

    cortex_p.SYST.enable_interrupt();
    cortex_p.SYST.enable_counter();
}

/// Sets the target value for scheduling ticks.
///
/// This function updates the `SCHED_TICKS_TARGET` with the provided `target` value.
/// The `Ordering::Relaxed` is used for the atomic operation, indicating that there
/// are no synchronization requirements for this store operation.
///
/// ## Parameters
/// - `target`: The target value for scheduling ticks, represented as a `u32`.
///
/// Note: Ensure that the chosen `target` value is suitable for your application's
/// scheduling requirements.
pub fn set_ticks_target(target: u32) {
    SCHED_TICKS_TARGET.store(target, Ordering::Relaxed);
}

/// Handles the SysTick exception (system timer interrupt).
///
/// This function is executed whenever the SysTick interrupt occurs, typically at regular
/// intervals to manage system timing and scheduling. Within the handler:
///
/// 1. It checks whether a rescheduling event should be triggered based on the system's
///    scheduling tick configuration (`SCHED_TICKS_TARGET`).
/// 2. If the current system tick matches the target interval, a PendSV interrupt is triggered
///    by calling `SCB::set_pendsv()` to handle any context switch for task scheduling.
/// 3. Independently of scheduling, the system tick counter is incremented by calling `HAL_IncTick()`.
///
/// # Globals:
/// - `SCHED_TICKS_TARGET`: A globally stored value used to define the target tick interval at
///   which rescheduling events should be considered. It is accessed using `Relaxed` memory ordering.
///
/// # Behavior:
/// - If `SCHED_TICKS_TARGET` is not zero and the current system tick (`HAL_GetTick()`) is divisible
///   by this target value, the handler requests a PendSV exception for context switching.
/// - Regardless of the rescheduling condition, the system tick counter is incremented.
///
/// # Safety:
/// - Interrupt handlers execute at a higher privilege level and must execute efficiently
///   to maintain system stability. Ensure that all operations within this function are
///   deterministic and bounded in execution time.
///
/// # Requirements:
/// - The `HAL_GetTick()` function provides the current system tick count.
/// - The `HAL_IncTick()` function increments the system tick counter.
/// - The `SCB::set_pendsv()` function triggers a PendSV exception for task switching.
///
/// # Notes:
/// - This function is part of the exception handling mechanism and should always remain
///   minimal in execution to avoid delaying other system-critical interrupts.
#[exception]
fn SysTick() {
    if SCHED_TICKS_TARGET.load(Ordering::Relaxed) != 0
        && HAL_GetTick() % SCHED_TICKS_TARGET.load(Ordering::Relaxed) == 0
    {
        SCB::set_pendsv();
    }

    HAL_IncTick();
}

/// Increments the system tick counter.
///
/// # Safety
/// This function is marked `unsafe` and uses `#[no_mangle]` to ensure it can be
/// called from external code such as a hardware abstraction layer (HAL) written in a different
/// programming language (e.g., C). Special care must be taken to ensure safe use when
/// integrating it into low-level system implementations.
///
/// # Details
/// - This function is designed to increment the `SCHED_TICKS_COUNTER` atomic counter, which
///   is intended to track system ticks, typically used for scheduling or timekeeping purposes.
/// - The increment operation is performed using `Ordering::Relaxed` to minimize synchronization
///   overhead, under the assumption that other parts of the program do not depend on strong ordering
///   guarantees.
///
/// # Usage
/// This function can be called from an external environment (e.g., embedded firmware) where a
/// regular system tick interrupt increments a global or shared tick counter.
///
/// # No-mangle Usage
/// The `#[no_mangle]` attribute ensures this function's symbol name in the compiled binary
/// is `HAL_IncTick`, making it accessible in systems or languages that call it by its exact name.
///
/// # Caveats
/// - Ensure atomic access to `SCHED_TICKS_COUNTER` aligns with the consistency requirements of the
///   system, as the `Relaxed` ordering does not impose any synchronization constraints.
///
/// # Note
/// This function is meant to be lightweight and efficient for real-time contexts where minimal
/// overhead is crucial. Incorrect usage of atomic counter manipulation or modification of the
/// counter elsewhere without proper synchronization could lead to undefined behavior.
#[unsafe(no_mangle)]
pub extern "C" fn HAL_IncTick() {
    SCHED_TICKS_COUNTER.fetch_add(1, Ordering::Relaxed);
}

/**
 * @brief Returns the current tick count from the scheduler ticker.
 *
 * This function provides an interface for retrieving the current
 * value of the `SCHED_TICKS_COUNTER`, which represents the number
 * of ticks elapsed since the system started. It is an external
 * interface designed for usage with foreign function interfaces
 * (FFI) and declares `no_mangle` to preserve the function name
 * for linkage purposes.
 *
 * # Safety
 * This function is marked as `unsafe` because it is intended to be used
 * in an FFI context, where the caller must ensure that it is called in
 * a safe manner. Callers should also ensure thread safety when accessing
 * the tick count.
 *
 * # Returns
 * A `u32` value representing the current tick count stored in `SCHED_TICKS_COUNTER`.
 * The value is loaded using relaxed memory ordering.
 *
 * # Attributes
 * - `#[no_mangle]`: Ensures the function name is preserved, making it accessible
 *   with its original name during FFI calls.
 * - `pub extern "C"`: Marks the function as a public externally-linked function
 *   following the C calling convention for cross-language compatibility.
 *
 */
#[unsafe(no_mangle)]
pub extern "C" fn HAL_GetTick() -> u32 {
    SCHED_TICKS_COUNTER.load(Ordering::Relaxed)
}

///
/// # HAL_Delay Function
///
/// This is an unsafe extern "C" function that halts program execution for a specified amount
/// of time, measured in milliseconds. The function is primarily used in embedded systems
/// where precise timing control is required. The function relies on polling the system
/// tick counter (`HAL_GetTick`) to determine when the delay period has elapsed.
///
/// ## Parameters
/// - `ms: u32`
///   - The number of milliseconds to delay execution. If `ms` is 0, it is automatically set to 1 to ensure at least a minimal delay.
///
/// ## Behavior
/// - The function computes a target tick count by adding the provided `ms` value to the current result of `HAL_GetTick`.
/// - It then executes a busy-wait loop, continuously checking the current tick count until the target tick count is reached.
///
/// ## Safety
/// - This function is marked as `unsafe` due to its potential side effects on deterministic program behavior and reliance on external hardware timer mechanisms.
/// - The `#[no_mangle]` attribute disables name mangling to make the function accessible by C and other ABI-compatible languages.
/// - It operates in an infinite busy loop until the target tick count is achieved, which may cause high CPU utilization and block other tasks from running (e.g
#[unsafe(no_mangle)]
pub extern "C" fn HAL_Delay(mut ms: u32) {
    if ms == 0 {
        ms = 1;
    }
    let ticks = HAL_GetTick() + ms;
    while HAL_GetTick() < ticks {}
}

/// The PendSV (Pendable Service Call) exception handler.
///
/// This function is marked with the `#[exception]` attribute, indicating that it handles
/// the PendSV exception for the system. PendSV is typically used in embedded systems to
/// handle context switching in operating systems.
///
/// In this implementation, the PendSV handler invokes a periodic task scheduler from
/// the `Kernel` module. The `Kernel::scheduler()` function retrieves the scheduler
/// instance, and the `periodic_task()` method is called to perform periodic task
/// management, such as context switching or scheduling tasks in a real-time operating system.
///
/// Note:
/// - This function is designed to work in tandem with an embedded operating system kernel.
/// - Proper setup of the PendSV exception and system configuration is required for this
///   function to be effective.
#[exception]
fn PendSV() {
    Kernel::scheduler().periodic_task();
}
