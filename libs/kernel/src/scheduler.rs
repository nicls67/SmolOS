use crate::KernelError::CannotAddNewPeriodicApp;
use crate::data::Kernel;
use crate::except::set_ticks_target;
use crate::{KernelError, KernelResult, Milliseconds, TerminalFormatting};
use cortex_m::peripheral::SCB;
use cortex_m::peripheral::scb::{Exception, SystemHandler, VectActive};
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

/// A type alias for a function pointer that takes an unsigned 32-bit integer (`u32`)
/// as an input parameter and returns a `KernelResult<()>`.
///
/// # Type Definition
/// ```
/// pub type AppParam = fn(u32) -> KernelResult<()>;
/// ```
///
/// # Usage
/// This type alias represents a callback or function interface that can be used within
/// the kernel or application context. The function is expected to perform an operation
/// based on the given `u32` parameter and return a `KernelResult<()>`.
///
/// The `KernelResult<()>` typically indicates whether the operation was
/// successful or encountered an error, following the convention of `Result<T, E>`
/// where `()` represents a unit type for successful results.
///
/// # Notes
/// - The `KernelResult` and potential return types (`Ok` or `Err`) need to be
///   appropriately defined in your context.
/// - This alias helps standardize function signatures across different modules.
pub type AppParam = fn(u32) -> KernelResult<()>;

/// The `AppCall` enum represents different ways an application can be invoked,
/// either without parameters or with a parameter and an associated value.
///
/// Variants:
///  - `AppNoParam(App)`:
///      Represents a call to an `App` without any additional parameters.
///      - `App`: The application being called.
///
///  - `AppParam(AppParam, u32)`:
///      Represents a call to an `App` with an associated parameter and a `u32` value.
///      - `AppParam`: The parameter linked to the application call.
///      - `u32`: An additional value associated with the parameter.
///
/// This enum provides a clear way to distinguish between calls with and without parameters,
/// allowing for flexible handling within applications.
pub enum AppCall {
    AppNoParam(App),
    AppParam(AppParam, u32),
}

/// `AppWrapper` is a structure that encapsulates metadata and runtime details for an application call.
/// It provides information about the application and its lifecycle configuration.
///
/// # Fields
///
/// * `name` - A fixed-size string (maximum 32 characters) representing the name of the application.
/// * `app` - An `AppCall` object that represents the callable application functionality or instance.
/// * `app_period` - A `u32` integer specifying the periodic execution interval of the application, in some time unit (e.g., seconds).
/// * `ends_in` - An optional `u32` specifying the remaining time duration (in the same time unit as `app_period`)
///   until the application is deactivated. If `None`, no expiration is set.
/// * `active` - A boolean indicating whether the application is currently active (`true`) or inactive (`false`).
///
struct AppWrapper {
    name: &'static str,
    app: AppCall,
    app_period: u32,
    ends_in: Option<u32>,
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

    /// Adds a periodic application to the kernel scheduler.
    ///
    /// This function registers an application to be periodically executed by
    /// the scheduler, according to the specified time period and optional
    /// expiration time.
    ///
    /// # Parameters
    /// - `name`: A string slice that represents the name of the application. It must
    ///   be a valid string that can be parsed and stored as a `String`.
    /// - `app`: The main application function to be executed periodically.
    /// - `init`: An optional initialization function for the application. If provided,
    ///   this will be called once to initialize the app. If the initialization fails,
    ///   a `KernelError::AppInitError` is returned.
    /// - `period`: The interval/duration (in milliseconds) at which the app will
    ///   be executed periodically.
    /// - `ends_in`: An optional duration (in milliseconds) that specifies when the
    ///   periodic app should stop running. If `None` is provided, the app runs indefinitely
    pub fn add_periodic_app(
        &mut self,
        name: &'static str,
        app: AppCall,
        init: Option<App>,
        period: Milliseconds,
        ends_in: Option<Milliseconds>,
    ) -> KernelResult<()> {
        // Try to initialize the app
        if let Some(init_func) = init {
            init_func().map_err(|_| KernelError::AppInitError(name))?;
        }

        // Register app in the scheduler
        self.tasks
            .push(AppWrapper {
                name,
                app,
                app_period: period.to_u32() / self.sched_period.to_u32(),
                active: true,
                ends_in: ends_in.map(|e| e.to_u32() / period.to_u32()),
            })
            .map_err(|_| CannotAddNewPeriodicApp(name))
    }

    /// Removes a periodic application task from the kernel's task list by its name.
    ///
    /// This function searches for a task with the specified name within the kernel's
    /// `tasks` list. If a matching task is found, it is removed from the list. If no
    /// task with the provided name is found, an error is returned.
    ///
    /// # Arguments
    ///
    /// * `name` - A static string slice that represents the name of the application task
    ///   to be removed. The name should match the name of the application in the task list.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the task was successfully located and removed from the list.
    /// * `Err(KernelError::AppNotFound)` - If no matching task is found in the tasks list.
    ///
    /// # Errors
    ///
    /// This function returns `KernelError::AppNotFound` if the specified task name
    /// does not exist in the kernel's task list.
    ///
    /// # Notes
    ///
    /// * The `name` parameter must be a static string (i.e., `'static` lifetime).
    /// * The `tasks` list is assumed to be a vector-like structure supporting `iter`
    ///   and `remove` operations.
    ///
    /// # Panics
    ///
    /// This function will panic if the conversion from `&str` to `String<32>` fails,
    /// which should not occur under normal circumstances if the input is valid.
    ///
    /// # Related
    ///
    /// * `Kernel::add_task` - To add tasks to the kernel's task list.
    /// * `KernelError` - Enum that defines possible kernel-related errors.
    pub fn remove_periodic_app(&mut self, name: &'static str) -> KernelResult<()> {
        let app_name: String<32> = String::from(name.parse().unwrap());
        for (index, task) in self.tasks.iter().enumerate() {
            if task.name == app_name {
                self.tasks.swap_remove(index);
                return Ok(());
            }
        }
        Err(KernelError::AppNotFound(name))
    }

