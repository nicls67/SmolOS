use crate::KernelError::CannotAddNewPeriodicApp;
use crate::data::Kernel;
use crate::systick::set_ticks_target;
use crate::{KernelError, KernelResult, Milliseconds, TerminalFormatting};
use cortex_m::peripheral::SCB;
use cortex_m::peripheral::scb::{Exception, SystemHandler, VectActive};
use heapless::Vec;

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

/// Represents different types of application calls within a system.
///
/// This enum encapsulates various variants of calls to an app, with or without parameters.
///
/// # Variants
///
/// - `AppNoParam`:
///   - Represents a call to an app that does not require any parameters.
///   - Fields:
///     - `App`: The primary application to be called.
///     - `Option<App>`: An optional secondary application reference.
///
/// - `AppParam`:
///   - Represents a call to an app that requires parameters.
///   - Fields:
///     - `AppParam`: The parameter structure associated with the application call.
///     - `u32`: A numeric identifier or parameter for the call.
///     - `Option<App>`: An optional secondary application reference.
///
pub enum AppCall {
    AppNoParam(App, Option<App>),
    AppParam(AppParam, u32, Option<App>),
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
/// Struct representing a Scheduler, which manages tasks and their execution
/// in a cyclic time period.
///
/// The `Scheduler` is responsible for maintaining a collection of tasks,
/// orchestrating their execution based on a periodic schedule, and handling
/// runtime states like error occurrences or the currently executing task.
///
/// # Fields
/// * `tasks` - A fixed-size vector containing the scheduled tasks (`AppWrapper`) managed by the scheduler.
///             Limited to a size of 128.
/// * `cycle_counter` - A counter representing the number of completed execution cycles.
/// * `sched_period` - The scheduling period, represented in milliseconds, specifying the frequency
///                    at which the scheduler cycles through tasks.
/// * `started` - A public boolean indicating whether the scheduler has been started for execution.
/// * `current_task_id` - An optional `usize` representing the index of the currently executing task within the `tasks` vector.
///                       If no task is currently active, it is `None`.
/// * `current_task_has_error` - A boolean flag indicating whether the currently executing task has encountered an error.
/// * `next_id` - A unique identifier (`u32`) for assigning to newly added tasks within the scheduler.
///
pub struct Scheduler {
    tasks: Vec<AppWrapper, 128>,
    cycle_counter: u32,
    sched_period: Milliseconds,
    pub started: bool,
    current_task_id: Option<usize>,
    current_task_has_error: bool,
    next_id: u32,
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
            next_id: 0,
        }
    }

    /// Starts the kernel scheduler with a specified SysTick period.
    ///
    /// This method initializes the scheduler by configuring the PendSV interrupt priority
    /// and calculates the target ticks for the scheduler based on the specified SysTick period.
    /// It also logs a message indicating that the scheduler has started successfully. The method
    /// overrides the `self.started` flag to ensure the scheduler can run.
    ///
    /// # Parameters
    /// - `systick_period`: The period or duration of a single SysTick in milliseconds, which is
    /// used to calculate the number of ticks between scheduler executions.
    ///
    /// # Returns
    /// - `KernelResult<()>`: Returns an empty result on success, or an error if the operation fails.
    ///
    /// # Panics
    /// This method may panic if there is an issue interacting with the underlying terminal logging system.
    ///
    /// # Safety
    /// This method uses unsafe code to manipulate hardware peripherals. It sets the priority for the PendSV
    /// interrupt to its lowest value (0xFF), and initializes periodic interrupts by setting the scheduler ticks target.
    /// The unsafe block must ensure safe interaction with shared hardware resources to avoid undefined behavior.
    ///
    pub fn start(&mut self, systick_period: Milliseconds) -> KernelResult<()> {
        let cortex_p = Kernel::cortex_peripherals();

        // Initialize scheduler periodic IT
        unsafe {
            cortex_p.SCB.set_priority(SystemHandler::PendSV, 0xFF);
            set_ticks_target(self.sched_period.to_u32() / systick_period.to_u32())
        }

        self.started = true;
        Kernel::terminal().write(&TerminalFormatting::StrNewLineBoth("Scheduler started !"))
    }

    /// Adds a periodic application to the scheduler.
    ///
    /// This function registers a periodic application with the scheduler.
    /// It verifies if the application already exists, optionally initializes
    /// the application, and schedules it to be executed at the specified intervals.
    ///
    /// # Parameters
    /// - `name`: A static string slice that represents the unique name of the application.
    /// - `app`: An `AppCall` variant specifying the callback function that implements the app's functionality.
    /// - `init`: An optional initialization function (`App`) to be executed before scheduling the application. This can be `None` if no initialization is required.
    /// - `period`: The periodic interval (in milliseconds) at which the application will be executed.
    /// - `ends_in`: An optional time duration (in milliseconds) specifying when the application should stop executing.
    ///              If `None`, the application will run indefinitely.
    ///
    /// # Returns
    /// Returns a `KernelResult<u8>` which contains:
    /// - `Ok(u32)`: If the application is successfully registered, the application ID is returned.
    /// - `Err(KernelError)`: Returns an error if one of the following occurs:
    ///   - `AppAlreadyExists`: If an application with the given `name` (and optionally, `app` parameters) is already registered.
    ///   - `AppInitError`: If the initialization function (`init`) fails.
    ///   - `CannotAddNewPeriodicApp`: If the application could not be registered in the scheduler due to capacity limits or internal errors.
    ///
    /// # Errors
    /// - `KernelError::AppAlreadyExists(name)`: Triggered if an application with the same name already exists in the scheduler.
    /// - `KernelError::AppInitError(name)`: Triggered if the provided initialization function fails to execute successfully.
    /// - `KernelError::CannotAddNewPeriodicApp(name)`: Triggered if the scheduler is unable to add this periodic application.
    ///
    /// # Notes
    /// - The `app_exists` method is used internally to check for duplicate application names.
    /// - Before scheduling, the function converts both the `period` and `sched_period` to `u32` units for computation.
    /// - The function calculates the execution periods relative to the scheduler's configured period (`sched_period`) using integer division.
    ///
    pub fn add_periodic_app(
        &mut self,
        name: &'static str,
        app: AppCall,
        init: Option<App>,
        period: Milliseconds,
        ends_in: Option<Milliseconds>,
    ) -> KernelResult<u32> {
        // Check if the app already exists
        if (match app {
            AppCall::AppNoParam(_, _) => self.app_exists(name, None),
            AppCall::AppParam(_, p, _) => self.app_exists(name, Some(p)),
        })
        .is_some()
        {
            return Err(KernelError::AppAlreadyExists(name));
        }

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
            .map_err(|_| CannotAddNewPeriodicApp(name))?;

        // Increment app ID
        self.next_id += 1;

        // Return ID
        Ok(self.next_id)
    }

    /// Removes a periodic application from the task list.
    ///
    /// This function searches for a task by its name and an optional parameter.
    /// If the task exists, it is removed from the internal task list. Otherwise,
    /// an error is returned indicating that the application was not found.
    ///
    /// # Parameters
    /// - `name`: A static string slice that specifies the name of the application
    ///           to be removed.
    /// - `param`: An optional parameter of type `u32` that can be used to
    ///            refine the search for the application.
    ///
    /// # Returns
    /// - `Ok(())`: If the application was successfully removed.
    /// - `Err(KernelError::AppNotFound)`: If no application with the specified
    ///                                    name (and parameter, if provided) exists.
    ///
    /// # Errors
    /// This function returns a `KernelError::AppNotFound` error if the application
    /// to be removed is not found in the task list.
    ///
    /// # Behavior
    /// - The `tasks` list is modified in-place, using the `swap_remove` method
    ///   which removes the item at the specified index by swapping it with the
    ///   last element and then removing it.
    /// - If the task does not exist, no changes are made to the list.
    pub fn remove_periodic_app(
        &mut self,
        name: &'static str,
        param: Option<u32>,
    ) -> KernelResult<()> {
        if let Some(index) = self.app_exists(name, param) {
            self.tasks.swap_remove(index);
            Ok(())
        } else {
            Err(KernelError::AppNotFound(name))
        }
    }

    /// Executes and manages periodic tasks within the system.
    ///
    /// This function iterates through all registered tasks and performs the following:
    ///
    /// 1. **Task Execution**:
    ///    - Executes tasks whose cycle timing (`app_period`) matches the current `cycle_counter`.
    ///    - Supports tasks with or without parameters (`AppNoParam` or `AppParam`).
    ///    - Handles errors during task execution using the system's error handler, making sure not to invoke
    ///      the error handler multiple times for the same task run.
    ///
    /// 2. **Task Lifetime Management**:
    ///    - Checks if a task is configured to terminate (`ends_in`).
    ///    - If a task's lifetime expires:
    ///      - It is added to the removal queue (`tasks_to_remove`).
    ///      - Executes any final associated closure (`closure`) for tasks with parameters, if present.
    ///
    /// 3. **Task Removal**:
    ///    - Removes tasks from the system based on the removal queue.
    ///    - Utilizes `remove_periodic_app` to safely remove tasks that have ended.
    ///
    /// 4. **Cycle Counter Update**:
    ///    - Increments the `cycle_counter` to track the progression of periodic task executions.
    ///
    /// ### Example Workflow:
    /// - A task is checked for execution based on the current cycle.
    /// - If it matches timing criteria and is active, it is executed.
    /// - If the task has a defined lifetime (`ends_in`), its countdown is reduced until it expires.
    /// - Expired tasks are then removed from the system.
    ///
    /// ### Key Implementation Details:
    /// - **Task Execution Modes**:
    ///   - `AppNoParam`: Tasks without parameters are executed directly.
    ///   - `AppParam`: Tasks with parameters are executed with the provided parameter and can optionally trigger a closure.
    /// - **Error Handling**:
    ///   - Uses `Kernel::errors().error_handler(&e)` to handle runtime errors encountered during task execution.
    ///
    /// ### Panics:
    /// - Panics if the removal of terminated tasks (`tasks_to_remove`) fails unexpectedly, as the `unwrap` method is used during this process.
    ///
    /// ### Fields Used:
    /// - `self.tasks`: A mutable list of registered tasks.
    /// - `self.cycle_counter`: Tracks the current execution cycle.
    /// - `self.current_task_id`: Temporarily stores the ID of the currently executing task.
    /// - `self.current_task_has_error`: Flags if a task encountered errors during execution.
    ///
    /// ### Assumptions:
    /// - The task system uses `AppCall` enum to classify tasks (`AppNoParam`, `AppParam`).
    /// - Each task has an associated name, period, active status, and optional termination (`ends_in`).
    ///
    /// ### Dependencies:
    /// - `Kernel::errors()` for accessing the global error handler.
    /// - `remove_periodic_app` for cleaning up terminated tasks.
    pub fn periodic_task(&mut self) {
        let mut tasks_to_remove: Vec<(&'static str, Option<u32>), 8> = Vec::new();

        // Run all tasks
        for (id, task) in self.tasks.iter_mut().enumerate() {
            if self.cycle_counter % task.app_period == 0 && task.active {
                // Execute the task
                self.current_task_id = Some(id);
                self.current_task_has_error = false;
                match task.app {
                    AppCall::AppNoParam(app, _) => match app() {
                        Ok(..) => {}
                        Err(e) => {
                            if !self.current_task_has_error {
                                Kernel::errors().error_handler(&e);
                            }
                        }
                    },
                    AppCall::AppParam(app, param, _) => match app(param) {
                        Ok(..) => {}
                        Err(e) => {
                            if !self.current_task_has_error {
                                Kernel::errors().error_handler(&e);
                            }
                        }
                    },
                }
                self.current_task_has_error = false;
                self.current_task_id = None;

                // Check if the task has ended
                if task.ends_in.is_some() {
                    task.ends_in = task.ends_in.map(|e| e - 1);
                    if task.ends_in.unwrap() == 0 {
                        let closure_to_apply;

                        match task.app {
                            AppCall::AppNoParam(_, closure) => {
                                tasks_to_remove.push((task.name, None)).unwrap();
                                closure_to_apply = closure;
                            }
                            AppCall::AppParam(_, p, closure) => {
                                tasks_to_remove.push((task.name, Some(p))).unwrap();
                                closure_to_apply = closure;
                            }
                        }

                        // Apply closure
                        if let Some(c) = closure_to_apply {
                            match c() {
                                Ok(..) => {}
                                Err(e) => {
                                    if !self.current_task_has_error {
                                        Kernel::errors().error_handler(&e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Remove tasks that have ended
        for (task_name, param) in tasks_to_remove {
            self.remove_periodic_app(task_name, param).unwrap();
        }

        // Increment cycle counter
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

    /// Checks if an application with the given name and an optional parameter exists within the task list.
    ///
    /// This function iterates through the internal list of tasks and checks if a task with the specified
    /// `name` exists. If a `param` is passed, the function further checks if the optional parameter of the
    /// task matches the provided value. If either of these conditions is met, the index of the matching
    /// task is returned; otherwise, the function returns `None`.
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice representing the name of the application to search for.
    /// * `param` - An `Option<u32>` representing the optional parameter to match against.
    ///             If `None` is provided, the function ignores parameter-based matching.
    ///
    /// # Returns
    ///
    /// * `Some(usize)` - The index of the first task in the list that matches the given name and optional parameter.
    /// * `None` - If no such application is found.
    ///
    /// # Behavior
    ///
    /// * If the task's name matches but the specific task has no associated parameter (i.e., the `AppCall` variant is not `AppParam`),
    ///   the function will return the index of that task.
    /// * If a `param` is provided, both the name and parameter must match for the index to be returned.
    /// * If no `param` is provided, only the name needs to match for the index to be returned.
    ///
    pub fn app_exists(&self, name: &str, param: Option<u32>) -> Option<usize> {
        for (index, task) in self.tasks.iter().enumerate() {
            if task.name == name {
                if let AppCall::AppParam(_, app_param, _) = task.app {
                    if let Some(p) = param {
                        if p == app_param {
                            return Some(index);
                        }
                    } else {
                        return Some(index);
                    }
                } else {
                    return Some(index);
                }
            }
        }
        None
    }

    /// Updates the duration for a task specified by its name and optional parameter.
    ///
    /// This function modifies the `ends_in` field of a task, recalculating its
    /// value based on the provided duration (`time`), the scheduler period, and
    /// the task's application period. If the specified task is not found, an error
    /// is returned.
    ///
    /// # Parameters
    /// - `name`: A static string slice representing the name of the task to update.
    /// - `param`: An optional 32-bit unsigned integer parameter to further identify
    ///   the task. This can differentiate between tasks with the same name.
    /// - `time`: A `Milliseconds` instance representing the new duration for the
    ///   task, usually measured in milliseconds.
    ///
    /// # Returns
    /// - `Ok(())`: If the task's duration was successfully updated.
    /// - `Err(KernelError::AppNotFound)`: If no task matching the specified `name`
    ///   and `param` is found.
    ///
    /// # Panics
    /// This function does not explicitly panic under normal conditions. However,
    /// unexpected panics could arise from underlying operations or misconfigured
    /// scheduler values.
    ///
    /// # Note
    /// The `ends_in` value is derived by dividing the given `time` by both the
    /// scheduler's period and the task's application period. Ensure that both
    /// `self.sched_period` and the specified task's `app_period` are non-zero to
    /// prevent division-related errors.
    pub fn set_new_task_duration(
        &mut self,
        name: &'static str,
        param: Option<u32>,
        time: Milliseconds,
    ) -> KernelResult<()> {
        if let Some(index) = self.app_exists(name, param) {
            self.tasks[index].ends_in =
                Some(time.to_u32() / self.sched_period.to_u32() / self.tasks[index].app_period);
            Ok(())
        } else {
            Err(KernelError::AppNotFound(name))
        }
    }
}
