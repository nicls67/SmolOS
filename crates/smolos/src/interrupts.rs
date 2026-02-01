use stm32f7::stm32f769::interrupt;

unsafe extern "C" {
    /// External C function that handles the USART1 interrupt logic.
    pub fn USART1_it_handler();
}

/// Interrupt handler for USART1.
///
/// This function is called by the hardware when a USART1 interrupt occurs.
/// It delegates the handling to the external C function `USART1_it_handler`.
#[allow(non_snake_case)]
#[interrupt]
fn USART1() {
    unsafe {
        USART1_it_handler();
    }
}
