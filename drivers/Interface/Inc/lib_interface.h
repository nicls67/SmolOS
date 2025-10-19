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
#include <stdbool.h>

#include "drivers_types.h"
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

/**
 * @brief Enables or disables the LCD identified by the given ID.
 *
 * This function manages the display state of an LCD peripheral, turning it
 * on or off based on the enable parameter. It verifies the validity of the
 * specified interface ID and ensures it corresponds to a valid and compatible
 * LCD-type interface.
 *
 * If the ID is valid, the function interacts with the hardware to modify the
 * state of the LCD by calling BSP_LCD_DisplayOn or BSP_LCD_DisplayOff, depending
 * on the value of the enable parameter.
 *
 * The following conditions are checked before performing the operation:
 * - The ID must be within the range of allocated driver entries.
 * - The specified interface must not be read-only.
 * - The specified interface must be of type LCD.
 *
 * @param id The ID of the LCD interface to be enabled or disabled.
 * @param enable A boolean flag where true enables the display, and false disables it.
 *
 * @return HAL_INTERFACE_RESULT
 *         - OK: If the operation was successful.
 *         - ERR_WRONG_INTERFACE_ID: If the ID is outside the allowable range.
 *         - ERR_READ_ONLY_INTERFACE: If the specified interface is read-only.
 *         - ERR_INCOMPATIBLE_ACTION: If the specified interface is not of LCD type.
 */
HAL_INTERFACE_RESULT lcd_enable(uint8_t id, bool enable);

/**
 * @brief Clears the specified LCD layer with the given color.
 *
 * This function selects the specified LCD layer and fills it with the specified color.
 * Before performing the operation, it checks the validity of the LCD interface ID.
 *
 * The function performs the following steps:
 * - Validates the LCD interface ID using lcd_id_check.
 * - Selects the specified layer using BSP_LCD_SelectLayer.
 * - Clears the selected layer with the provided color using BSP_LCD_Clear.
 *
 * @param id The interface ID corresponding to the target LCD. Must be within
 *           the valid range of interface IDs and associated with an LCD interface.
 * @param layer The layer index to select for clearing.
 * @param color The color value to fill the selected layer.
 * @return HAL_INTERFACE_RESULT Returns one of the following results:
 *         - OK on success.
 *         - ERR_WRONG_INTERFACE_ID if the specified interface ID is invalid.
 *         - ERR_READ_ONLY_INTERFACE if the interface ID corresponds to a read-only interface.
 *         - ERR_INCOMPATIBLE_ACTION if the specified interface is not an LCD type.
 */
HAL_INTERFACE_RESULT lcd_clear(uint8_t id, uint8_t layer, uint32_t color);

/**
 * @brief Draws a pixel on the specified LCD layer at the given coordinates
 *        with the specified color.
 *
 * This function checks the validity of the LCD interface using the given
 * interface ID, selects the appropriate layer, and draws a pixel at the
 * specified X and Y coordinates using the provided color. Errors may be
 * returned if the interface ID does not refer to a compatible writable LCD
 * interface or if the ID does not exist.
 *
 * @param id The interface ID for the LCD driver to be used. Must refer to
 *           a valid writable LCD interface.
 * @param layer The LCD layer to which the pixel is drawn. Must be a valid
 *              layer value supported by the hardware.
 * @param x The X-coordinate of the pixel to be drawn.
 * @param y The Y-coordinate of the pixel to be drawn.
 * @param color The RGB color value of the pixel to be drawn.
 *
 * @return HAL_INTERFACE_RESULT Returns OK if the pixel is drawn successfully.
 *         Returns an error code (e.g., ERR_WRONG_INTERFACE_ID,
 *         ERR_INCOMPATIBLE_ACTION, ERR_READ_ONLY_INTERFACE) if the interface ID
 *         is invalid or the action is disallowed.
 */
HAL_INTERFACE_RESULT lcd_draw_pixel(uint8_t id, uint8_t layer, uint16_t x, uint16_t y, uint32_t color);

/**
 * @brief Retrieves the size (width and height) of the specified LCD panel.
 *
 * This function checks the validity of the given LCD interface ID and ensures
 * it is configured for proper use as an output-type LCD device. If the interface
 * is valid, it retrieves the screen dimensions (width and height) in pixels.
 *
 * @param id The ID of the LCD interface to query. Must be within the valid range
 *           and correspond to an output-type LCD interface.
 * @param size Pointer to a PIXEL_COORD structure where the LCD dimensions will
 *             be stored. The `x` member will store the width, and the `y` member
 *             will store the height of the LCD.
 * @return A HAL_INTERFACE_RESULT indicating the result of the operation:
 *         - OK: The operation was successful, and the size has been retrieved.
 *         - ERR_WRONG_INTERFACE_ID: The provided ID is out of range.
 *         - ERR_READ_ONLY_INTERFACE: The interface is read-only and cannot be queried.
 *         - ERR_INCOMPATIBLE_ACTION: The ID does not correspond to an LCD device.
 */
HAL_INTERFACE_RESULT get_lcd_size(uint8_t id, uint16_t *x, uint16_t *y);

/**
 * @brief Retrieves the frame buffer address for a specific LCD interface and layer.
 *
 * This function checks the validity of the provided LCD interface ID and layer,
 * and if valid, retrieves the starting address of the frame buffer corresponding
 * to the specified layer. It ensures the interface ID corresponds to an LCD interface
 * and that the provided parameters are compatible.
 *
 * @param id The ID of the LCD interface to check.
 * @param layer The layer number for which the frame buffer address is requested.
 * @param addr Pointer to a variable where the frame buffer address will be stored.
 *
 * @return Returns a result of type HAL_INTERFACE_RESULT indicating the outcome:
 *         - OK: The operation was successful, and the frame buffer address is available.
 *         - ERR_WRONG_INTERFACE_ID: The specified ID is outside the valid range.
 *         - ERR_INCOMPATIBLE_ACTION: The interface ID does not correspond to an LCD.
 */
HAL_INTERFACE_RESULT get_fb_address(uint8_t id, uint8_t layer, uint32_t *addr);

/**
 * @brief Sets the frame buffer address for a specific LCD layer.
 *
 * This function sets the address of the frame buffer for the specified
 * LCD layer. It first validates the LCD ID to ensure it is within the
 * range of allocated drivers and that the driver is of type LCD. If the
 * validation fails, it returns an appropriate error code. Otherwise, it
 * updates the specified layer's address.
 *
 * @param id The identifier of the LCD driver to modify.
 * @param layer The layer index for which the frame buffer address is to be set.
 * @param addr The new frame buffer address to be assigned.
 *
 * @return An instance of HAL_INTERFACE_RESULT indicating the result of
 *         the operation. Possible return values are:
 *         - OK: The operation was successful.
 *         - ERR_WRONG_INTERFACE_ID: The provided ID is invalid.
 *         - ERR_INCOMPATIBLE_ACTION: The driver ID does not correspond
 *           to an LCD interface.
 */
HAL_INTERFACE_RESULT set_fb_address(uint8_t id, uint8_t layer, uint32_t addr);

#endif //LIB_INTERFACE_H
