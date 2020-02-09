/***********************************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **********************************************************************************************************************/

//! # Uart0 (Pl011) API
//!
//! This is a more fully featured Uart peripheral. In the Raspberry Pi this is most likely configured to act as
//! communication bridge to other peripherals like the buit in bluetooth low energy chip.
//!

use crate::alloc::boxed::Box;
use crate::error::*;
use crate::errors::{UartError, UartErrorType::*};
use crate::ConsoleImpl;
use ruspiro_interrupt::{Interrupt, InterruptManager, IRQ_MANAGER};
mod interface;

/// Uart0 peripheral representation
pub struct Uart0 {
    initialized: bool,
}

impl Uart0 {
    /// get a new Uart0 instance
    pub const fn new() -> Self {
        Uart0 { initialized: false }
    }

    /// Initialize the Uart0 peripheral for usage. It takes the UART clock rate and the
    /// baud rate to configure correct communication speed. Please not that in the current version the initialization
    /// of the Uart0 will use the GPIO pins 32 and 33 to configure the bridge to the on-board bluetooth low energy chip.
    ///
    /// # Example
    /// ```no_run
    /// # use ruspiro_uart::uart0::*;
    /// # fn doc() {
    /// let mut uart = Uart0::new();
    /// assert_eq!(uart.initialize(3_000_000, 115_200), Ok(()));
    /// # }
    /// ```
    pub fn initialize(&mut self, clock_rate: u32, baud_rate: u32) -> Result<(), BoxError> {
        interface::init(clock_rate, baud_rate).map(|_| {
            self.initialized = true;
        })
    }

    /// Write the byte buffer to the Uart0 transmit buffer/fifo which inturn will send the data to any connected device. In the current setup
    /// this is the BLE chip.
    /// # Example
    /// ```no_run
    /// # use ruspiro_uart::uart0::*;
    /// # fn doc() {
    /// # let mut uart = Uart0::new();
    /// # let _ = uart.initialize(3_000_000, 115_200);
    /// let data: [u8; 4] = [1, 15, 20, 10];
    /// uart.send_data(&data);
    /// # }
    /// ```
    pub fn send_data(&self, data: &[u8]) -> Result<(), BoxError> {
        if self.initialized {
            for byte in data {
                interface::send_byte(*byte);
            }
            Ok(())
        } else {
            Err(
                Box::new(UartError::new(UartNotInitialized))
            )
        }
    }

    /// Read one byte from the Uart0 receive buffer/Fifo if available.
    /// # Example
    /// ```no_run
    /// # use ruspiro_uart::uart0::*;
    /// # fn doc() {
    /// # let mut uart = Uart0::new();
    /// # let _ = uart.initialize(3_000_000, 115_200);
    /// let mut buffer: [u8; 5] = [0; 5];
    /// if uart.receive_data(&mut buffer).is_ok() {
    ///     println!("received {:X?}", buffer);
    /// }
    /// # }
    /// ```
    pub fn receive_data(&self, buffer: &mut [u8]) -> Result<usize, BoxError> {
        if self.initialized {
            if buffer.is_empty() {
                Err(Box::new(UartError::new(ReceiveBufferEmpty)))//"buffer size expected to be at least 1")
            } else {
                for data in &mut *buffer {
                    *data = interface::receive_byte()?;
                }
                Ok(buffer.len())
            }
        } else {
            Err(Box::new(UartError::new(UartNotInitialized)))//"Uart0 not initialized")
        }
    }

    /// Register a callback function / closure to be execuded whenever an Uart0 related
    /// interrupt is raised. This will also activate the intterrupts for Uart0 to be dispatched
    /// by the global interrupt manager
    pub fn register_irq_handler<F: FnMut() + 'static + Send>(&self, function: F) {
        interface::set_irq_handler(function);
        IRQ_MANAGER.take_for(|mgr: &mut InterruptManager| mgr.activate(Interrupt::Pl011));
    }
}

/// When the Uart0 is dropped it should release the GPIO pins that have been aquired.
impl Drop for Uart0 {
    fn drop(&mut self) {
        // release the GPIO pin's occupied by the Uart0
        interface::release();
    }
}

/// to use the Uart0 as a console to output strings implement the respective trait
impl ConsoleImpl for Uart0 {
    fn putc(&self, c: char) {
        let data: [u8; 1] = [c as u8];
        self.send_data(&data);
    }

    fn puts(&self, s: &str) {
        self.send_data(s.as_bytes());
    }
}
