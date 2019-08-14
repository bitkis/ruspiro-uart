/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-uart/0.2.0")]
#![no_std]

//! # UART API for Raspberry Pi
//! 
//! This crate provides access to the Uart1 (miniUART) peripheral of the Raspberry Pi. This is quite helpful during bare metal
//! development to use an terminal console connected to the miniUART of the Raspberry Pi to get some debug information printed
//! while the program is executed on the device. Especialy if the program is in a state where there is no other output option.
//! 
//! # Usage
//! 
//! The UART function can be accessed through a singleton representation like so:
//! ```
//! use ruspiro_uart::Uart1;
//! 
//! fn demo() {
//!     let mut uart = Uart1::new();
//!     if uart.initialize(250_000_000, 115_200).is_ok() {
//!         uart.send_string("This is some string");
//!     }
//! }
//! ```
//! 
//! In this example the Uart1 will be no longer available once it goes out of scope. Whichs makes it a bit cumbersome
//! to use in a real world example. Therefore the proposed usage of the UART is to attach it to a generic console as an 
//! output channel. To do so, please refer to the [ruspiro-console crate](https://crates.io/crates/ruspiro-console).
//! But in case you would like to use the uart without the console abstraction it is recommended to wrap it into a singleton
//! to guaranty safe cross core access and only 1 time initialization. In the example we pass a fixed core clock rate to
//! the initialization function. However, the real core clock rate could be optained with a call to the mailbox property
//! tag interface of the Raspberry Pi (see [ruspiro-mailbox crate](https://crates.io/crates/ruspiro-mailbox) for details.). This
//! mailbox crate is not linked into the Uart crate to ensure usability of this crate with as less dependencies as possible.
//! 
//! ```
//! use ruspiro_singleton::Singleton; // don't forget the dependency to be setup
//! use ruspiro_uart::Uart1;
//! 
//! static UART: Singleton<Uart1> = Singleton::new(Uart1::new());
//! 
//! fn demo() {
//!     let _ = UART.take_for(|uart| uart.initialize(250_000_000, 115_200)); // initialize(...) gives a Result, you may want to panic if there is an Error returned.
//! 
//!     print_something("Hello Uart...");
//!     let t = 5;
//!     t = 6;
//! }
//! 
//! fn print_something(s: &str) {
//!     UART.take_for(|uart| uart.send_string(s));
//! }
//! ```

extern crate alloc;
use alloc::vec::Vec;

use ruspiro_console::ConsoleImpl;

mod interface;

#[repr(u8)]
pub enum InterruptType {    
    Receive,
    Transmit,
    RecieveTransmit,
}

/// Uart1 (miniUART) peripheral representation
pub struct Uart1 {
    initialized: bool,
}

impl Uart1 {
    // get a new Uart1 instance
    pub const fn new() -> Self {
        Uart1 {
            initialized: false,
        }
    }

    /// Initialize the Uart1 peripheral for usage. It takes the core clock rate and the
    /// baud rate to configure correct speed.
    /// # Example
    /// ```
    /// # fn demo() {
    /// let mut uart = Uart1::new();
    /// assert_eq!(uart.initialize(250_000_000, 115_200), Ok(()));
    /// # }
    /// ```
    /// 
    pub fn initialize(&mut self, clock_rate: u32, baud_rate: u32) -> Result<(), &'static str> {
        interface::uart1_init(clock_rate, baud_rate)
            .map(|_| { self.initialized = true; } )
    }

    /// Send a single character to the uart peripheral
    /// # Example
    /// ```
    /// # fn demo() {
    /// # let mut uart = Uart1::new();
    /// uart.send_char('A');
    /// # }
    /// ```
    /// 
    pub fn send_char(&self, c: char) {
        if self.initialized {
            interface::uart1_send_char(c);
        }
    }

    /// Send a string to the uart peripheral
    /// # Example
    /// ```
    /// # fn demo() {
    /// # let mut uart = Uart1::new();
    /// uart.send_string("Test string with line break\r\n");
    /// # }
    /// ```
    /// 
    pub fn send_string(&self, s: &str) {
        if self.initialized {
            interface::uart1_send_string(s);
        }
    }

    /// Try to recieve data from the Uart of the given size
    /// If the requested size could be read it returns a ``Ok(data: Vec<u8>)`` containing the data
    /// otherwise an ``Err(msg: &str)``.
    /// 
    /// # Example
    /// ```
    /// # fn demo() {
    /// # let uart = Uart1::new()
    /// let data = uart.try_receive_data(8).expect("unable to receive 8 bytes");
    /// # }
    /// ```
    /// 
    pub fn try_receive_data(&self, size: usize) -> Result<Vec<u8>, &'static str> {
        if self.initialized {
            let data = interface::uart1_receive_data(size, false);
            if data.len() < size {
                Err("unable to receive enough data from uart")
            } else {
                Ok(data)
            }
        } else {
            // if Uart is not initialized return 0 size vector or error?
            Err("Uart not initialized")
        }
    }

    pub fn enable_interrupts(&self, i_type: InterruptType) {
        if self.initialized {
            interface::uart1_enable_interrupts(i_type);
        }
    }

    pub fn disable_interrupts(&self, i_type: InterruptType) {
        if self.initialized {
            interface::uart1_disable_interrupts(i_type);
        }
    }

    pub fn acknowledge_interrupt(&self) {
        if self.initialized {
            interface::uart1_acknowledge_interrupt();
        }
    }
}

impl Drop for Uart1 {
    fn drop(&mut self) {
        // ensure the Uart1 peripheral is released once this instance is dropped
        interface::uart1_release();
    }
}

// to use the Uart1 as a console to output strings implement the respective trait
impl ConsoleImpl for Uart1 {
    fn putc(&self, c: char) {
        self.send_char(c);
    }

    fn puts(&self, s: &str) {
        self.send_string(s);
    }
}