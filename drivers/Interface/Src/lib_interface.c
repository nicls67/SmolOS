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
#include "../Inc/lib_interface.h"
#include "../Inc/drivers_alloc.h"
#include <stdbool.h>
#include <sys/types.h>


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
bool str_compare(const uint8_t *str1, const uint8_t *str2)
{
    uint8_t i = 0;
    while (str1[i] != '\0' && str2[i] != '\0')
    {
        if (str1[i] != str2[i])
        {
            return false;
        }
        i++;
    }
    return true;
}

extern void SystemClock_Config();

extern void PeriphCommonClock_Config();

/********************/
/* Public functions */
/********************/

void hal_init()
{
    //HAL_Init();
    SystemClock_Config();
    PeriphCommonClock_Config();
    MX_GPIO_Init();
    MX_USART1_UART_Init();
}

HAL_INTERFACE_RESULT get_interface_id(const uint8_t *name, uint8_t *id)
{
    for (uint8_t i = 0; i < DRIVERS_ALLOC_SIZE; i++)
    {
        if (str_compare(name, DRIVERS_ALLOC[i].drv_name))
        {
            *id = DRIVERS_ALLOC[i].drv_id;
            return OK;
        }
    }
    return ERR_INTERFACE_NOT_FOUND;
}

HAL_INTERFACE_RESULT get_interface_name(const uint8_t id, uint8_t *name)
{
    if (id >= DRIVERS_ALLOC_SIZE)
    {
        return ERR_WRONG_INTERFACE_ID;
    }

    uint8_t i = 0;
    while (*DRIVERS_ALLOC[i].drv_name != '\0')
    {
        name[i] = *DRIVERS_ALLOC[i].drv_name;
        i++;
    }
    return OK;
}

HAL_INTERFACE_RESULT gpio_write(const uint8_t id, const GPIO_WRITE_ACTION action)
{
    if (id >= DRIVERS_ALLOC_SIZE)
    {
        return ERR_WRONG_INTERFACE_ID;
    }

    if (DRIVERS_ALLOC[id].drv_direction == IN)
    {
        return ERR_READ_ONLY_INTERFACE;
    }

    if (DRIVERS_ALLOC[id].drv_type != GPIO)
    {
        return ERR_INCOMPATIBLE_ACTION;
    }

    const GPIO_ALLOC *gpio = DRIVERS_ALLOC[id].drv;
    switch (action)
    {
        case SET_PIN:
            HAL_GPIO_WritePin(gpio->gpio, gpio->pin, GPIO_PIN_SET);
            break;
        case CLEAR_PIN:
            HAL_GPIO_WritePin(gpio->gpio, gpio->pin, GPIO_PIN_RESET);
            break;
        case TOGGLE_PIN:
            HAL_GPIO_TogglePin(gpio->gpio, gpio->pin);
            break;
    }

    return OK;
}


HAL_INTERFACE_RESULT usart_write(const uint8_t id, const uint8_t *str, const uint16_t len)
{
    if (id >= DRIVERS_ALLOC_SIZE)
    {
        return ERR_WRONG_INTERFACE_ID;
    }

    if (DRIVERS_ALLOC[id].drv_direction == IN)
    {
        return ERR_READ_ONLY_INTERFACE;
    }

    if (DRIVERS_ALLOC[id].drv_type != USART)
    {
        return ERR_INCOMPATIBLE_ACTION;
    }

    if (HAL_UART_Transmit(DRIVERS_ALLOC[id].drv, str, len, HAL_MAX_DELAY) != HAL_OK)
    {
        return ERR_WRITE_ERROR;
    }
    return OK;
}

uint32_t get_core_clk()
{
    return HAL_RCC_GetSysClockFreq();
}
