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

/// `AppWrapper` is a structure that encapsulates metadata and state for an application
/// or service within a system. It provides details such as the application name,
/// its initialization state, runtime period, lifecycle, and active status.
///
/// # Fields
///
/// * `name` (`&'static str`) -
///   The static name identifier for the application. This name remains constant
///   throughout the lifecycle of the application.
///
/// * `app` (`AppCall`) -
///   Represents the core application logic or callable function associated with the application.
///   This is the primary entry point for executing application-specific logic.
///
/// * `app_init` (`Option<App>`) -
///   Optional initialization structure or state for the application. It may hold
///   configuration or pre-instantiation data necessary for the application startup.
///
/// * `app_period` (`u32`) -
///   Specifies the periodic interval or runtime duration for the application's operations,
///   typically represented as a time cycle in seconds or milliseconds.
///
/// * `ends_in` (`Option<u32>`) -
///   An optional field indicating the remaining duration until the application finishes
///   its lifecycle or task. A `None` value indicates that the application does not have
///   a designated end time.
///
/// * `active` (`bool`) -
///   A flag indicating the operational status of the application. A value of `true`
///   implies the application is actively running or enabled, while `false` means it is
///   inactive or disabled.
///
/// # Usage
///
/// The `AppWrapper` structure is used to manage the state and metadata of applications
/// in environments where dynamic application handling is required. It keeps track of
///  the application lifecycle and provides mechanisms to control application execution.
///
struct AppWrapper {
    name: &'static str,
    app: AppCall,
    app_init: Option<App>,
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
///   Limited to a size of 128.
/// * `cycle_counter` - A counter representing the number of completed execution cycles.
/// * `sched_period` - The scheduling period, represented in milliseconds, specifying the frequency
///   at which the scheduler cycles through tasks.
/// * `started` - A public boolean indicating whether the scheduler has been started for execution.
/// * `current_task_id` - An optional `usize` representing the index of the currently executing task within the `tasks` vector.
///   If no task is currently active, it is `None`.
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
    /// This function allows for the registration of a periodic application that will
    /// run based on the specified period. It includes optional initialization and a configurable
    /// duration for how long the application will remain active.
    ///
    /// ### Parameters:
    /// - `name`: A static string slice representing the unique name of the application.
    /// - `app`: An instance of `AppCall`, defining the function or process to execute.
    ///   It can either be a callable application without parameters (`AppCall::AppNoParam`)
    ///   or one with parameters (`AppCall::AppParam`).
    /// - `app_init`: An optional initialization routine or state for the application (can be `None`).
    /// - `period`: The desired period (in milliseconds) at which the application should execute.
    /// - `ends_in`: An optional duration (in milliseconds) after which the application will no longer
    ///   execute periodically. If `None`, the application will run indefinitely according to its period.
    ///
    /// ### Returns:
    /// - `Ok(u32)`: Returns the unique ID of the newly added application on a successful addition.
    /// - `Err(KernelError)`: Returns an error if:
    ///   - The application with the same name and parameters already exists
    ///     (`KernelError::AppAlreadyExists`).
    ///   - A new periodic application cannot be added due to internal constraints
    ///     (`KernelError::CannotAddNewPeriodicApp`).
    ///
    /// ### Errors:
    /// - **`KernelError::AppAlreadyExists`**: This error is raised when an application with the
    ///   same name and parameters is already registered in the scheduler.
    /// - **`KernelError::CannotAddNewPeriodicApp`**: This error occurs if the scheduler
    ///   fails to add the new application due to internal capacity or system-level restrictions.
    ///
    /// ### Internal Workflow:
    /// 1. **Duplicate Check**: First, checks whether the application already exists by using
    ///    `self.app_exists()`. This considers both the name and parameters of the application.
    /// 2. **Scheduler Registration**: If the application is unique, registers it in the scheduler's
    ///    task queue with the provided configurations.
    /// 3. **ID Tracking**: Assigns a unique identifier to the new application and increments
    ///    the internal counter `self.next_id`.
    /// 4. **Return Value**: Returns the assigned application ID or an error.
    ///
    /// ### Notes:
    /// - The periodic execution is calculated as fractions of the scheduler's `sched_period`.
    /// - Proper handling of initialization (`app_init`) and termination conditions
    ///   (`app_period` and `ends_in`) should be ensured for accurate functioning.
    /// - This function modifies the internal state of the scheduler; make sure it's used in a
    ///   thread-safe context if applicable.
    ///
    /// ### See Also:
    /// - [`AppCall`] for application execution configurations.
    /// - [`KernelError`] for different error types that may be returned.
    pub fn add_periodic_app(
        &mut self,
        name: &'static str,
        app: AppCall,
        app_init: Option<App>,
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

        // Register app in the scheduler
        self.tasks
            .push(AppWrapper {
                name,
                app,
                app_init,
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
    ///   to be removed.
    /// - `param`: An optional parameter of type `u32` that can be used to
    ///   refine the search for the application.
    ///
    /// # Returns
    /// - `Ok(())`: If the application was successfully removed.
    /// - `Err(KernelError::AppNotFound)`: If no application with the specified
    ///   name (and parameter, if provided) exists.
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

    /// Executes periodic tasks based on their scheduling requirements, manages their lifecycle,
    /// handles errors, and ensures that completed tasks are removed properly.
    ///
    /// This method performs the following actions:
    /// 1. Iterates through the list of tasks and checks if a task should be executed based on the current cycle count and task period.
    /// 2. Initializes the task using an optional initialization function (`app_init`) if it's the first time the task runs.
    /// 3. Executes the main function of the task:
    ///    - This can either be a function with no parameters (`AppNoParam`) or one with parameters (`AppParam`).
    ///    - Any errors encountered during execution are handled by invoking the provided error handler.
    /// 4. Manages the end-of-life scenario for tasks:
    ///    - If a task is configured to end after a specific number of cycles (`ends_in`), it decrements the remaining cycle count.
    ///    - When the count reaches zero, the task is marked for removal and any cleanup closure associated with the task is executed.
    /// 5. Removes tasks that have completed their lifecycle from the task list.
    /// 6. Increments the global cycle counter after processing all tasks.
    ///
    /// # Task Lifecycle
    /// - Tasks can be periodic, running at scheduled intervals defined by their `app_period`.
    /// - Tasks can optionally initialize themselves during their first execution.
    /// - Tasks may have an optional end condition determined by the `ends_in` field. If a task ends,
    ///   a closure may execute to perform custom cleanup logic.
    /// - Tasks that have completed their lifecycle are safely removed from the task list.
    ///
    /// # Error Management
    /// - Errors encountered during task initialization or execution are handled by invoking `Kernel::errors().error_handler(&e)`.
    /// - The system ensures error handling is invoked only once per erroneous execution per task.
    ///
    /// # Panics
    /// This function may panic if:
    /// - The task to be removed cannot be found.
    /// - Adding a task to the `tasks_to_remove` list exceeds its fixed capacity.
    ///
    /// This will run the tasks scheduled for the current cycle, handle any errors, and clean up completed tasks.
    ///
    /// # Note
    /// - This function works with a mutable reference to itself to allow modifications to the task list.
    /// - The capacity of `tasks_to_remove` is fixed at 8. If more than 8 tasks are to be removed in a single cycle,
    ///   the method will panic.
    pub fn periodic_task(&mut self) {
        let mut tasks_to_remove: Vec<(&'static str, Option<u32>), 8> = Vec::new();

        // Run all tasks
        for (id, task) in self.tasks.iter_mut().enumerate() {
            if self.cycle_counter.is_multiple_of(task.app_period) && task.active {
                self.current_task_id = Some(id);
                self.current_task_has_error = false;

                // Try to initialize the app at the first call
                if let Some(init_func) = task.app_init {
                    match init_func() {
                        Ok(..) => task.app_init = None,
                        Err(e) => {
                            Kernel::errors().error_handler(&e);
                            continue;
                        }
                    }
                }

                // Execute the task
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
                        let closure_to_apply = match task.app {
                            AppCall::AppNoParam(_, closure) => {
                                tasks_to_remove.push((task.name, None)).unwrap();
                                closure
                            }
                            AppCall::AppParam(_, p, closure) => {
                                tasks_to_remove.push((task.name, Some(p))).unwrap();
                                closure
                            }
                        };

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
    ///   If `None` is provided, the function ignores parameter-based matching.
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
