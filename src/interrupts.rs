use stm32f7::stm32f769::interrupt;

#[allow(non_snake_case)]
#[interrupt]
fn USART1() {
    panic!("USART1 interrupt not implemented")
}
