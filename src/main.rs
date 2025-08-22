#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};
use cortex_m::peripheral::SCB;
use cortex_m::peripheral::scb::SystemHandler;
use cortex_m::peripheral::syst::SystClkSource;

use panic_semihosting as _;

use cortex_m_rt::{entry, exception};
use cortex_m_semihosting::hprintln;

use embassy_stm32::Config;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::rcc::{
    AHBPrescaler, APBPrescaler, Hse, HseMode, Pll, PllMul, PllPDiv, PllPreDiv, PllSource, Sysclk,
};
use embassy_stm32::time::Hertz;

static TICKS: AtomicU32 = AtomicU32::new(0);
static TICKS2: AtomicU32 = AtomicU32::new(0);

#[entry]
fn main() -> ! {
    let mut config = Config::default();
    config.rcc.hsi = true;
    config.rcc.hse = Some(Hse {
        freq: Hertz(25_000_000),
        mode: HseMode::Oscillator,
    });
    config.rcc.sys = Sysclk::PLL1_P;
    config.rcc.pll_src = PllSource::HSE;
    config.rcc.pll = Some(Pll {
        prediv: PllPreDiv::DIV25,
        mul: PllMul::MUL432,
        divp: Some(PllPDiv::DIV2),
        divq: None,
        divr: None,
    });
    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV4;
    config.rcc.apb2_pre = APBPrescaler::DIV2;

    let mut cp = cortex_m::Peripherals::take().unwrap();
    cp.SYST.set_reload(216_000 - 1);
    cp.SYST.set_clock_source(SystClkSource::Core);
    cp.SYST.clear_current();
    cp.SYST.enable_interrupt();

    let p = embassy_stm32::init(config);
    let mut led = Output::new(p.PJ13, Level::High, Speed::Low);
    led.set_low();
    hprintln!("coucou");

    unsafe {
        cp.SCB.set_priority(SystemHandler::PendSV, 0xFF);
    }

    cp.SYST.enable_counter();

    loop {}
}

#[exception]
fn SysTick() {
    let value = TICKS.load(Ordering::Relaxed);

    if value >= 1000 {
        SCB::set_pendsv();
        TICKS.store(0, Ordering::Relaxed);
    } else {
        TICKS.fetch_add(1, Ordering::Relaxed);
    }
}

#[exception]
fn PendSV() {
    let value = TICKS2.load(Ordering::Relaxed);
    // Handler de l'interruption logicielle
    if value % 30 == 0 {
        hprintln!("{}", value);
    }

    TICKS2.fetch_add(1, Ordering::Relaxed);
}
