use crate::InterfaceReadActions::UartRead;
use crate::InterfaceWriteActions::{GpioWrite, UartWrite};
use crate::UartReadActions::Read;
use core::cell::Cell;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use cortex_m::asm::wfi;
use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Async;
use embassy_stm32::usart::Uart;

pub enum InterfaceWriteActions {
    GpioWrite(GpioWriteActions),
    UartWrite(UartWriteActions),
}

impl InterfaceWriteActions {
    pub fn name(&self) -> &'static str {
        match self {
            GpioWrite(_) => "GPIO Write",
            UartWrite(_) => "Uart Write",
        }
    }
}

pub enum GpioWriteActions {
    Set,
    Clear,
    Toggle,
}

impl GpioWriteActions {
    pub fn action(&self, pin: &mut Output) {
        match self {
            GpioWriteActions::Set => pin.set_high(),
            GpioWriteActions::Clear => pin.set_low(),
            GpioWriteActions::Toggle => pin.toggle(),
        }
    }
}

pub enum UartWriteActions {
    SendChar(u8),
}

impl UartWriteActions {
    pub fn action(&self, uart: &mut Uart<'static, Async>) {
        match self {
            UartWriteActions::SendChar(c) => {
                let data_arr = [*c];
                block_on(uart.write(&data_arr)).unwrap();
            }
        }
    }
}

fn block_on<F: Future>(mut fut: F) -> F::Output {
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

pub enum InterfaceReadActions<'a> {
    UartRead(UartReadActions<'a>),
}

impl InterfaceReadActions<'_> {
    pub fn name(&self) -> &'static str {
        match self {
            UartRead(_) => "UART Read",
        }
    }
}

pub enum UartReadActions<'a> {
    Read(&'a mut [u8]),
}

impl UartReadActions<'_> {
    pub fn action(&mut self, uart: &mut Uart<'static, Async>) {
        match self {
            Read(c) => {
                block_on(uart.read_until_idle(c)).unwrap();
            }
        }
    }
}
