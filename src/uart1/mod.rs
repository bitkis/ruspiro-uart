/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **********************************************************************************************************************/

//! # Uart1 (miniUart) API
//! 
//! As per the Raspberry Pi peripheral document the miniUART is a lightweight serial communication channel that does only
//! need 3 wires (TX, RX, GND) to be connected to the device. The miniUART is typically used to connect the device to
//! a PC or Mac that runs a terminal console application and is able to display the characters received through this 
//! channel. This allows to pass debug information from the device running the bare metal kernel to improve root cause
//! analysis.
//! 
//! There is no singleton accessor provided for this peripheral as it will be quite likely attached to a ``Console``
//! abstraction that will than **own** this peripheral and should itself providing exclusive access to the inner accessor
//! of the actual device. Please refer to the [``ruspiro-console`` crate](https://crates.io/crates/ruspiro-console).
//! 

extern crate alloc;
use ruspiro_console::ConsoleImpl;
use crate::InterruptType;

mod interface;

/// Uart1 (miniUART) peripheral representation
pub struct Uart1 {
    initialized: bool,
}

impl Uart1 {
    /// Get a new Uart1 instance, that needs to be initialized before it can be used.
    /// # Example
    /// # fn doc() {
    /// let _miniUart = Uart1::new();
    /// # }
    pub const fn new() -> Self {
        Uart1 {
            initialized: false,
        }
    }

    /// Initialize the Uart1 peripheral for usage. It takes the core clock rate and the
    /// baud rate to configure correct communication speed.
    /// # Example
    /// ```
    /// # fn doc() {
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
    /// # fn doc() {
    /// # let mut uart = Uart1::new();
    /// # let _ = uart.initialize(250_000_000, 115_200);
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
    /// # fn doc() {
    /// # let mut uart = Uart1::new();
    /// # let _ = uart.initialize(250_000_000, 115_200);
    /// uart.send_string("Test string with line break\r\n");
    /// # }
    /// ```
    /// 
    pub fn send_string(&self, s: &str) {
        if self.initialized {
            interface::uart1_send_string(s);
        }
    }

    /// Send a byte buffer to the uart peripheral
    /// # Example
    /// ```
    /// # fn doc() {
    /// # let mut uart = Uart1::new();
    /// # let _ = uart.initialize(20_000_000, 115_200);
    /// uart.send_data("SomeData".as_bytes());
    /// #}
    /// ```
    pub fn send_data(&self, d: &[u8]) {
        if self.initialized {
            interface::uart1_send_data(d);
        }
    }

    /// Try to recieve data from the Uart of the given size
    /// If the requested size could be read it returns a ``Ok(data: Vec<u8>)`` containing the data
    /// otherwise an ``Err(msg: &str)``.
    /// 
    /// # Example
    /// ```
    /// # fn doc() {
    /// # let uart = Uart1::new();
    /// # let _ = uart.initialize(250_000_000, 115_200);
    /// let data = uart.try_receive_data(8).expect("unable to receive 8 bytes");
    /// # }
    /// ```
    /// 
    pub fn try_receive_data(&self, buffer: &mut [u8]) -> Result<usize, &'static str> {
        if self.initialized {
            if buffer.len() < 1 {
                Err("buffer size expected to be at least 1")
            } else {
                for c in 0..buffer.len() {
                    buffer[c] = interface::uart1_receive_data(1000)?;
                }
                Ok(buffer.len())
            }
        } else {
            // if Uart is not initialized return 0 size vector or error? For now -> error
            Err("Uart not initialized")
        }
    }

    pub fn receive_data(&self, buffer: &mut [u8]) -> Result<usize, &'static str> {
        if self.initialized {
            if buffer.len() < 1 {
                Err("buffer size expected to be at least 1")
            } else {
                for c in 0..buffer.len() {
                    buffer[c] = interface::uart1_receive_data(0)?;
                }
                Ok(buffer.len())
            }
        } else {
            // if Uart is not initialized return 0 size vector or error? For now -> error
            Err("Uart not initialized")
        }
    }

    /// Enable Interrupts to be triggered by the miniUart. The ``i_type`` specifies the interrupts
    /// that shall be triggered. To receive/handle the interrupts a corresponding interrupt handler need to be
    /// implemented, for example by using the [``ruspiro-interrupt`` crate](https://crates.io/crates/ruspiro-interrupt).
    /// # Example
    /// ```
    /// # fn doc() {
    /// # let uart = Uart1::new();
    /// # let _ = uart.initialize(250_000_000, 115_200);
    /// // enable the interrupt to be triggered when data is recieved by the miniUart
    /// uart.enable_interrupts(InterruptType::Receive);
    /// # }
    pub fn enable_interrupts(&self, i_type: InterruptType) {
        if self.initialized {
            interface::uart1_enable_interrupts(i_type);
        }
    }

    /// Disable Interrupts from beeing triggered by the miniUart. The ``i_type`` specifies the interrupts
    /// that shall disbabled.
    /// # Example
    /// ```
    /// # fn doc() {
    /// # let uart = Uart1::new();
    /// # let _ = uart.initialize(250_000_000, 115_200);
    /// // disable the interrupt to be triggered when data is recieved by the miniUart
    /// uart.disable_interrupts(InterruptType::Receive);
    /// # }
    pub fn disable_interrupts(&self, i_type: InterruptType) {
        if self.initialized {
            interface::uart1_disable_interrupts(i_type);
        }
    }

    /// Read the current interrupt status.
    /// Bit 0 -> is set to 0 if an interrupt is pending
    /// Bit [1:2] -> 01 = transmit register is empty
    ///              10 = recieve register holds valid data
    /// # Example
    /// ```
    /// # fn doc() {
    /// # let uart = Uart1::new();
    /// # let _ = uart.initialize(250_000_000, 115_200);
    /// let irq_status = uart.get_interrupt_status();
    /// if (irq_status & 0b010) != 0 {
    ///     println!("transmit register empty raised");
    /// }
    /// # }
    pub fn get_interrupt_status(&self) -> u32 {
        if self.initialized {
            interface::uart1_get_interrupt_status()
        } else {
            0
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