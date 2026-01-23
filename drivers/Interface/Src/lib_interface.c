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

#include "fmc.h"
#include "stm32f769i_discovery_lcd.h"


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
HAL_INTERFACE_CALLBACK G_callbacks[K_DRIVERS_ALLOC_SIZE];

/*********************/
/* Private functions */
/*********************/

/**
 * @brief Compares two null-terminated strings for equality.
 *
 * This function performs a character-by-character comparison of two
 * null-terminated strings. The comparison stops at the first mismatching
 * character or when the null terminator is encountered in either string.
 *
 * @param p_str1 Pointer to the first null-terminated string.
 * @param p_str2 Pointer to the second null-terminated string.
 *
 * @return true if both strings are identical; otherwise, false.
 */
bool str_compare(const uint8_t *p_str1, const uint8_t *p_str2)
{
    uint8_t l_i = 0;
    while (p_str1[l_i] != '\0' && p_str2[l_i] != '\0')
    {
        if (p_str1[l_i] != p_str2[l_i])
        {
            return false;
        }
        l_i++;
    }
    return true;
}

extern void SystemClock_Config();

extern void PeriphCommonClock_Config();

/**
 * @brief Validates the provided LCD interface ID and checks its compatibility.
 *
 * This function ensures that the specified ID corresponds to a valid
 * LCD interface entry in the allocated driver table. It checks if the
 * provided ID is within bounds and confirms that the driver type associated
 * with the ID is LCD.
 *
 * @param p_id The LCD interface ID to be validated.
 *
 * @return HAL_INTERFACE_RESULT indicating the result of the validation:
 * - OK: If the ID is valid and corresponds to an LCD interface.
 * - ERR_WRONG_INTERFACE_ID: If the ID exceeds the maximum allocated driver size.
 * - ERR_INCOMPATIBLE_ACTION: If the driver type for the provided ID is not LCD.
 */
HAL_INTERFACE_RESULT lcd_id_check(const uint8_t p_id)
{
    if (p_id >= K_DRIVERS_ALLOC_SIZE)
    {
        return ERR_WRONG_INTERFACE_ID;
    }

    if (K_DRIVERS_ALLOC[p_id].drv_type != LCD)
    {
        return ERR_INCOMPATIBLE_ACTION;
    }

    return OK;
}

/********************/
/* Public functions */
/********************/

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
void hal_init()
{
    //HAL_Init();
    SystemClock_Config();
    PeriphCommonClock_Config();
    MX_FMC_Init();

    drivers_init();

    // Initialize callback table to null
    for (uint8_t l_i = 0; l_i < K_DRIVERS_ALLOC_SIZE; l_i++)
    {
        G_callbacks[l_i] = NULL;
    }
}

/**
 * Retrieves the interface ID associated with a given interface name.
 *
 * @param p_name The name of the interface to search for. It is a pointer to a null-terminated string.
 * @param p_id A pointer to a location where the retrieved interface ID will be stored.
 *           This value is valid only if the function returns OK.
 * @return Returns a HalInterfaceResult:
 *         - OK if the interface name was found and the ID was successfully retrieved.
 *         - ERR if the interface name was not found.
 */
HAL_INTERFACE_RESULT get_interface_id(const uint8_t *p_name, uint8_t *p_id)
{
    for (uint8_t l_i = 0; l_i < K_DRIVERS_ALLOC_SIZE; l_i++)
    {
        if (str_compare(p_name, K_DRIVERS_ALLOC[l_i].drv_name))
        {
            *p_id = K_DRIVERS_ALLOC[l_i].drv_id;
            return OK;
        }
    }
    return ERR_INTERFACE_NOT_FOUND;
}

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
 * @param p_id The ID of the interface whose name is to be retrieved.
 * @param p_name Pointer to a buffer where the interface name will be stored.
 *             The caller must ensure the buffer is large enough to hold the name.
 *
 * @return HAL_INTERFACE_RESULT
 *         - OK if the name is successfully retrieved.
 *         - ERR_WRONG_INTERFACE_ID if the given ID is invalid.
 */
