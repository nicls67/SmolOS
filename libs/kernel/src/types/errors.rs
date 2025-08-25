pub type KernelResult<T> = Result<T, KernelError>;

pub enum KernelError {
    Fatal(KernelErrorKind),
    Error(KernelErrorKind),
}
pub enum KernelErrorKind {}
