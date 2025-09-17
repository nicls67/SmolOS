use crate::KernelError::CannotAddNewPeriodicApp;
use crate::data::Kernel;
use crate::except::set_ticks_target;
use crate::{KernelError, KernelResult, Milliseconds, TerminalFormatting};
use cortex_m::peripheral::scb::SystemHandler;
use heapless::{String, Vec};

/// Type alias `App` represents a function pointer type that returns a `KernelResult<()>`.
///
/// This type alias is used as a shorthand for functions that are intended to serve
/// as entry points or main execution units within the application. The generic
/// `KernelResult<()>` type encapsulates the result of the function, indicating
/// either successful execution (with an empty `()` value) or an error.
///
/// # Type Signature
/// - `fn() -> KernelResult<()>`
///   - `fn()` indicates a function with no parameters.
///   - `KernelResult<()>` signifies the function's return type:
///     - `Ok(())` if the operation is successful.
///     - `Err(err)` if an error occurs, where `err` represents the specific failure.
///
/// This type alias improves code readability and reduces verbosity, particularly
/// in scenarios where the same function signature is repeatedly defined.
///
pub type App = fn() -> KernelResult<()>;

/// A structure that wraps an application with additional metadata.
///
/// The `AppWrapper` struct is used to encapsulate an application instance
/// (`app`) along with its metadata, such as its name, operational period,
/// and whether it is currently active or not.
///
/// # Fields
///
/// * `name` - A string containing the name of the application.
///   The name has a maximum length of 32 characters.
/// * `app` - The actual application instance encapsulated within the wrapper.
/// * `app_period` - A 32-bit unsigned integer specifying the operational
///   duration or period of the application. This might represent time or specific
///   interval information, depending on the context of use.
/// * `active` - A boolean flag indicating whether the application is currently active
///   (`true`) or inactive (`false`).
///
struct AppWrapper {
    name: String<32>,
    app: App,
    app_period: u32,
    active: bool,
}
/// The `Scheduler` struct is responsible for managing and executing a collection
/// of tasks that have been scheduled. It keeps track of task execution order,
/// maintains the state of its operation, and handles task-related errors.
///
/// # Fields
///
/// * `tasks`:
///   A fixed-size vector of up to 128 tasks wrapped in `AppWrapper`. This vector
///   stores all the tasks that are managed and scheduled by the `Scheduler`.
///
/// * `cycle_counter`:
///   A 32-bit unsigned integer that tracks the number of completed scheduling
///   cycles. It increments periodically whenever the scheduler completes a full cycle
///   through its tasks.
///
/// * `sched_period`:
///   Represents the interval between scheduler cycles expressed in `Milliseconds`.
///   This period defines how frequently the scheduler executes and rotates through
///   its tasks.
///
/// * `started`:
///   A boolean flag that indicates whether the scheduler is currently active (`true`)
///   or stopped (`false`). Starting the scheduler initializes task execution and
///   cycling.
///
/// * `current_task_id`:
///   An optional `usize` that stores the index of the currently executing task within
///   the `tasks` list. If `None`, it indicates that no task is currently being executed.
///
/// * `current_task_has_error`:
///   A boolean flag that reflects whether the currently executing task has encountered
///   an error (`true`). This is used to record task failures during execution and may
///   influence scheduler behavior.
pub struct Scheduler {
    tasks: Vec<AppWrapper, 128>,
    cycle_counter: u32,
    sched_period: Milliseconds,
    pub started: bool,
    current_task_id: Option<usize>,
    current_task_has_error: bool,
}

