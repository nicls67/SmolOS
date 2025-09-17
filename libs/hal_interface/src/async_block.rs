use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use cortex_m::asm::wfi;

// Flag global pour signaler un réveil depuis n'importe quel contexte (IT inclus).
static WOKE_FLAG: AtomicBool = AtomicBool::new(true);

// Garde simple pour éviter les appels réentrants à block_on (non supporté).
static EXEC_IN_USE: AtomicBool = AtomicBool::new(false);

fn raw_waker_from_flag(flag: &'static AtomicBool) -> RawWaker {
    unsafe fn clone(p: *const ()) -> RawWaker {
        let flag = &*(p as *const AtomicBool);
        raw_waker_from_flag(flag)
    }
    unsafe fn wake(p: *const ()) {
        let flag = &*(p as *const AtomicBool);
        // Wake depuis IT/Thread: publication forte
        flag.store(true, Ordering::SeqCst);
    }
    unsafe fn wake_by_ref(p: *const ()) {
        let flag = &*(p as *const AtomicBool);
        flag.store(true, Ordering::SeqCst);
    }
    unsafe fn drop(_: *const ()) {
        // Rien à faire: flag 'static, pas de refcount
    }
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);
    RawWaker::new(flag as *const _ as *const (), &VTABLE)
}

/// Exécute un Future jusqu’à complétion de manière synchrone.
/// Non réentrant. Sécurisé pour des réveils depuis les interruptions.
pub fn block_on<F: Future>(mut fut: F) -> F::Output {
    // Wait until the previous execution is done
    while EXEC_IN_USE
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {}

    // Marquer un premier poll
    WOKE_FLAG.store(true, Ordering::SeqCst);

    // Waker basé sur un flag 'static (évite les pointeurs pendus si un wake arrive tard)
    let waker = unsafe { Waker::from_raw(raw_waker_from_flag(&WOKE_FLAG)) };
    let mut cx = Context::from_waker(&waker);

    // Épingler le future
    let mut fut = unsafe { Pin::new_unchecked(&mut fut) };

    let output = loop {
        // Acquire: observe les effets précédant le store du wake
        if WOKE_FLAG.swap(false, Ordering::Acquire) {
            match fut.as_mut().poll(&mut cx) {
                Poll::Ready(v) => break v,
                Poll::Pending => {
                    // Rien: on repasse en sommeil
                }
            }
        }
        // Attendre une IT qui fera wake() → WOKE_FLAG=true
        // Remarque: s'il n'y a pas d'IT, ce WFI peut dormir indéfiniment.
        // Adapter selon ta stratégie (time slice, timeout, etc.).
        wfi();
    };

    // Relâcher la garde
    EXEC_IN_USE.store(false, Ordering::SeqCst);
    output
}
