use stm32f7::stm32f769::interrupt;

unsafe extern "C" {
    pub fn USART1_it_handler();
}

#[allow(non_snake_case)]
#[interrupt]
fn USART1() {
    unsafe { USART1_it_handler(); }
}