impl Scheduler {
    /// Creates and initializes a new `Scheduler` instance.
    ///
    /// # Parameters
    /// - `period`: The scheduling period in milliseconds, represented as a `Milliseconds` type.
    ///   This defines the interval at which the scheduler cycles through its tasks.
    ///
    /// # Returns
    /// Returns a `Scheduler` instance with the following default configuration:
    /// - `tasks`: An empty vector to store scheduled tasks.
    /// - `cycle_counter`: Initialized to `0` to track the number of completed scheduler cycles.
    /// - `sched_period`: Set to the provided `period`, determining how often the scheduler runs.
    /// - `started`: Set to `false`, indicating that the scheduler has not yet started.
    /// - `current_task_id`: Set to `None`, as no task is currently being executed.
    /// - `current_task_has_error`: Set to `false`, indicating no task errors have been encountered.
    ///
    /// Use this constructor to create a new instance of the `Scheduler` and begin adding tasks or configuring it based on specified requirements.
    pub fn new(period: Milliseconds) -> Scheduler {
        Scheduler {
            tasks: Vec::new(),
            cycle_counter: 0,
            sched_period: period,
            started: false,
            current_task_id: None,
            current_task_has_error: false,
        }
    }

    /// Starts the kernel's scheduler with the specified system tick period.
    ///
    /// # Parameters
    /// - `systick_period`: The duration of the system tick interval, specified as a `Milliseconds` value.
    ///   This parameter represents the periodicity at which the system tick timer is triggered.
    ///
    /// # Returns
    /// - `KernelResult<()>`: Returns `Ok(())` if the scheduler starts successfully. If any error occurs
    ///   during the operation, an appropriate error result is returned.
    ///
    /// # Description
    /// This function initializes and starts the kernel's scheduler by performing the following steps:
    /// 1. Retrieves the Cortex-M peripherals through `Kernel::cortex_peripherals()`.
    /// 2. Configures the priority of the PendSV system handler to the lowest priority (`0xFF`).
    /// 3. Sets the target tick period for the scheduler by computing the ratio of the scheduler period to the
    ///    system tick period (`self.sched_period.to_u32() / systick_period.to_u32()`).
    /// 4. Enables the SysTick counter by calling `cortex_p.SYST.enable_counter()`.
    /// 5. Marks the scheduler as started by setting `self.started` to `true`.
    ///
    /// Additionally, the function writes a notification message, "Scheduler started!", to the kernel's
    /// terminal to indicate successful start of the scheduler.
    ///
    /// # Safety
    /// Unsafe code is used to set the priority of the PendSV system handler and configure the scheduler's
    /// tick period. Care should be taken to ensure proper configuration of system-level components to avoid
    /// unintended behavior or crashing.
    ///
    /// # Errors
    /// - Returns a `KernelResult` with an error value if the terminal writing operation fails.
    ///
    /// # Notes
    /// - The function assumes that `self.sched_period` is already set prior to calling this function.
    /// - The function writes a status message to the terminal, which may fail if the terminal subsystem is not ready.
    ///
    /// # Requirements
    /// - The system must support Cortex-M processor peripherals as used within this implementation.
    /// - The `set_ticks_target()` function must handle the tick period computation correctly.
    ///
    /// # See Also
    /// - `Kernel::cortex_peripherals()`
    /// - `Kernel::terminal()`
    /// - `cortex_m::peripheral::SYST::enable_counter`
    pub fn start(&mut self, systick_period: Milliseconds) -> KernelResult<()> {
        let cortex_p = Kernel::cortex_peripherals();

        // Initialize scheduler periodic IT
        unsafe {
            cortex_p.SCB.set_priority(SystemHandler::PendSV, 0xFF);
            set_ticks_target(self.sched_period.to_u32() / systick_period.to_u32())
        }

        cortex_p.SYST.enable_counter();
        self.started = true;
        Kernel::terminal().write(&TerminalFormatting::StrNewLineBoth("Scheduler started !"))
    }

