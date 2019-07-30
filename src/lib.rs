/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-uart/0.0.2")]
#![no_std]

//! # UART API for Raspberry Pi
//! 
//! This crate provides access to the Uart0 (miniUART) peripheral of the Raspberry Pi. This is quite helpful during bare metal
//! development to use an terminal console connected to the miniUART of the Raspberry Pi to get some debug information printed
//! while the program is executed on the device. Especialy if the program is in a state where there is no other output option.
//! 
//! # Usage
//! 
//! The UART function can be accessed through a singleton representation like so:
//! ```
//! use ruspiro_uart::Uart0;
//! 
//! fn demo() {
//!     let mut uart = Uart0::new();
//!     if uart.initialize(25_000_000, 115_200).is_ok() {
//!         uart.send_string("This is some string");
//!     }
//! }
//! ```
//! 
//! In this example the Uart0 will be no longer available once it goes out of scope. Whichs makes it a bit cumbersome
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
//! use ruspiro_uart::Uart0;
//! 
//! static UART: Singleton<Uart0> = Singleton::new(Uart0::new());
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

use ruspiro_console::ConsoleImpl;

mod interface;

// Peripheral MMIO base address - depends on the target family (Pi2 or 3)
#[cfg(not(target_family="ruspiro-rpi3"))]
const PERIPHERAL_BASE: u32 = 0x2000_0000;

#[cfg(target_family="ruspiro-rpi3")]
const PERIPHERAL_BASE: u32 = 0x3F00_0000;

// UART0 MMIO base address
const UART0_BASE: u32 =  PERIPHERAL_BASE + 0x0021_5000;

/// Uart0 (miniUART) peripheral representation
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

    /// Initialize the Uart0 peripheral for usage. It takes the core clock rate and the
    /// baud rate to configure correct speed.
    pub fn initialize(&mut self, clock_rate: u32, baud_rate: u32) -> Result<(), &'static str> {
        interface::uart0_init(clock_rate, baud_rate)
            .map(|_| { self.initialized = true; } )
    }

    pub fn send_char(&self, c: char) {
        if self.initialized {
            interface::uart0_send_char(c);
        }
    }

    pub fn send_string(&self, s: &str) {
        if self.initialized {
            interface::uart0_send_string(s);
        }
    }
}

impl Drop for Uart0 {
    fn drop(&mut self) {
        // ensure the Uart0 peripheral is released once this instance is dropped
        interface::uart0_release();
    }
}

// to use the Uart0 as a console to output strings implement the respective trait
impl ConsoleImpl for Uart0 {
    fn putc(&self, c: char) {
        self.send_char(c);
    }

    fn puts(&self, s: &str) {
        self.send_string(s);
    }
}