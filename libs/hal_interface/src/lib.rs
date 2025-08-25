#![no_std]

mod errors;

use embassy_stm32::Config;
use embassy_stm32::rcc::{
    AHBPrescaler, APBPrescaler, Hse, HseMode, Pll, PllMul, PllPDiv, PllPreDiv, PllSource, Sysclk,
};
use embassy_stm32::time::Hertz;

pub use errors::*;

pub enum CoreClkConfig {
    Max,
    Default,
}
pub struct HalConfig {
    pub core_clk_config: CoreClkConfig,
}

pub struct Hal {
    peripherals: embassy_stm32::Peripherals,
    pub core_clk_freq: u32,
}

impl Hal {
    pub fn init(hal_config: HalConfig) -> Self {
        // Initialize HAL
        let mut config = Config::default();
        let mut core_freq = 16_000_000;

        if let CoreClkConfig::Max = hal_config.core_clk_config {
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

            core_freq = 216_000_000;
        }

        Self {
            peripherals: embassy_stm32::init(config),
            core_clk_freq: core_freq,
        }
    }
}
