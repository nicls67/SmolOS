/**
  *******************************************************************
  * @file               : drivers_types
  * @date               : Created on 26/09/2025
  * @author             : Nicolas SIMON
  *******************************************************************
  * @brief 
  *******************************************************************
  */

#ifndef DRIVERS_TYPES_H
#define DRIVERS_TYPES_H

/************/
/* Includes */
/************/
#include <stdint.h>
#include "stm32f769xx.h"

/********************/
/* Exported defines */
/********************/

/******************/
/* Exported types */
/******************/

/**
 * @brief Represents the types of hardware interfaces supported.
 */
typedef enum
{
    GPIO,   /**< General Purpose Input/Output */
    USART,  /**< Universal Synchronous/Asynchronous Receiver/Transmitter */
    LCD     /**< Liquid Crystal Display */
} INTERFACE_TYPE;

/**
 * @brief Represents the data flow direction of an interface.
 */
typedef enum
{
    IN,     /**< Input only */
    OUT,    /**< Output only */
    INOUT   /**< Bidirectional */
} INTERFACE_DIRECTION;

/**
 * @brief Configuration structure for an allocated driver interface.
 */
typedef struct
{
    uint8_t *drv_name;                  /**< Unique name of the interface */
    INTERFACE_TYPE drv_type;           /**< Type of the interface */
    INTERFACE_DIRECTION drv_direction;  /**< Data direction */
    void *drv;                          /**< Pointer to the underlying hardware handle */
    void *buffer;                       /**< Pointer to an optional data buffer */
    uint8_t drv_id;                     /**< Unique identifier for the interface */
} DRIVER_ALLOC;

/**
 * @brief Allocation structure for GPIO-specific data.
 */
typedef struct
{
    GPIO_TypeDef *gpio;                 /**< Pointer to the GPIO peripheral base address */
    uint16_t pin;                       /**< Pin number */
} GPIO_ALLOC;

/**
 * @brief Type definition for HAL interface callbacks.
 *
 * @param p_id The interface ID that triggered the callback.
 */
typedef void (*HAL_INTERFACE_CALLBACK)(uint8_t);

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


#endif //DRIVERS_TYPES_H