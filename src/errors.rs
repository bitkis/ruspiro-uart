/*************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **************************************************************************************************/

//! # UART Errors
//! Definition of the error type that can occur within the ``ruspiro-uart`` crate.
//! 

use crate::alloc::boxed::Box;
use crate::error::Error;

#[derive(Debug)]
pub enum UartErrorType {
    InitializationFailed,
    UartNotInitialized,
    SendDataFailed,
    ReceiveDataFailed,
    ReceiveBufferEmpty,
    ReceiveDataTimeOut,
}

pub struct UartError {
    error_type: UartErrorType,
}

impl UartError {
    pub fn new(error_type: UartErrorType) -> Self {
        Self {
            error_type,
        }
    }
}

impl Error for UartError {}
//unsafe impl Send for UartError {}
//unsafe impl Sync for UartError {}

impl core::fmt::Display for UartError {
    /// Provide the human readable text for this error
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?} Error in Uart", self.error_type)
    }
}

impl core::fmt::Debug for UartError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // debug just calls the diplay implementation
        <UartError as core::fmt::Display>::fmt(self, f)
    }
}