HAL_INTERFACE_RESULT get_interface_name(const uint8_t p_id, uint8_t *p_name)
{
    if (p_id >= K_DRIVERS_ALLOC_SIZE)
    {
        return ERR_WRONG_INTERFACE_ID;
    }

    uint8_t l_i = 0;
    while (K_DRIVERS_ALLOC[p_id].drv_name[l_i] != '\0')
    {
        p_name[l_i] = K_DRIVERS_ALLOC[p_id].drv_name[l_i];
        l_i++;
    }
    return OK;
}

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
uint32_t get_core_clk()
{
    return HAL_RCC_GetSysClockFreq();
}

/**
 * @brief Configures a callback function for a specified interface ID.
 *
 * This function associates a callback function with a given interface ID.
 * The callback is stored and can be invoked when required by the specified interface.
 * The ID must be within the valid range of allocated driver interfaces.
 *
 * @param p_id The ID of the interface to configure the callback for. Must be less than K_DRIVERS_ALLOC_SIZE.
 * @param p_callback The callback function pointer to assign to the specified interface ID.
 *
 * @return OK if the callback is successfully configured;
 *         ERR_WRONG_INTERFACE_ID if the provided interface ID is invalid.
 */
HAL_INTERFACE_RESULT configure_callback(const uint8_t p_id, const HAL_INTERFACE_CALLBACK p_callback)
{
    if (p_id >= K_DRIVERS_ALLOC_SIZE)
    {
        return ERR_WRONG_INTERFACE_ID;
    }

    G_callbacks[p_id] = p_callback;
    return OK;
}

/**
 * @brief Retrieves a read buffer for a specified interface ID.
 *
 * This function checks the validity of the given interface ID and retrieves
 * the associated read buffer if it is available and properly configured for
 * read operations. The buffer is returned through the provided pointer
 * parameter if all checks pass successfully.
 *
 * @param p_id The ID of the interface for which the read buffer is being requested.
 * @param p_buffer Pointer to a pointer variable where the read buffer will be stored.
 *
 * @return - OK: The read buffer was successfully retrieved.
 *         - ERR_WRONG_INTERFACE_ID: The provided interface ID is invalid or out of bounds.
 *         - ERR_WRITE_ONLY_INTERFACE: The interface is configured as write-only.
 *         - ERR_NO_BUFFER: No buffer is associated with the specified interface.
 */
HAL_INTERFACE_RESULT get_read_buffer(const uint8_t p_id, RX_BUFFER **p_buffer)
{
    if (p_id >= K_DRIVERS_ALLOC_SIZE)
    {
        return ERR_WRONG_INTERFACE_ID;
    }
    if (K_DRIVERS_ALLOC[p_id].drv_direction == OUT)
    {
        return ERR_WRITE_ONLY_INTERFACE;
    }
    if (K_DRIVERS_ALLOC[p_id].buffer == NULL)
    {
        return ERR_NO_BUFFER;
    }

    *p_buffer = K_DRIVERS_ALLOC[p_id].buffer;

    return OK;
}

