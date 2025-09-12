use crate::data::Kernel;
use crate::errors_mgt::error_handler;
use crate::{KernelError, KernelResult};
use hal_interface::{InterfaceReadActions, InterfaceWriteActions};

pub struct SysCallHalArgs<'a> {
    pub id: usize,
    pub write_action: Option<InterfaceWriteActions<'a>>,
    pub read_action: Option<InterfaceReadActions<'a>>,
}

pub enum Syscall<'a> {
    Hal(SysCallHalArgs<'a>),
    HalGetId(&'static str, &'a mut usize),
}

pub fn syscall(syscall_type: Syscall) -> KernelResult<()> {
    let result = match syscall_type {
        Syscall::Hal(args) => {
            if let Some(write_action) = args.write_action {
                if args.read_action.is_none() {
                    Kernel::hal()
                        .interface_write(args.id, write_action)
                        .map_err(KernelError::HalError)
                } else {
                    Err(KernelError::WrongSyscallArgs(
                        "cannot call for HAL write and read in the same time",
                    ))
                }
            } else if let Some(read_action) = args.read_action {
                Kernel::hal()
                    .interface_read(args.id, read_action)
                    .map_err(KernelError::HalError)
            } else {
                Err(KernelError::WrongSyscallArgs(
                    "HAL call should have at least one action",
                ))
            }
        }
        Syscall::HalGetId(name, id) => match Kernel::hal().get_interface_id(name) {
            Ok(hal_id) => {
                *id = hal_id;
                Ok(())
            }
            Err(e) => Err(KernelError::HalError(e)),
        },
    };

    match result {
        Ok(..) => Ok(()),
        Err(err) => {
            error_handler(&err);
            Err(err)
        }
    }
}
