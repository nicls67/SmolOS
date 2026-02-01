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
/// * `app` (`App`) -
///   Represents the core application logic or callable function associated with the application.
///   This is the primary entry point for executing application-specific logic.
///
/// * `app_closure` (`Option<App>`) -
///   Optional cleanup function called when the application's lifetime expires.
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
/// * `managed_by_apps` (`bool`) -
///   A flag indicating whether the application is managed by the `AppsManager`.
///   If true, cleanup is handled by the `AppsManager`; otherwise, it's handled internally.
///
/// # Usage
///
/// The `AppWrapper` structure is used to manage the state and metadata of applications
/// in environments where dynamic application handling is required. It keeps track of
///  the application lifecycle and provides mechanisms to control application execution.
///
struct AppWrapper {
    name: &'static str,
    app: App,
    app_closure: Option<App>,
    app_period: u32,
    ends_in: Option<u32>,
    active: bool,
    app_id: u32,
    managed_by_apps: bool,
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
///   Limited to a size of 32.
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
    pub fn new(p_period: Milliseconds) -> Scheduler {
        Scheduler {
            tasks: Vec::new(),
            cycle_counter: 0,
            sched_period: p_period,
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
    pub fn start(&mut self, p_systick_period: Milliseconds) -> KernelResult<()> {
        let l_cortex_p = Kernel::cortex_peripherals();

        // Initialize scheduler periodic IT
        unsafe {
            l_cortex_p.SCB.set_priority(SystemHandler::PendSV, 0xFF);
            set_ticks_target(self.sched_period.to_u32() / p_systick_period.to_u32())
        }

        self.started = true;
        Kernel::terminal().write(&ConsoleFormatting::StrNewLineBoth("Scheduler started !"))
    }

    /// Registers a new periodic application with the scheduler.
    ///
    /// This method adds an application to the scheduler's task list, configuring it to run
    /// at a specified interval. The application can optionally have a cleanup callback,
    /// as well as a finite lifetime.
    ///
    /// # Parameters
    ///
    /// * `name` - A static string identifier for the application. Must be unique within
    ///   the scheduler.
    ///
    /// * `app` - The application entry point.
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
    ///   is already registered.
    ///
    /// * `Err(KernelError::CannotAddNewPeriodicApp)` - If the task list is full and
    ///   cannot accommodate additional applications.
    pub fn add_periodic_app(
        &mut self,
        p_name: &'static str,
        p_app: App,
        p_app_closure: Option<App>,
        p_period: Milliseconds,
        p_ends_in: Option<Milliseconds>,
        p_managed_by_apps: bool,
    ) -> KernelResult<u32> {
        // Check if the app already exists
        if (self.app_exists(p_name)).is_some() {
            return Err(KernelError::AppAlreadyScheduled(p_name));
        }

        // Increment app ID
        self.next_id += 1;

        // Register app in the scheduler
        self.tasks
            .push(AppWrapper {
                name: p_name,
                app: p_app,
                app_closure: p_app_closure,
                app_period: p_period.to_u32() / self.sched_period.to_u32(),
                active: true,
                ends_in: p_ends_in.map(|l_e| l_e.to_u32() / p_period.to_u32()),
                app_id: self.next_id,
                managed_by_apps: p_managed_by_apps,
            })
            .map_err(|_| CannotAddNewPeriodicApp(p_name))?;

        // Return ID
        Ok(self.next_id)
    }

    /// Removes a periodic application from the task list.
    ///
    /// This function searches for a task by its name. If the task exists, it is removed
    /// from the internal task list. Otherwise,
    /// an error is returned indicating that the application was not found.
    ///
    /// # Parameters
    /// - `name`: A static string slice that specifies the name of the application
    ///   to be removed.
    /// # Returns
    /// - `Ok(())`: If the application was successfully removed.
    /// - `Err(KernelError::AppNotScheduled)`: If no application with the specified
    ///   name exists.
    ///
    /// # Errors
    /// This function returns a `KernelError::AppNotScheduled` error if the application
    /// to be removed is not found in the task list.
    ///
    /// # Behavior
    /// - The `tasks` list is modified in-place, using the `swap_remove` method
    ///   which removes the item at the specified index by swapping it with the
    ///   last element and then removing it.
    /// - If the task does not exist, no changes are made to the list.
    pub fn remove_periodic_app(&mut self, p_name: &'static str) -> KernelResult<()> {
        if let Some(l_index) = self.app_exists(p_name) {
            self.tasks.swap_remove(l_index);
            Ok(())
        } else {
            Err(KernelError::AppNotScheduled(p_name))
        }
    }

    /// Removes a periodic application from the task list using its unique ID.
    ///
    /// This function searches for a task by its ID. If the task exists, it is removed
    /// from the internal task list. Otherwise, an error is returned.
    ///
    /// # Parameters
    /// - `app_id`: The unique identifier of the application to be removed.
    /// # Returns
    /// - `Ok(())`: If the application was successfully removed.
    /// - `Err(KernelError::AppNotFound)`: If no application with the specified ID exists.
    pub fn remove_periodic_app_by_id(&mut self, p_app_id: u32) -> KernelResult<()> {
        if let Some(l_index) = self
            .tasks
            .iter()
            .position(|l_task| l_task.app_id == p_app_id)
        {
            self.tasks.swap_remove(l_index);
            Ok(())
        } else {
            Err(KernelError::AppNotFound)
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
    /// 1. **Execution**: The main application function is invoked. Errors are routed through
    ///    the kernel error handler unless an error was already flagged for this task.
    ///
    /// 2. **Lifetime management**: If the task has a finite lifetime (`ends_in`), the
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
    /// ending in a single cycle) or if `Kernel::apps().stop_app` fails unexpectedly.
    pub fn periodic_task(&mut self) {
        let mut l_tasks_to_remove: Vec<u32, 8> = Vec::new();

        // Run all tasks
        for (l_id, l_task) in self.tasks.iter_mut().enumerate() {
            if self.cycle_counter.is_multiple_of(l_task.app_period) && l_task.active {
                self.current_task_id = Some(l_id);
                self.current_task_has_error = false;

                // Execute the task
                match (l_task.app)() {
                    Ok(..) => {}
                    Err(l_e) => {
                        if !self.current_task_has_error {
                            Kernel::errors().error_handler(&l_e);
                        }
                    }
                }
                self.current_task_has_error = false;
                self.current_task_id = None;

                // Check if the task has ended
                if l_task.ends_in.is_some() {
                    l_task.ends_in = l_task.ends_in.map(|l_e| l_e - 1);
                    if l_task.ends_in.unwrap() == 0 {
                        l_tasks_to_remove.push(l_task.app_id).unwrap();

                        // Apply closure only for internal tasks
                        // (managed apps handle it in their stop() logic)
                        if !l_task.managed_by_apps {
                            if let Some(l_c) = l_task.app_closure {
                                match l_c() {
                                    Ok(..) => {}
                                    Err(l_e) => {
                                        if !self.current_task_has_error {
                                            Kernel::errors().error_handler(&l_e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Remove tasks that have ended
        for l_task_id in l_tasks_to_remove {
            match Kernel::apps().stop_app(l_task_id) {
                Ok(()) => {}
                Err(KernelError::AppNotFound) => {
                    // Internal task, remove it directly from scheduler
                    self.remove_periodic_app_by_id(l_task_id).unwrap();
                }
                Err(l_e) => {
                    if !self.current_task_has_error {
                        Kernel::errors().error_handler(&l_e);
                    }
                }
            }
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
    /// handle tasks that encounter a hardware exception or a runtime error.
    pub fn abort_task_on_error(&mut self) {
        if SCB::vect_active() == VectActive::Exception(Exception::PendSV) {
            // Set the current task as inactive
            if let Some(l_id) = self.current_task_id {
                self.tasks[l_id].active = false;
                self.current_task_has_error = true;
            }
        }
    }

    /// Checks if an application with the given name exists within the task list.
    ///
    /// This function iterates through the internal list of tasks and checks if a task with the specified
    /// `name` exists. If a matching task is found, the index is returned; otherwise, the function returns `None`.
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice representing the name of the application to search for.
    /// # Returns
    ///
    /// * `Some(usize)` - The index of the first task in the list that matches the given name.
    /// * `None` - If no such application is found.
    ///
    /// # Behavior
    ///
    /// * If the task's name matches, the function returns the index of that task.
    pub fn app_exists(&self, p_name: &str) -> Option<usize> {
        for (l_index, l_task) in self.tasks.iter().enumerate() {
            if l_task.name == p_name {
                return Some(l_index);
            }
        }
        None
    }

    /// Updates the duration for a task specified by its name.
    ///
    /// This function modifies the `ends_in` field of a task, recalculating its
    /// value based on the provided duration (`time`), the scheduler period, and
    /// the task's application period. If the specified task is not found, an error
    /// is returned.
    ///
    /// # Parameters
    /// - `name`: A static string slice representing the name of the task to update.
    /// - `time`: A `Milliseconds` instance representing the new duration for the
    ///   task, usually measured in milliseconds.
    ///
    /// # Returns
    /// - `Ok(())`: If the task's duration was successfully updated.
    /// - `Err(KernelError::AppNotFound)`: If no task matching the specified `name`
    ///   is found.
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
        p_name: &'static str,
        p_time: Milliseconds,
    ) -> KernelResult<()> {
        if let Some(l_index) = self.app_exists(p_name) {
            self.tasks[l_index].ends_in =
                Some(p_time.to_u32() / self.sched_period.to_u32() / self.tasks[l_index].app_period);
            Ok(())
        } else {
            Err(KernelError::AppNotScheduled(p_name))
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
