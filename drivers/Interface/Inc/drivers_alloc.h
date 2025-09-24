/**
  *******************************************************************
  * @file               : drivers_alloc
  * @date               : Created on 24/09/2025
  * @author             : Nicolas SIMON
  *******************************************************************
  * @brief 
  *******************************************************************
  */

#ifndef DRIVERS_ALLOC_H
#define DRIVERS_ALLOC_H
#include <stdint.h>

#include "lib_interface.h"
#include "stm32f769xx.h"

/************/
/* Includes */
/************/

/********************/
/* Exported defines */
/********************/
#define DRIVERS_ALLOC_SIZE 3

/******************/
/* Exported types */
/******************/

typedef enum
{
    GPIO,
    USART
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
extern const DRIVER_ALLOC DRIVERS_ALLOC[];

/**********************/
/* Exported variables */
/**********************/

/******************/
/* Exported macro */
/******************/

/**********************/
/* Exported functions */
/**********************/


#endif //DRIVERS_ALLOC_H