    /// Executes periodic tasks in the system based on their configured execution period.
    ///
    /// This function iterates through the list of tasks and checks each one to determine
    /// if it should be executed during the current cycle. A task is executed when the following
    /// conditions are met:
    /// - The task is active (`task.active` is `true`).
    /// - The system's `cycle_counter` modulo `task.app_period` equals `0` (indicating it's time to run the task).
    ///
    /// Each task can be either:
    /// - A function without parameters (`AppCall::AppNoParam`), or
    /// - A function with parameters (`AppCall::AppParam`).
    ///
    /// When a task is executed:
    /// - If the task runs successfully, no additional action is taken.
    /// - If the task returns an error, the system's error handler (`Kernel::errors().error_handler()`) is invoked
    ///   to handle the error. Errors are only reported once per task invocation to avoid redundant handling.
    ///
    /// The function also updates:
    /// - `current_task_id` to indicate which task is currently being executed, or `None` when no task is running.
    /// - `current_task_has_error` to track whether a task has encountered an error.
    ///
    /// After all tasks are evaluated, the function increments the `cycle_counter` by 1 to advance to the next cycle.
    ///
    /// # Notes
    ///
    /// - If a task is inactive (`task.active` is `false`), it will be skipped entirely.
    /// - Tasks with execution periods that do not align with the current `cycle_counter` are not executed in that cycle.
    pub fn periodic_task(&mut self) {
        let mut tasks_to_remove: Vec<&'static str, 8> = Vec::new();

        // Run all tasks
        for (id, task) in self.tasks.iter_mut().enumerate() {
            if self.cycle_counter % task.app_period == 0 && task.active {
                self.current_task_id = Some(id);
                self.current_task_has_error = false;
                match task.app {
                    AppCall::AppNoParam(app) => match app() {
                        Ok(..) => {}
                        Err(e) => {
                            if !self.current_task_has_error {
                                Kernel::errors().error_handler(&e);
                            }
                        }
                    },
                    AppCall::AppParam(app, param) => match app(param) {
                        Ok(..) => {}
                        Err(e) => {
                            if !self.current_task_has_error {
                                Kernel::errors().error_handler(&e);
                            }
                        }
                    },
                }
                self.current_task_id = None;
                if task.ends_in.is_some() {
                    task.ends_in = task.ends_in.map(|e| e - 1);
                    if task.ends_in.unwrap() == 0 {
                        tasks_to_remove.push(task.name).unwrap();
                    }
                }
            }
        }

        // Remove tasks that have ended
        for task_name in tasks_to_remove {
            self.remove_periodic_app(task_name).unwrap();
        }

        self.cycle_counter += 1;
    }

    /// Aborts the current task when an error occurs during the PendSV exception.
    ///
    /// This function is designed to be executed during the PendSV exception,
    /// which is typically used for context switching in embedded systems.
    /// If the PendSV exception is active, the function will retrieve the ID
    /// of the currently running task. It then marks the task as inactive and
    /// sets a flag indicating that the task encountered an error, preventing
    /// it from further execution.
    ///
    /// # Behavior
    /// - This function performs no action if the PendSV exception is not active.
    /// - If the PendSV exception is active, the task with the ID stored in
    ///   `self.current_task_id` is marked as inactive, and
    ///   `self.current_task_has_error` is set to `true`.
    /// - It assumes that `self.current_task_id` is `Some`, and the corresponding
    ///   task exists in the `self.tasks` list.
    ///
    /// # Usage
    /// This function should be called during the PendSV exception handler to
    /// handle tasks that encounter
    pub fn abort_task_on_error(&mut self) {
        if SCB::vect_active() == VectActive::Exception(Exception::PendSV) {
            // Set the current task as inactive
            if let Some(id) = self.current_task_id {
                self.tasks[id].active = false;
                self.current_task_has_error = true;
            }
        }
    }

    /// Checks if an application with the given name exists within the stored tasks.
    ///
    /// # Parameters
    /// - `name`: A string slice representing the name of the application to search for.
    ///
    /// # Returns
    /// - `true`: If an application with the specified name exists.
    /// - `false`: If no application with the specified name is found.
    ///
    /// # Panics
    /// - This function will panic if the provided `name` cannot be parsed into a valid `String<32>`.
    ///
    pub fn app_exists(&self, name: &str) -> bool {
        let app_name: String<32> = String::from(name.parse().unwrap());
        for task in self.tasks.iter() {
            if task.name == app_name {
                return true;
            }
        }
        false
    }
}
