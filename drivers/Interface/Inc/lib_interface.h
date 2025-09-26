/**
  *******************************************************************
  * @file               : lib_interface
  * @date               : Created on 24/09/2025
  * @author             : Nicolas SIMON
  *******************************************************************
  * @brief 
  *******************************************************************
  */

#ifndef LIB_INTERFACE_H
#define LIB_INTERFACE_H


/************/
/* Includes */
/************/
#include "stm32f769xx.h"

/********************/
/* Exported defines */
/********************/

/******************/
/* Exported types */
/******************/

typedef enum
{
    OK = 0,
    ERR_INTERFACE_NOT_FOUND = 1,
    ERR_WRONG_INTERFACE_ID = 2,
    ERR_READ_ONLY_INTERFACE = 3,
    ERR_WRITE_ONLY_INTERFACE = 4,
    ERR_INCOMPATIBLE_ACTION = 5,
    ERR_WRITE_ERROR = 6,
} HAL_INTERFACE_RESULT;


typedef enum
{
    SET_PIN = 0,
    CLEAR_PIN = 1,
    TOGGLE_PIN = 2
} GPIO_WRITE_ACTION;

/**********************/
/* Exported constants */
/**********************/

/**********************/
/* Exported variables */
/**********************/

/******************/
/* Exported macro */
/******************/

/**********************/
/* Exported functions */
/**********************/

/**
 * @brief Initializes the hardware abstraction layer (HAL) and necessary
 *        peripherals for the system.
 *
 * This function is responsible for configuring the system clock, initializing
 * the peripheral clocks, and setting up GPIO and USART1 UART peripherals. It
 * acts as a core initialization routine to ensure the proper configuration
 * of hardware resources before the system begins execution of its core tasks.
 *
 * The sequence of initialization includes:
 * - Configuring the system clock using SystemClock_Config.
 * - Configuring the common peripheral clocks using PeriphCommonClock_Config.
 * - Initializing the GPIO peripherals using MX_GPIO_Init.
 * - Setting up the USART1 UART peripheral using MX_USART1_UART_Init.
 */
void hal_init();

/**
 * Retrieves the interface ID associated with a given interface name.
 *
 * @param name The name of the interface to search for. It is a pointer to a null-terminated string.
 * @param id A pointer to a location where the retrieved interface ID will be stored.
 *           This value is valid only if the function returns OK.
 * @return Returns a HalInterfaceResult:
 *         - OK if the interface name was found and the ID was successfully retrieved.
 *         - ERR if the interface name was not found.
 */
HAL_INTERFACE_RESULT get_interface_id(const uint8_t *name, uint8_t *id);

/**
 * @brief Retrieves the name of an interface corresponding to the given ID.
 *
 * This function looks up the interface identified by the provided ID and copies
 * its associated name into the given buffer. If the ID is invalid (greater than
 * or equal to the allocated driver size), the function returns with an error
 * code indicating the issue.
 *
 * The interface name is stored as a null-terminated string in the provided buffer.
 *
 * @param id The ID of the interface whose name is to be retrieved.
 * @param name Pointer to a buffer where the interface name will be stored.
 *             The caller must ensure the buffer is large enough to hold the name.
 *
 * @return HAL_INTERFACE_RESULT
 *         - OK if the name is successfully retrieved.
 *         - ERR_WRONG_INTERFACE_ID if the given ID is invalid.
 */
HAL_INTERFACE_RESULT get_interface_name(uint8_t id, uint8_t *name);

/**
 * @brief Writes a specified action to a GPIO pin, identified by its interface ID.
 *
 * This function performs actions such as setting, clearing, or toggling a GPIO pin
 * based on the provided action. It verifies that the provided interface ID corresponds
 * to a valid GPIO interface and that the direction and type of the interface are compatible
 * with the operation. If any of these validations fail, an appropriate error code is returned.
 *
 * @param id The identifier of the GPIO interface within the driver allocation table.
 *           Must be less than DRIVERS_ALLOC_SIZE.
 * @param action The action to perform on the GPIO pin, specified as a value of type GPIO_WRITE_ACTION.
 *               Possible actions include:
 *               - SET_PIN: Set the pin to a high state.
 *               - CLEAR_PIN: Set the pin to a low state.
 *               - TOGGLE_PIN: Toggle the current state of the pin.
 *
 * @return HAL_INTERFACE_RESULT Status of the operation:
 *         - OK: Action completed successfully.
 *         - ERR_WRONG_INTERFACE_ID: The id does not correspond to a valid interface.
 *         - ERR_READ_ONLY_INTERFACE: The interface is read-only and cannot perform a write action.
 *         - ERR_INCOMPATIBLE_ACTION: The interface is not a GPIO type.
 */
HAL_INTERFACE_RESULT gpio_write(uint8_t id, GPIO_WRITE_ACTION action);

/**
 * @brief Writes data to a specified USART interface.
 *
 * This function sends a data buffer to the specified USART interface
 * identified by its ID. It validates the interface ID, ensures it is
 * compatible with the USART type, and confirms write permissions before
 * transmitting the data. If any condition is not met, an error code is returned.
 *
 * @param id The ID of the USART interface to which the data will be written.
 * @param str Pointer to the buffer containing the data to be transmitted.
 * @param len The length of the data to be transmitted, in bytes.
 * @return A result of type HAL_INTERFACE_RESULT, indicating success (OK) or
 *         an error code such as:
 *         - ERR_WRONG_INTERFACE_ID: Invalid USART interface ID.
 *         - ERR_READ_ONLY_INTERFACE: Interface is read-only.
 *         - ERR_INCOMPATIBLE_ACTION: Interface type is not USART.
 *         - ERR_WRITE_ERROR: Transmission failure.
 *         - OK: Data successfully written to the USART interface.
 */
HAL_INTERFACE_RESULT usart_write(uint8_t id, const uint8_t *str, uint16_t len);

/**
 * @brief Retrieves the core system clock frequency.
 *
 * This function returns the current frequency of the core system clock
 * as configured in the hardware. The frequency value is determined using
 * the hardware abstraction layer (HAL) function HAL_RCC_GetSysClockFreq.
 *
 * This information is typically used for timing-critical operations or
 * for configuring system modules that rely on accurate clock frequency
 * values.
 *
 * @return The frequency of the core system clock in hertz (Hz).
 */
uint32_t get_core_clk();

#endif //LIB_INTERFACE_H
