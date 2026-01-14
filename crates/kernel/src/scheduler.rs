use crate::KernelError::CannotAddNewPeriodicApp;
use crate::console_output::ConsoleFormatting;
use crate::data::Kernel;
use crate::systick::set_ticks_target;
use crate::{KernelError, KernelResult, Milliseconds};
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

/// Represents the different ways an application can be called by the scheduler.
///
/// This enum allows the scheduler to handle both parameterless applications and
/// applications that require a runtime parameter. This is useful for scenarios where
/// multiple instances of the same application logic need to operate on different
/// resources (e.g., blinking different LEDs identified by their HAL interface ID).
///
/// # Variants
///
/// * `AppNoParam(App)` - An application that takes no parameters. The wrapped [`App`]
///   function pointer is called directly without arguments.
///
/// * `AppParam(AppParam, u32)` - An application that requires a `u32` parameter. The
///   wrapped [`AppParam`] function pointer is called with the stored `u32` value.
///   This variant enables a single function to be scheduled multiple times with
///   different parameters, allowing for resource-specific behavior.
pub enum AppCall {
    AppNoParam(App),
    AppParam(AppParam, u32),
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
/// * `app_id` (`u32`) -
///   A unique identifier for the application within the system. This ID is used for
///   tracking and managing the application's lifecycle and interactions with other
///   components of the system.
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
    app_closure: Option<App>,
    app_period: u32,
    ends_in: Option<u32>,
    active: bool,
    app_id: u32,
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
    tasks: Vec<AppWrapper, 32>,
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
        Kernel::terminal().write(&ConsoleFormatting::StrNewLineBoth("Scheduler started !"))
    }

    /// Registers a new periodic application with the scheduler.
    ///
    /// This method adds an application to the scheduler's task list, configuring it to run
    /// at a specified interval. The application can optionally have initialization and
    /// cleanup callbacks, as well as a finite lifetime.
    ///
    /// # Parameters
    ///
    /// * `name` - A static string identifier for the application. Must be unique within
    ///   the scheduler (combined with the parameter value for `AppParam` variants).
    ///
    /// * `app` - The application entry point, either [`AppCall::AppNoParam`] for
    ///   parameterless apps or [`AppCall::AppParam`] for apps requiring a `u32` argument.
    ///
    /// * `app_init` - Optional initialization function called once before the first
    ///   execution of the application. If initialization fails, the app is skipped
    ///   until the next cycle and init is retried.
    ///
    /// * `app_closure` - Optional cleanup function called when the application's lifetime
    ///   expires (i.e., when `ends_in` reaches zero). Useful for releasing resources.
    ///
    /// * `period` - The interval between consecutive executions of the application,
    ///   expressed in milliseconds. Internally converted to scheduler cycles.
    ///
    /// * `ends_in` - Optional finite lifetime for the application. When specified, the
    ///   application will be automatically removed after this duration elapses.
    ///   If `None`, the application runs indefinitely until explicitly removed.
    ///
    /// # Returns
    ///
    /// * `Ok(u32)` - The unique identifier assigned to the newly registered application.
    ///   This ID can be used for tracking or removing the application later.
    ///
    /// * `Err(KernelError::AppAlreadyScheduled)` - If an application with the same name
    ///   (and parameter, if applicable) is already registered.
    ///
    /// * `Err(KernelError::CannotAddNewPeriodicApp)` - If the task list is full and
    ///   cannot accommodate additional applications.
    pub fn add_periodic_app(
        &mut self,
        name: &'static str,
        app: AppCall,
        app_init: Option<App>,
        app_closure: Option<App>,
        period: Milliseconds,
        ends_in: Option<Milliseconds>,
    ) -> KernelResult<u32> {
        // Check if the app already exists
        if (match app {
            AppCall::AppNoParam(_) => self.app_exists(name, None),
            AppCall::AppParam(_, p) => self.app_exists(name, Some(p)),
        })
        .is_some()
        {
            return Err(KernelError::AppAlreadyScheduled(name));
        }

        // Increment app ID
        self.next_id += 1;

        // Register app in the scheduler
        self.tasks
            .push(AppWrapper {
                name,
                app,
                app_init,
                app_closure,
                app_period: period.to_u32() / self.sched_period.to_u32(),
                active: true,
                ends_in: ends_in.map(|e| e.to_u32() / period.to_u32()),
                app_id: self.next_id,
            })
            .map_err(|_| CannotAddNewPeriodicApp(name))?;

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
            Kernel::apps().stop_app(self.tasks[index].app_id)?;
            self.tasks.swap_remove(index);
            Ok(())
        } else {
            Err(KernelError::AppNotScheduled(name))
        }
    }

    /// Executes all due periodic tasks for the current scheduler cycle.
    ///
    /// This method is the core scheduling loop, typically invoked from the PendSV interrupt
    /// handler. It iterates through all registered tasks and executes those whose period
    /// aligns with the current cycle counter.
    ///
    /// # Behavior
    ///
    /// For each active task whose execution period has elapsed:
    ///
    /// 1. **Initialization**: If the task has a pending `app_init` function, it is called
    ///    first. On success, the init function is cleared (runs only once). On failure,
    ///    the error is handled and the task is skipped for this cycle.
    ///
    /// 2. **Execution**: The main application function is invoked (with or without
    ///    parameter depending on the [`AppCall`] variant). Errors are routed through
    ///    the kernel error handler unless an error was already flagged for this task.
    ///
    /// 3. **Lifetime management**: If the task has a finite lifetime (`ends_in`), the
    ///    remaining count is decremented. When it reaches zero:
    ///    - The `app_closure` callback is invoked (if configured) for cleanup.
    ///    - The task is marked for removal.
    ///
    /// 4. **Cleanup**: All tasks marked for removal are unregistered from the scheduler.
    ///
    /// 5. **Cycle increment**: The global cycle counter is incremented.
    ///
    /// # Error handling
    ///
    /// Errors during task execution are passed to [`Kernel::errors().error_handler()`].
    /// The `current_task_has_error` flag prevents duplicate error handling if the error
    /// handler itself triggers additional errors for the same task.
    ///
    /// # Panics
    ///
    /// May panic if the internal `tasks_to_remove` buffer overflows (more than 8 tasks
    /// ending in a single cycle) or if `remove_periodic_app` fails unexpectedly.
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
                self.current_task_has_error = false;
                self.current_task_id = None;

                // Check if the task has ended
                if task.ends_in.is_some() {
                    task.ends_in = task.ends_in.map(|e| e - 1);
                    if task.ends_in.unwrap() == 0 {
                        match task.app {
                            AppCall::AppNoParam(_) => {
                                tasks_to_remove.push((task.name, None)).unwrap();
                            }
                            AppCall::AppParam(_, p) => {
                                tasks_to_remove.push((task.name, Some(p))).unwrap();
                            }
                        };

                        // Apply closure
                        if let Some(c) = task.app_closure {
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
                if let AppCall::AppParam(_, app_param) = task.app {
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
            Err(KernelError::AppNotScheduled(name))
        }
    }

    /// Returns the scheduling period of the current object.
    ///
    /// This method retrieves the value of `sched_period`, which represents
    /// the time interval (in milliseconds) associated with this object. The
    /// scheduling period could reflect a timing configuration or be used
    /// to specify execution intervals for scheduling tasks.
    ///
    /// # Returns
    /// A `Milliseconds` value representing the scheduling period.
    ///
    pub fn get_period(&self) -> Milliseconds {
        self.sched_period
    }
}