    /// Adds a periodic application to the kernel.
    ///
    /// This function attempts to initialize and register a new application
    /// that will execute periodically. The user needs to provide the application's
    /// name, the periodic application object, its initializer, and the desired
    /// execution period in milliseconds.
    ///
    /// # Parameters
    /// - `name`: A string slice representing the name of the application.
    /// - `app`: The actual application object that will be executed periodically.
    /// - `init`: A function or closure that initializes the application before it is added.
    /// - `period`: The execution period of the application, represented in milliseconds.
    ///
    /// # Returns
    /// - `KernelResult<()>`: Returns `Ok(())` if the application is successfully added
    ///   and scheduled. Otherwise, returns an appropriate error wrapped in `KernelError`.
    ///
    /// # Errors
    /// - `KernelError::AppInitError`: If the `init` function fails during initialization.
    /// - `KernelError::CannotAddNewPeriodicApp`: If the scheduler cannot register the application.
    ///
    /// # Behavior
    /// 1. The function attempts to initialize the application by invoking the provided `init` function.
    ///    If the initialization fails, it returns a `KernelError::AppInitError`.
    /// 2. If initialization succeeds, the function registers the application in the scheduler by
    ///    adding it to the internal `tasks` list. The registration associates the app name, application
    ///    object, computed period in scheduler ticks (derived from the app's
    pub fn add_periodic_app(
        &mut self,
        name: &str,
        app: App,
        init: App,
        period: Milliseconds,
    ) -> KernelResult<()> {
        let app_name = String::from(name.parse().unwrap());

        // Try to initialize the app
        init().map_err(|_| KernelError::AppInitError(app_name.clone()))?;

        // Register app in the scheduler
        self.tasks
            .push(AppWrapper {
                name: app_name.clone(),
                app,
                app_period: period.to_u32() / self.sched_period.to_u32(),
                active: true,
            })
            .map_err(|_| CannotAddNewPeriodicApp(app_name))
    }

    /// Executes periodic tasks stored in the kernel's task list.
    ///
    /// This function iterates through all tasks and checks if each task's period
    /// matches the current cycle count (`cycle_counter`). If the task's period matches
    /// and the task is marked as active, the task's application function is executed.
    /// Any errors encountered during the execution of a task are passed to the kernel's
    /// error handler.
    ///
    /// ### Behavior:
    /// - Each task has an associated period (`app_period`) and an active state (`active`).
    /// - The function ensures that tasks are run in their scheduled cycles based on the
    ///   period and the kernel's `cycle_counter`.
    /// - The currently running task's ID and any errors are tracked using `current_task_id`
    ///   and `current_task_has_error`.
    /// - If the task function completes successfully, no additional action is taken.
    /// - If the task function returns an error, the kernel's error handler processes it.
    ///
    /// ### Properties:
    /// - `self.cycle_counter`: Tracks the number of cycles the kernel has executed. Increments
    ///   after each call to this method.
    /// - `self.current_task_id`: Temporarily stores the ID of the currently running task for
    ///   tracking purposes.
    /// - `self.current_task_has_error`: Tracks whether an error was encountered in the
    ///   current task to avoid reporting the same error multiple times.
    ///
    /// ### Arguments:
    /// This method takes mutable access to `self` since it alters the state of the kernel,
    /// updating the cycle counter and tracking task execution state.
    ///
    /// ### Notes:
    /// - Tasks that are inactive (`task.active == false`) are skipped.
    /// - Task execution state is encapsulated using `current_task_id` to allow better
    ///   debugging or logging if necessary.
    pub fn periodic_task(&mut self) {
        // Run all tasks
        for (id, task) in self.tasks.iter_mut().enumerate() {
            if self.cycle_counter % task.app_period == 0 && task.active {
                self.current_task_id = Some(id);
                self.current_task_has_error = false;
                match (task.app)() {
                    Ok(..) => {}
                    Err(e) => {
                        if !self.current_task_has_error {
                            Kernel::errors().error_handler(&e);
                        }
                    }
                }
                self.current_task_id = None;
            }
        }

        self.cycle_counter += 1;
    }

    /// Aborts the currently active task by marking it as inactive and signaling that it encountered an error.
    ///
    /// # Behavior
    /// - If a task is currently active (indicated by `self.current_task_id` being `Some`):
    ///   - The task's `active` status is set to `false`.
    ///   - The `current_task_has_error` flag is set to `true` to indicate that an error occurred in the current task.
    /// - If no task is active (`self.current_task_id` is `None`), this function does nothing.
    ///
    /// # Usage
    /// Call this function to terminate the currently running task and flag it as having an error.
    /// Ensure that proper task management logic accounts for the inactive and erroneous state.
    ///
    pub fn abort_task_on_error(&mut self) {
        // Set the current task as inactive
        if let Some(id) = self.current_task_id {
            self.tasks[id].active = false;
            self.current_task_has_error = true;
        }
    }
}
