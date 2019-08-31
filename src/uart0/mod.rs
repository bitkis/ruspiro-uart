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

use ruspiro_console::*;

mod interface;

/// Uart0 peripheral representation
pub struct Uart0 {
    initialized: bool,
}

impl Uart0 {
    // get a new Uart0 instance
    pub const fn new() -> Self {
        Uart0 {
            initialized: false,
        }
    }

    /// Initialize the Uart0 peripheral for usage. It takes the UART clock rate and the
    /// baud rate to configure correct communication speed. Please not that in the current version the initialization
    /// of the Uart0 will use the GPIO pins 32 and 33 to configure the bridge to the on-board bluetooth low energy chip.
    /// 
    /// # Example
    /// ```
    /// # fn demo() {
    /// let mut uart = Uart1::new();
    /// assert_eq!(uart.initialize(3_000_000, 115_200), Ok(()));
    /// # }
    /// ```
    /// 
    pub fn initialize(&mut self, clock_rate: u32, baud_rate: u32) -> Result<(), &'static str> {
        interface::init(clock_rate, baud_rate)
            .map(|_| {
                self.initialized = true;
            })
    }

    /// Write the byte buffer to the Uart0 transmit buffer/fifo which inturn will send the data to any connected device. In the current setup
    /// this is the BLE chip.
    /// # Example
    /// ```
    /// # fn doc() {
    /// # let uart = Uart0::new();
    /// # let _ = uart.initialize(3_000_000, 115_200);
    /// let data: [u8; 4] = [1, 15, 20, 10];
    /// uart.write_data(&data);
    /// # }
    /// ```
    /// 
    pub fn write_data(&self, data: &[u8]) {
        if self.initialized {
            for d in 0..data.len() {
                interface::write_byte(data[d]);
            }
        }
    }

    /// Read one byte from the Uart0 receive buffer/Fifo if available.
    /// # Example
    /// ```
    /// # fn doc() {
    /// # let uart = Uart0::new();
    /// # let _ = uart.initialize(3_000_000, 115_200);
    /// if let Some(data) = uart.read_data() {
    ///     println!("received {}", data);
    /// }
    /// # }
    /// ```
    pub fn read_data(&self) -> Option<u8> {
        if self.initialized {
            interface::read_byte()
        } else {
            None
        }
    }
}

// When the Uart0 is dropped it should release the GPIO pins that have been aquired.
impl Drop for Uart0 {
    fn drop(&mut self) {
        // release the GPIO pin's occupied by the Uart0
        interface::release();
    }
}

// to use the Uart0 as a console to output strings implement the respective trait
impl ConsoleImpl for Uart0 {
    fn putc(&self, c: char) {
        let data: [u8;1] = [c as u8];
        self.write_data(&data);
    }

    fn puts(&self, s: &str) {
        self.write_data(s.as_bytes());
    }
}
