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

    /// Adds a periodic application to the scheduler.
    ///
    /// This method registers a new application that will be executed periodically by the system.
    /// The application can optionally run an initialization function and have a specific lifetime.
    ///
    /// # Parameters
    /// - `name`: A static string slice representing the name of the app. This name must be unique.
    /// - `app`: The function or closure that will be executed periodically. It can optionally take a parameter.
    /// - `init`: An optional initialization function for the app. This is useful for setting up app-specific resources.
    /// - `period`: The period in milliseconds between consecutive executions of the app.
    /// - `ends_in`: An optional duration (in milliseconds) after which the app will stop executing. If `None`, the app will not have a fixed lifetime.
    ///
    /// # Return
    /// Returns a [`KernelResult<()>`] which is:
    /// - `Ok(())` on successful registration of the app.
    /// - `Err(KernelError)` if the registration failed:
    ///   - [`KernelError::AppAlreadyExists`]: If an app with the same `name` and parameters already exists.
    ///   - [`KernelError::AppInitError`]: If the provided initialization function fails.
    ///   - [`CannotAddNewPeriodicApp`]: If the app cannot be pushed into the internal scheduler (e.g., due to capacity issues).
    ///
    /// # Errors
    /// - **AppAlreadyExists:** If an application with the same name and parameters is already registered.
    /// - **AppInitError:** If the initialization function for the app fails.
    /// - **CannotAddNewPeriodicApp:** If the task cannot be added to the scheduler's task list.
    ///
    /// # Behavior
    /// - If `init` is provided, it is executed before the app is registered. A failure in the initialization will result in the app not being added.
    /// - The periodicity of the app execution is decided based on the provided `period` and the system's scheduling period (`sched_period`).
    /// - If `ends_in` is provided, the app will automatically deactivate once the duration elapses.
    /// - All apps are set to `active` by default upon registration.
    ///
    /// # Notes
    /// - This function requires that the `name` parameter is unique for each app.
    /// - The `period` must be compatible with the system's scheduler. Ensure it is an integer multiple or factor of the `sched_period` for predictable behavior.
    /// - The scheduler holds only a finite number of apps. Adding beyond its capacity will result in an error.
    pub fn add_periodic_app(
        &mut self,
        name: &'static str,
        app: AppCall,
        init: Option<App>,
        period: Milliseconds,
        ends_in: Option<Milliseconds>,
    ) -> KernelResult<()> {
        // Check if the app already exists
        if let Some(_) = match app {
            AppCall::AppNoParam(_) => self.app_exists(name, None),
            AppCall::AppParam(_, p) => self.app_exists(name, Some(p)),
        } {
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
            .map_err(|_| CannotAddNewPeriodicApp(name))
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

    /// Executes periodic tasks based on their defined execution intervals and manages task lifecycle.
    ///
    /// This function iterates through all registered tasks, executing those that are due,
    /// and handles their termination if their lifetime (`ends_in`) is completed. The task's
    /// execution is determined by the current cycle count and the task's application period.
    /// Errors encountered during task execution are handled via the error handler system.
    ///
    /// # Functionality
    /// - Evaluates and executes tasks whose periodicity matches the current cycle count.
    /// - Handles tasks with or without parameters depending on the type of `AppCall`.
    /// - Tracks and manages errors occurring in task execution, ensuring appropriate error handling.
    /// - Terminates tasks when their predefined duration (`ends_in`) has elapsed.
    /// - Removes completed tasks and increments the cycle count.
    ///
    /// # Task Execution
    /// Each task is represented by an `AppCall`, which can either:
    /// - `AppNoParam`: A function with no parameters.
    /// - `AppParam`: A function with associated parameters.
    ///
    /// The task is executed periodically based on the `app_period` and only if it is marked as `active`.
    /// Upon execution:
    /// - If the task executes successfully, it continues its lifecycle.
    /// - If an error occurs, it is passed to the system's error handler.
    ///
    /// # Task Termination
    /// If a task has a defined `ends_in` value:
    /// - The value decrements after each execution.
    /// - Once the value reaches 0, the task is added to a removal list to be cleaned up at the end
    ///   of the cycle.
    ///
    /// # Error Handling
    /// Task execution errors are detected, and if the task has not already logged an error in
    /// the current execution cycle, they are passed to the system-wide error handler.
    ///
    /// # Parameters
    /// - `self` (`&mut Self`): A mutable reference to the structure containing the periodic task system.
    ///
    /// # Behavior
    /// 1. Iterates over the task list and executes eligible tasks based on their periodicity and state.
    /// 2. Manages task error detection and invokes the system's error handler when necessary.
    /// 3. Checks tasks with defined lifetimes and schedules their removal when their time expires.
    /// 4. Incrementally updates the cycle counter for the task scheduler.
    ///
    /// # Removes Tasks
    /// Tasks that are completed (based on their `ends_in` value reaching 0) are removed from
    /// the system using the `remove_periodic_app` method.
    ///
    /// # Panics
    /// The function assumes the task removal and addition to the `tasks_to_remove` vector will not
    /// exceed the maximum capacity of 8 elements; if this assumption is violated, it will panic.
    ///
    pub fn periodic_task(&mut self) {
        let mut tasks_to_remove: Vec<(&'static str, Option<u32>), 8> = Vec::new();

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
                        match task.app {
                            AppCall::AppNoParam(_) => {
                                tasks_to_remove.push((task.name, None)).unwrap()
                            }
                            AppCall::AppParam(_, p) => {
                                tasks_to_remove.push((task.name, Some(p))).unwrap()
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
