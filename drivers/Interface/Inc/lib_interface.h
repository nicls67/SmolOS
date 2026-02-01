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
#include <stdint.h>

/********************/
/* Exported defines */
/********************/

/******************/
/* Exported types */
/******************/

/**
 * @brief Represents the result codes returned by the hardware abstraction layer.
 */
typedef enum
{
    OK = 0,                         /**< Operation successful */
    ERR_INTERFACE_NOT_FOUND = 1,    /**< Specified interface not found */
    ERR_WRONG_INTERFACE_ID = 2,     /**< Provided interface ID is invalid */
    ERR_READ_ONLY_INTERFACE = 3,    /**< Attempted to write to a read-only interface */
    ERR_WRITE_ONLY_INTERFACE = 4,   /**< Attempted to read from a write-only interface */
    ERR_INCOMPATIBLE_ACTION = 5,    /**< Requested action not compatible with interface type */
    ERR_WRITE_ERROR = 6,            /**< Error during write operation */
    ERR_NO_BUFFER = 7,              /**< No buffer associated with the interface for reading */
} HAL_INTERFACE_RESULT;

/**
 * @brief Represents possible actions on a GPIO pin.
 */
typedef enum
{
    SET_PIN = 0,    /**< Set pin high */
    CLEAR_PIN = 1,  /**< Set pin low */
    TOGGLE_PIN = 2  /**< Toggle pin state */
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
 * @brief Initializes the Hardware Abstraction Layer (HAL).
 */
void hal_init();

/**
 * @brief Retrieves the interface ID associated with a given interface name.
 *
 * @param p_name Pointer to a null-terminated string containing the interface name.
 * @param p_id Pointer to where the interface ID will be stored on success.
 * @return OK if found, ERR_INTERFACE_NOT_FOUND otherwise.
 */
HAL_INTERFACE_RESULT get_interface_id(const uint8_t *p_name, uint8_t *p_id);

/**
 * @brief Retrieves the name of an interface corresponding to the given ID.
 *
 * @param p_id The interface ID.
 * @param p_name Pointer to a buffer where the name will be stored as a null-terminated string.
 * @return OK if successful, ERR_WRONG_INTERFACE_ID otherwise.
 */
HAL_INTERFACE_RESULT get_interface_name(const uint8_t p_id, uint8_t *p_name);

/**
 * @brief Configures a callback function for a specific interface.
 *
 * @param p_id The interface ID.
 * @param p_callback The callback function pointer.
 * @return OK if successful, ERR_WRONG_INTERFACE_ID otherwise.
 */
HAL_INTERFACE_RESULT configure_callback(const uint8_t p_id, const HAL_INTERFACE_CALLBACK p_callback);

/**
 * @brief Writes an action to a GPIO interface.
 *
 * @param p_id The GPIO interface ID.
 * @param p_action The action to perform (SET, CLEAR, TOGGLE).
 * @return OK if successful, or an error code.
 */
HAL_INTERFACE_RESULT gpio_write(const uint8_t p_id, const GPIO_WRITE_ACTION p_action);

/**
 * @brief Writes a string to a USART interface.
 *
 * @param p_id The USART interface ID.
 * @param p_str Pointer to the string data.
 * @param p_len Length of the data in bytes.
 * @return OK if successful, or an error code.
 */
HAL_INTERFACE_RESULT usart_write(const uint8_t p_id, const uint8_t *p_str, const uint16_t p_len);

/**
 * @brief Retrieves the receive buffer for a given interface.
 *
 * @param p_id The interface ID.
 * @param p_buffer Pointer to a pointer that will be set to the RX buffer address.
 * @return OK if successful, or an error code.
 */
HAL_INTERFACE_RESULT get_read_buffer(const uint8_t p_id, RX_BUFFER **p_buffer);

/**
 * @brief Returns the current core clock frequency in Hz.
 *
 * @return The clock frequency.
 */
uint32_t get_core_clk();

/**
 * @brief Enables or disables an LCD interface.
 *
 * @param p_id The LCD interface ID.
 * @param p_enable True to enable, false to disable.
 * @return OK if successful, or an error code.
 */
HAL_INTERFACE_RESULT lcd_enable(const uint8_t p_id, const bool p_enable);

/**
 * @brief Clears an LCD layer with a color.
 *
 * @param p_id The LCD interface ID.
 * @param p_layer The layer index.
 * @param p_color The ARGB color.
 * @return OK if successful, or an error code.
 */
HAL_INTERFACE_RESULT lcd_clear(const uint8_t p_id, const uint8_t p_layer, const uint32_t p_color);

/**
 * @brief Draws a pixel on an LCD layer.
 *
 * @param p_id The LCD interface ID.
 * @param p_layer The layer index.
 * @param p_x X coordinate.
 * @param p_y Y coordinate.
 * @param p_color The ARGB color.
 * @return OK if successful, or an error code.
 */
HAL_INTERFACE_RESULT lcd_draw_pixel(const uint8_t p_id, const uint8_t p_layer, const uint16_t p_x, const uint16_t p_y, const uint32_t p_color);

/**
 * @brief Retrieves the dimensions of an LCD.
 *
 * @param p_id The LCD interface ID.
 * @param p_x Pointer to store width.
 * @param p_y Pointer to store height.
 * @return OK if successful, or an error code.
 */
HAL_INTERFACE_RESULT get_lcd_size(const uint8_t p_id, uint16_t *p_x, uint16_t *p_y);

/**
 * @brief Retrieves the frame buffer address for an LCD layer.
 *
 * @param p_id The LCD interface ID.
 * @param p_layer The layer index.
 * @param p_addr Pointer to store the address.
 * @return OK if successful, or an error code.
 */
HAL_INTERFACE_RESULT get_fb_address(const uint8_t p_id, const uint8_t p_layer, uint32_t *p_addr);

/**
 * @brief Sets the frame buffer address for an LCD layer.
 *
 * @param p_id The LCD interface ID.
 * @param p_layer The layer index.
 * @param p_addr The new address.
 * @return OK if successful, or an error code.
 */
HAL_INTERFACE_RESULT set_fb_address(const uint8_t p_id, const uint8_t p_layer, const uint32_t p_addr);

/**
 * @brief Provides a delay in milliseconds.
 *
 * @param p_ms Number of milliseconds to delay.
 */
void HAL_Delay(uint32_t p_ms);

#endif //LIB_INTERFACE_H

