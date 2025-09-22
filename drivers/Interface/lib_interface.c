/**
  *******************************************************************
  * @file               : lib_interface
  * @date               : Created on 22/09/2025
  * @author             : Nicolas SIMON
  *******************************************************************
  * @brief 
  *******************************************************************
  */


/********************/
/* Private includes */
/********************/
#include "gpio.h"
#include "usart.h"

/*******************/
/* Private typedef */
/*******************/

/*******************/
/* Private defines */
/*******************/

/*********************/
/* Private constants */
/*********************/

/******************/
/* Private macros */
/******************/

/*********************/
/* Private variables */
/*********************/

/*********************/
/* Private functions */
/*********************/


/********************/
/* Public functions */
/********************/
void init()
{
    //HAL_Init();
    SystemClock_Config();
    PeriphCommonClock_Config();
    MX_GPIO_Init();
    MX_USART1_UART_Init();
}

void toggle_pin()
{
    HAL_GPIO_TogglePin(GPIOJ, GPIO_PIN_5);
}

void usart_write(const uint8_t* str, uint16_t len)
{
    HAL_UART_Transmit(&huart1, str, len, HAL_MAX_DELAY);
}
