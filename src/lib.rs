/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-uart/0.0.1")]
#![no_std]

//! # UART
//! 
use ruspiro_singleton::Singleton;

mod interface;

// Peripheral MMIO base address
const PERIPHERAL_BASE: u32 = 0x3F00_0000;

// UART0 MMIO base address
const UART0_BASE: u32 =  PERIPHERAL_BASE + 0x0021_5000;

/// UART0 accessor wrapped into a singleton as only one of it could
/// exists.
pub static UART0: Singleton<Uart0> = Singleton::new(Uart0::new());

pub struct Uart0 {
    initialized: bool,
}

impl Uart0 {
    // get a new Uart0 instance
    pub(crate) const fn new() -> Self {
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