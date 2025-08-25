pub type HalResult<T> = Result<T, HalError>;

pub enum HalError {
    Fatal(HalErrorKind),
    Error(HalErrorKind),
}

pub enum HalErrorKind {}