#ifdef K_DRIVER_ACTIVATE_GPIO
/**
 * @brief Writes a specified action to a GPIO pin, identified by its interface ID.
 *
 * This function performs actions such as setting, clearing, or toggling a GPIO pin
 * based on the provided action. It verifies that the provided interface ID corresponds
 * to a valid GPIO interface and that the direction and type of the interface are compatible
 * with the operation. If any of these validations fail, an appropriate error code is returned.
 *
 * @param p_id The identifier of the GPIO interface within the driver allocation table.
 *           Must be less than K_DRIVERS_ALLOC_SIZE.
 * @param p_action The action to perform on the GPIO pin, specified as a value of type GPIO_WRITE_ACTION.
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
HAL_INTERFACE_RESULT gpio_write(const uint8_t p_id, const GPIO_WRITE_ACTION p_action)
{
    if (p_id >= K_DRIVERS_ALLOC_SIZE)
    {
        return ERR_WRONG_INTERFACE_ID;
    }

    if (K_DRIVERS_ALLOC[p_id].drv_direction == IN)
    {
        return ERR_READ_ONLY_INTERFACE;
    }

    if (K_DRIVERS_ALLOC[p_id].drv_type != GPIO)
    {
        return ERR_INCOMPATIBLE_ACTION;
    }

    const GPIO_ALLOC *l_gpio = K_DRIVERS_ALLOC[p_id].drv;
    switch (p_action)
    {
        case SET_PIN:
            HAL_GPIO_WritePin(l_gpio->gpio, l_gpio->pin, GPIO_PIN_SET);
            break;
        case CLEAR_PIN:
            HAL_GPIO_WritePin(l_gpio->gpio, l_gpio->pin, GPIO_PIN_RESET);
            break;
        case TOGGLE_PIN:
            HAL_GPIO_TogglePin(l_gpio->gpio, l_gpio->pin);
            break;
    }

    return OK;
}
#endif

#ifdef K_DRIVER_ACTIVATE_USART
/**
 * @brief Writes data to a specified USART interface.
 *
 * This function sends a data buffer to the specified USART interface
 * identified by its ID. It validates the interface ID, ensures it is
 * compatible with the USART type, and confirms write permissions before
 * transmitting the data. If any condition is not met, an error code is returned.
 *
 * @param p_id The ID of the USART interface to which the data will be written.
 * @param p_str Pointer to the buffer containing the data to be transmitted.
 * @param p_len The length of the data to be transmitted, in bytes.
 * @return A result of type HAL_INTERFACE_RESULT, indicating success (OK) or
 *         an error code such as:
 *         - ERR_WRONG_INTERFACE_ID: Invalid USART interface ID.
 *         - ERR_READ_ONLY_INTERFACE: Interface is read-only.
 *         - ERR_INCOMPATIBLE_ACTION: Interface type is not USART.
 *         - ERR_WRITE_ERROR: Transmission failure.
 *         - OK: Data successfully written to the USART interface.
 */
HAL_INTERFACE_RESULT usart_write(const uint8_t p_id, const uint8_t *p_str, const uint16_t p_len)
{
    if (p_id >= K_DRIVERS_ALLOC_SIZE)
    {
        return ERR_WRONG_INTERFACE_ID;
    }

    if (K_DRIVERS_ALLOC[p_id].drv_direction == IN)
    {
        return ERR_READ_ONLY_INTERFACE;
    }

    if (K_DRIVERS_ALLOC[p_id].drv_type != USART)
    {
        return ERR_INCOMPATIBLE_ACTION;
    }

    if (HAL_UART_Transmit(K_DRIVERS_ALLOC[p_id].drv, p_str, p_len, HAL_MAX_DELAY) != HAL_OK)
    {
        return ERR_WRITE_ERROR;
    }
    return OK;
}

/**
 * @brief Callback function triggered upon UART receive complete interrupt.
 *
 * This function is executed when a UART receive operation is completed,
 * specifically in interrupt mode. The function checks if the completed
 * operation is associated with USART1 and reinitializes the interrupt
 * mechanism for further data reception. Additionally, it identifies
 * the corresponding driver and calls the associated callback function
 * if it's configured.
 *
 * @param p_huart Pointer to the UART handle structure that contains
 *              information about the UART instance.
 */
