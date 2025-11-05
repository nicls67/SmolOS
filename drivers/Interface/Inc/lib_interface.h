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

#endif //LIB_INTERFACE_H