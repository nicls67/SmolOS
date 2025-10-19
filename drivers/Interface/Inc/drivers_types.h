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
typedef enum
{
    GPIO,
    USART,
    LCD
} INTERFACE_TYPE;

typedef enum
{
    IN,
    OUT,
    INOUT
} INTERFACE_DIRECTION;


typedef struct
{
    uint8_t *drv_name;
    INTERFACE_TYPE drv_type;
    INTERFACE_DIRECTION drv_direction;
    void *drv;
    uint8_t drv_id;
} DRIVER_ALLOC;


typedef struct
{
    GPIO_TypeDef *gpio;
    uint16_t pin;
} GPIO_ALLOC;

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