void HAL_UART_RxCpltCallback(UART_HandleTypeDef *p_huart)
{
    // Re-initialize IT
    if (p_huart->Instance == USART1)
    {
        HAL_UART_Receive_IT(&huart1, G_USART1_BUFFER.buffer, 1);
        G_USART1_BUFFER.size++;
    }

    // Get the ID corresponding to the handler
    for (uint8_t l_i = 0; l_i < K_DRIVERS_ALLOC_SIZE; l_i++)
    {
        if (K_DRIVERS_ALLOC[l_i].drv == p_huart)
        {
            // If a callback is configured
            if (G_callbacks[l_i] != NULL)
            {
                // Call the callback
                G_callbacks[l_i](l_i);
            }
            break;
        }
    }
}
#endif


#ifdef K_DRIVER_ACTIVATE_LCD
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
 * @param p_id The ID of the LCD interface to be enabled or disabled.
 * @param p_enable A boolean flag where true enables the display, and false disables it.
 *
 * @return HAL_INTERFACE_RESULT
 *         - OK: If the operation was successful.
 *         - ERR_WRONG_INTERFACE_ID: If the ID is outside the allowable range.
 *         - ERR_READ_ONLY_INTERFACE: If the specified interface is read-only.
 *         - ERR_INCOMPATIBLE_ACTION: If the specified interface is not of LCD type.
 */
HAL_INTERFACE_RESULT lcd_enable(const uint8_t p_id, const bool p_enable)
{
    const HAL_INTERFACE_RESULT l_result = lcd_id_check(p_id);
    if (l_result != OK)
    {
        return l_result;
    }

    if (p_enable)
    {
        BSP_LCD_DisplayOn();
    }
    else
    {
        BSP_LCD_DisplayOff();
    }
    return OK;
}

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
 * @param p_id The interface ID corresponding to the target LCD. Must be within
 *           the valid range of interface IDs and associated with an LCD interface.
 * @param p_layer The layer index to select for clearing.
 * @param p_color The color value to fill the selected layer.
 * @return HAL_INTERFACE_RESULT Returns one of the following results:
 *         - OK on success.
 *         - ERR_WRONG_INTERFACE_ID if the specified interface ID is invalid.
 *         - ERR_READ_ONLY_INTERFACE if the interface ID corresponds to a read-only interface.
 *         - ERR_INCOMPATIBLE_ACTION if the specified interface is not an LCD type.
 */
HAL_INTERFACE_RESULT lcd_clear(const uint8_t p_id, const uint8_t p_layer, const uint32_t p_color)
{
    const HAL_INTERFACE_RESULT l_result = lcd_id_check(p_id);
    if (l_result != OK)
    {
        return l_result;
    }

    BSP_LCD_SelectLayer(p_layer);
    BSP_LCD_Clear(p_color);

    return OK;
}

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
 * @param p_id The interface ID for the LCD driver to be used. Must refer to
 *           a valid writable LCD interface.
 * @param p_layer The LCD layer to which the pixel is drawn. Must be a valid
 *              layer value supported by the hardware.
 * @param p_x The X-coordinate of the pixel to be drawn.
 * @param p_y The Y-coordinate of the pixel to be drawn.
 * @param p_color The RGB color value of the pixel to be drawn.
 *
 * @return HAL_INTERFACE_RESULT Returns OK if the pixel is drawn successfully.
 *         Returns an error code (e.g., ERR_WRONG_INTERFACE_ID,
 *         ERR_INCOMPATIBLE_ACTION, ERR_READ_ONLY_INTERFACE) if the interface ID
 *         is invalid or the action is disallowed.
 */
HAL_INTERFACE_RESULT lcd_draw_pixel(const uint8_t p_id, const uint8_t p_layer, const uint16_t p_x, const uint16_t p_y,
                                    const uint32_t p_color)
{
    const HAL_INTERFACE_RESULT l_result = lcd_id_check(p_id);
    if (l_result != OK)
    {
        return l_result;
    }

    BSP_LCD_SelectLayer(p_layer);
    BSP_LCD_DrawPixel(p_x, p_y, p_color);

    return OK;
}

