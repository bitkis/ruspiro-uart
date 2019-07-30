/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **********************************************************************************************************************/

//! # Low-Level UART interface implementation
//! 

use ruspiro_register::define_registers;
use ruspiro_gpio::GPIO;

// initialize the UART0 peripheral of the Raspberry Pi3. This will reserve 2 GPIO pins for UART0 usage.
// Those pins actually are GPIO14 and 15.
pub(crate) fn uart0_init(clock_rate: u32, baud_rate: u32) -> Result<(), &'static str> {
    
    GPIO.take_for(|gpio| {
        let maybe_tx = gpio.get_pin(14).map(|pin| pin.to_alt_f5().to_pud_disabled());
        let maybe_ty = gpio.get_pin(15).map(|pin| pin.to_alt_f5().to_pud_disabled());

        // returns OK only if both pins could be setup correctly
        maybe_tx.and(maybe_ty)
    }).map(|_| {
        AUX_ENABLES::Register.set(0x1); // enable mini UART
        AUX_MU_IER_REG::Register.set(0x0); // disable interrupts
        AUX_MU_CNTL_REG::Register.set(0x0); // disable transmitter (to set new baud rate)
        AUX_MU_LCR_REG::Register.set(0x3); // set 8bit data transfer mode
        AUX_MU_MCR_REG::Register.set(0x0); // set UART_RTS line to high (ready to send)
        AUX_MU_IER_REG::Register.set(0x0); // disable interrupts
        AUX_MU_IIR_REG::Register.set(0xC6); // clear recieve/transmit FIFO, set FIFO as always enabled
        AUX_MU_BAUD_REG::Register.set(clock_rate/(8*baud_rate)-1); // set the baud rate based on the core clock rate

        AUX_MU_CNTL_REG::Register.set(0x3); // enable transmitter
    })
}

// release the UART0 peripheral, this will also free the pins reserved for UART0 till now
pub(crate) fn uart0_release() {
    GPIO.take_for(|gpio| {
        gpio.free_pin(14);
        gpio.free_pin(15);
    });
}

// send a character string to the UART0 peripheral
pub(crate) fn uart0_send_char(c: char) {
    let data: [u8;1] = [c as u8];
    uart0_send_data(&data);
}

// send a character string to the UART0 peripheral
pub(crate) fn uart0_send_string(s: &str) {
    uart0_send_data(s.as_bytes());
}

// send byte data to the UART0 peripheral
fn uart0_send_data(data: &[u8]) {
    for byte in data {
        // wait for the transmitter to be empty
        while AUX_MU_LSR_REG::Register.read(AUX_MU_LSR_REG::TRANSEMPTY) == 0 { }
        AUX_MU_IO_REG::Register.set(*byte as u32);
    }
}

use super::UART0_BASE as UART0_BASE;

// specify the UART0 registers
define_registers! [
    AUX_IRQ:          ReadWrite<u32> @ UART0_BASE + 0x00 => [],
    AUX_ENABLES:      ReadWrite<u32> @ UART0_BASE + 0x04 => [],
    AUX_MU_IO_REG:    ReadWrite<u32> @ UART0_BASE + 0x40 => [],
    AUX_MU_IER_REG:   ReadWrite<u32> @ UART0_BASE + 0x44 => [],
    AUX_MU_IIR_REG:   ReadWrite<u32> @ UART0_BASE + 0x48 => [],
    AUX_MU_LCR_REG:   ReadWrite<u32> @ UART0_BASE + 0x4C => [],
    AUX_MU_MCR_REG:   ReadWrite<u32> @ UART0_BASE + 0x50 => [],
    AUX_MU_LSR_REG:   ReadOnly<u32> @ UART0_BASE + 0x54 => [
        DATAREADY  OFFSET(0) BITS(0),
        RCVOVERRUN OFFSET(1) BITS(1),
        TRANSEMPTY OFFSET(5) BITS(1),
        TRANSIDLE  OFFSET(6) BITS(1)
    ],
    AUX_MU_MSR_REG:   ReadWrite<u32> @ UART0_BASE + 0x58 => [],
    AUX_MU_CNTL_REG:  ReadWrite<u32> @ UART0_BASE + 0x60 => [],
    AUX_MU_STAT_REG:  ReadWrite<u32> @ UART0_BASE + 0x64 => [],
    AUX_MU_BAUD_REG:  ReadWrite<u32> @ UART0_BASE + 0x68 => []
];