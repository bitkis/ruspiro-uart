/***********************************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-uart/0.4.0")]
#![no_std]
#![feature(asm)]
//! # UART API for Raspberry Pi
//!
//! This crate provides access to the Uart0 (PL011) and the Uart1 (miniUART) peripheral of the Raspberry Pi. It is quite
//! helpful during bare metal development to use a terminal console connected to the miniUART of the Raspberry Pi to get
//! some debug information printed while the program is executed on the device. Especialy if the program is in a state
//! where there is no other output option or blinking LEDs are not sufficient.
//!
//! # Example
//!
//! The proposed usage of the UART is to attach it to a generic console as an output channel instead of using it directly.
//! To do so, please refer to the [``ruspiro-console`` crate](https://crates.io/crates/ruspiro-console).
//!
//! But in case you would like to use the uart without the console abstraction it is recommended to wrap it into a singleton
//! to guaranty safe cross core access and ensure only one time initialization. In the example we pass a fixed core clock rate to
//! the initialization function. However, the real core clock rate could be optained with a call to the mailbox property
//! tag interface of the Raspberry Pi (see [``ruspiro-mailbox`` crate](https://crates.io/crates/ruspiro-mailbox) for details.).
//!
//! ```no_run
//! use ruspiro_singleton::Singleton; // don't forget the dependency to be setup in ``Cargo.toml``
//! use ruspiro_uart::Uart1;
//!
//! static UART: Singleton<Uart1> = Singleton::new(Uart1::new());
//!
//! fn main() {
//!     let _ = UART.take_for(|uart| uart.initialize(250_000_000, 115_200));
//!     // initialize(...) gives a [Result], you may want to panic if there is an Error returned.
//!
//!     print_something("Hello Uart...");
//! }
//!
//! fn print_something(s: &str) {
//!     UART.take_for(|uart| uart.send_string(s));
//! }
//! ```
extern crate alloc;
use ruspiro_core::*;

pub mod errors;

pub mod uart0;
#[doc(inline)]
pub use uart0::*;

pub mod uart1;
#[doc(inline)]
pub use uart1::*;

type UartResult<T> = Result<T, &'static str>;

/// The different types of interrupts that can be raised from an Uart peripheral.
#[repr(u8)]
pub enum InterruptType {
    Receive,
    Transmit,
    RecieveTransmit,
}