/**
 * @brief Retrieves the size of an LCD interface specified by its ID.
 *
 * This function checks the validity of the given LCD ID and fetches
 * the horizontal and vertical dimensions of the LCD screen in pixels.
 * It populates the provided pointers with the retrieved dimensions.
 *
 * @param p_id The ID of the LCD interface to query.
 * @param p_x Pointer to a variable where the horizontal size (width) of the LCD will be stored.
 * @param p_y Pointer to a variable where the vertical size (height) of the LCD will be stored.
 *
 * @return OK if the operation is successful.
 *         ERR_WRONG_INTERFACE_ID if the ID is invalid or out of bounds.
 *         ERR_INCOMPATIBLE_ACTION if the ID does not correspond to an LCD interface.
 */
HAL_INTERFACE_RESULT get_lcd_size(const uint8_t p_id, uint16_t *p_x, uint16_t *p_y)
{
    const HAL_INTERFACE_RESULT l_result = lcd_id_check(p_id);
    if (l_result != OK)
    {
        return l_result;
    }

    *p_x = (uint16_t) BSP_LCD_GetXSize();
    *p_y = (uint16_t) BSP_LCD_GetYSize();

    return OK;
}

/**
 * @brief Retrieves the frame buffer address for a specific LCD interface and layer.
 *
 * This function checks the validity of the provided LCD interface ID and layer,
 * and if valid, retrieves the starting address of the frame buffer corresponding
 * to the specified layer. It ensures the interface ID corresponds to an LCD interface
 * and that the provided parameters are compatible.
 *
 * @param p_id The ID of the LCD interface to check.
 * @param p_layer The layer number for which the frame buffer address is requested.
 * @param p_addr Pointer to a variable where the frame buffer address will be stored.
 *
 * @return Returns a result of type HAL_INTERFACE_RESULT indicating the outcome:
 *         - OK: The operation was successful, and the frame buffer address is available.
 *         - ERR_WRONG_INTERFACE_ID: The specified ID is outside the valid range.
 *         - ERR_INCOMPATIBLE_ACTION: The interface ID does not correspond to an LCD.
 */
HAL_INTERFACE_RESULT get_fb_address(const uint8_t p_id, const uint8_t p_layer, uint32_t *p_addr)
{
    const HAL_INTERFACE_RESULT l_result = lcd_id_check(p_id);
    if (l_result != OK)
    {
        return l_result;
    }

    switch (p_layer)
    {
        case 1:
            *p_addr = LCD_FB_START_ADDRESS;
            break;
        default:
            break;
    }

    return OK;
}

/**
 * @brief Sets the frame buffer address for a specific LCD layer.
 *
 * This function sets the address of the frame buffer for the specified
 * LCD layer. It first validates the LCD ID to ensure it is within the
 * range of allocated drivers and that the driver is of type LCD. If the
 * validation fails, it returns an appropriate error code. Otherwise, it
 * updates the specified layer's address.
 *
 * @param p_id The identifier of the LCD driver to modify.
 * @param p_layer The layer index for which the frame buffer address is to be set.
 * @param p_addr The new frame buffer address to be assigned.
 *
 * @return An instance of HAL_INTERFACE_RESULT indicating the result of
 *         the operation. Possible return values are:
 *         - OK: The operation was successful.
 *         - ERR_WRONG_INTERFACE_ID: The provided ID is invalid.
 *         - ERR_INCOMPATIBLE_ACTION: The driver ID does not correspond
 *           to an LCD interface.
 */
HAL_INTERFACE_RESULT set_fb_address(const uint8_t p_id, const uint8_t p_layer, const uint32_t p_addr)
{
    const HAL_INTERFACE_RESULT l_result = lcd_id_check(p_id);
    if (l_result != OK)
    {
        return l_result;
    }

    BSP_LCD_SetLayerAddress(p_layer, p_addr);
    return OK;
}
#endif
