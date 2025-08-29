use core::cell::Cell;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use cortex_m::asm::wfi;

/// A function that executes a given asynchronous task (future) to completion
/// within a synchronous context. It blocks the thread on which it is called
/// until the future is finished, returning the completed value of the future.
///
/// # Type Parameters
/// - `F`: A type that implements the `Future` trait.
///
/// # Parameters
/// - `fut`: The future to be executed to completion.
///
/// # Returns
/// The output of the completed future (`F::Output`).
///
/// # Implementation Details
/// This function:
///
/// 1. Utilizes a `Cell<bool>` named `woke` to indicate whether the future has
///    been awoken (i.e., it needs to be polled or is ready to progress).
///
/// 2. Creates a custom `RawWaker` which forms the core of the `Waker` that drives
///    the future. The `RawWaker` has the following behavior:
///    - `clone`: Creates another identical `RawWaker`.
///    - `wake` and `wake_by_ref`: Set the `woke` state to `true`, indicating the
///      task should be polled.
///    - `drop`:
pub fn block_on<F: Future>(mut fut: F) -> F::Output {
    let woke = Cell::new(true); // true pour un premier poll immédiat

    fn raw_waker(woke: *const Cell<bool>) -> RawWaker {
        unsafe fn clone(p: *const ()) -> RawWaker {
            raw_waker(p as *const Cell<bool>)
        }
        unsafe fn wake(p: *const ()) {
            let cell = &*(p as *const Cell<bool>);
            cell.set(true);
        }
        unsafe fn wake_by_ref(p: *const ()) {
            let cell = &*(p as *const Cell<bool>);
            cell.set(true);
        }
        unsafe fn drop(_: *const ()) {}
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);
        RawWaker::new(woke as *const (), &VTABLE)
    }

    let waker = unsafe { Waker::from_raw(raw_waker(&woke as *const _)) };
    let mut cx = Context::from_waker(&waker);
    // Épingler la future
    let mut fut = unsafe { Pin::new_unchecked(&mut fut) };

    loop {
        if woke.replace(false) {
            if let Poll::Ready(v) = fut.as_mut().poll(&mut cx) {
                return v;
            }
        }
        wfi(); // dormir jusqu'à la prochaine IRQ
    }
}
