use crate::data::Kernel;
use crate::scheduler::{App, AppCall};
use crate::{KernelError, KernelResult, Milliseconds};
use hal_interface::InterfaceWriteActions;

pub struct SysCallHalArgs<'a> {
    pub id: usize,
    pub action: InterfaceWriteActions<'a>,
}

pub enum Syscall<'a> {
    Hal(SysCallHalArgs<'a>),
    HalGetId(&'static str, &'a mut usize),
    AddPeriodicTask(
        &'static str,
        AppCall,
        Option<App>,
        Milliseconds,
        Option<Milliseconds>,
    ),
    RemovePeriodicTask(&'static str, Option<u32>),
    NewTaskDuration(&'static str, Option<u32>, Milliseconds),
}

/**
 * Executes various system calls based on the provided `Syscall` enum variant.
 * This function acts as the central interface for managing kernel operations
 * such as hardware interactions, periodic task scheduling, and modifying task properties.
 *
 * # Arguments
 *
 * * `syscall_type` - A variant of the `Syscall` enum that specifies the type of system call to execute.
 *
 * # Returns
 *
 * * `Ok(())` - If the requested system call is successful.
 * * `Err(KernelError)` - If an error occurs during the execution of the system call, which is then handled
 *    by the kernel's error handler.
 *
 * # Syscall Variants
 *
 * ## `Syscall::Hal(args)`
 * Calls a hardware abstraction layer (HAL) interface function. Internally invokes
 * `Kernel::hal().interface_action()` with the provided arguments:
 *
 * - `args.id` - Identifier of the hardware interface.
 * - `args.action` - The action to perform on the specified hardware interface.
 *
 * ## `Syscall::HalGetId(name, id)`
 * Retrieves the unique identifier for a hardware interface.
 *
 * - `name` - The name of the hardware interface.
 * - `id` - A mutable reference to store the resulting HAL ID. This is populated on success.
 *
 * ## `Syscall::AddPeriodicTask(name, app, init, period, ends_in)`
 * Registers a new periodic task in the kernel's scheduler.
 *
 * - `name` - The unique name of the task.
 * - `app` - The application or task logic to run periodically.
 * - `init` - Initial delay before starting the first execution.
 * - `period` - The duration between consecutive executions.
 * - `ends_in` - The optional end time for the periodic task.
 *
 * ## `Syscall::RemovePeriodicTask(name, param)`
 * Removes a periodic task from the scheduler.
 *
 * - `name` - The name of the task to be removed.
 * - `param` - Additional parameters for task removal.
 *
 * ## `Syscall::NewTaskDuration(name, param, time)`
 * Updates the execution duration or timing details of an existing task.
 *
 * - `name` - The name of the task to update.
 * - `param` - Additional parameters related to the task.
 * - `time` - The new duration or timing configuration for the task.
 *
 * # Error Handling
 *
 * In the case of an error during the execution of the system call, the kernel error
 * handler processes the error by invoking `Kernel::errors().error_handler()`. The error
 * is then returned as a `KernelError`.
 *
 */
pub fn syscall(syscall_type: Syscall) -> KernelResult<()> {
    let result = match syscall_type {
        Syscall::Hal(args) => Kernel::hal()
            .interface_write(args.id, args.action)
            .map_err(KernelError::HalError),
        Syscall::HalGetId(name, id) => match Kernel::hal().get_interface_id(name) {
            Ok(hal_id) => {
                *id = hal_id;
                Ok(())
            }
            Err(e) => Err(KernelError::HalError(e)),
        },
        Syscall::AddPeriodicTask(name, app, init, period, ends_in) => {
            Kernel::scheduler().add_periodic_app(name, app, init, period, ends_in)
        }
        Syscall::RemovePeriodicTask(name, param) => {
            Kernel::scheduler().remove_periodic_app(name, param)
        }
        Syscall::NewTaskDuration(name, param, time) => {
            Kernel::scheduler().set_new_task_duration(name, param, time)
        }
    };

    match result {
        Ok(..) => Ok(()),
        Err(err) => {
            Kernel::errors().error_handler(&err);
            Err(err)
        }
    }
}
