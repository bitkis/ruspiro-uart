/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **********************************************************************************************************************/

//! # Low-Level UART interface implementation
//! 

extern crate alloc;
use alloc::vec::Vec;

use ruspiro_register::{define_registers, RegisterFieldValue};
use ruspiro_gpio::GPIO;
use crate::InterruptType;

// Peripheral MMIO base address - depends on the right feature
#[cfg(feature="ruspiro_pi3")]
const PERIPHERAL_BASE: u32 = 0x3F00_0000;

// AUX MMIO base address
const AUX_BASE: u32 =  PERIPHERAL_BASE + 0x0021_5000;

// initialize the UART1 peripheral of the Raspberry Pi3. This will reserve 2 GPIO pins for UART1 usage.
// Those pins actually are GPIO14 and 15.
pub(crate) fn uart1_init(clock_rate: u32, baud_rate: u32) -> Result<(), &'static str> {
    
    GPIO.take_for(|gpio| {
        let maybe_tx = gpio.get_pin(14).map(|pin| pin.to_alt_f5().to_pud_disabled());
        let maybe_ty = gpio.get_pin(15).map(|pin| pin.to_alt_f5().to_pud_disabled());

        // returns OK only if both pins could be setup correctly
        maybe_tx.and(maybe_ty)
    }).map(|_| {
        AUX_ENABLES::Register.write(AUX_ENABLES::MINIUART_ENABLE, 0x1); // enable mini UART
        AUX_MU_IER_REG::Register.set(0x0); // disable interrupts
        AUX_MU_CNTL_REG::Register.set(0x0); // disable transmitter and receiver (to set new baud rate)
        AUX_MU_LCR_REG::Register.write(AUX_MU_LCR_REG::DATASIZE, 0x3); // set 8bit data transfer mode
        AUX_MU_MCR_REG::Register.set(0x0); // set UART_RTS line to high (ready to send)
        AUX_MU_IER_REG::Register.set(0x0); // disable interrupts
        AUX_MU_IIR_REG::Register//.set(0xC6);
        .write_value(
            RegisterFieldValue::<u32>::new(AUX_MU_IIR_REG::IRQID_FIFOCLR, 0b11) |
            RegisterFieldValue::<u32>::new(AUX_MU_IIR_REG::FIFO_ENABLES, 0b11)
        ); // clear recieve/transmit FIFO, set FIFO as always enabled
        AUX_MU_BAUD_REG::Register.set(clock_rate/(8*baud_rate)-1); // set the baud rate based on the core clock rate

        AUX_MU_CNTL_REG::Register//.set(0x3);
        .write_value(
            RegisterFieldValue::<u32>::new(AUX_MU_CNTL_REG::RCV_ENABLE, 0x1) |
            RegisterFieldValue::<u32>::new(AUX_MU_CNTL_REG::TRANS_ENABLE, 0x1)
        ); // enable receiver and transmitter
    })
}

// release the UART1 peripheral, this will also free the pins reserved for UART1 till now
pub(crate) fn uart1_release() {
    GPIO.take_for(|gpio| {
        gpio.free_pin(14);
        gpio.free_pin(15);
    });
}

// send a character string to the UART1 peripheral
pub(crate) fn uart1_send_char(c: char) {
    let data: [u8;1] = [c as u8];
    uart1_send_data(&data);
}

// send a character string to the UART1 peripheral
pub(crate) fn uart1_send_string(s: &str) {
    uart1_send_data(s.as_bytes());
}

// send byte data to the UART1 peripheral
fn uart1_send_data(data: &[u8]) {
    for byte in data {
        // wait for the transmitter to be empty
        while AUX_MU_LSR_REG::Register.read(AUX_MU_LSR_REG::TRANSEMPTY) == 0 { }
        AUX_MU_IO_REG::Register.set(*byte as u32);
    }
}

// try to recieve the up to the number given bytes from the uart
// it's up te the caller to check if the requested ammount of data has been
// recieved, or less...
pub(crate) fn uart1_receive_data(up_to: usize, blocking: bool) -> Vec<u8> {
    const TRIES:u32 = 2000u32;
    let mut data = Vec::<u8>::with_capacity(up_to);    
    for _ in 0..up_to {        
        // wait for data beeing received        
        if blocking { 
            while AUX_MU_LSR_REG::Register.read(AUX_MU_LSR_REG::DATAREADY) == 0 { }
        } else {
            let mut count = 0;     
            while (AUX_MU_LSR_REG::Register.read(AUX_MU_LSR_REG::DATAREADY) == 0) && (count < TRIES) { count += 1; } 
            if count >= TRIES {
                return data;
            }
        }
        
        data.push((AUX_MU_IO_REG::Register.get() & 0xFF) as u8);
    }

    data
}

pub(crate) fn uart1_enable_interrupts(i_type: InterruptType) {
    match i_type {
        InterruptType::Receive => {
            AUX_MU_IER_REG::Register.write_value(
                RegisterFieldValue::<u32>::new(AUX_MU_IER_REG::RCV_IRQ, 0b11) |
                RegisterFieldValue::<u32>::new(AUX_MU_IER_REG::RX_ENABLE, 0x1)
            );
        },
        InterruptType::Transmit => {
            AUX_MU_IER_REG::Register.write_value(
                RegisterFieldValue::<u32>::new(AUX_MU_IER_REG::RCV_IRQ, 0b11) |
                RegisterFieldValue::<u32>::new(AUX_MU_IER_REG::TX_ENABLE, 0x1)
            );
        },
        InterruptType::RecieveTransmit => {
            AUX_MU_IER_REG::Register.write_value(
                RegisterFieldValue::<u32>::new(AUX_MU_IER_REG::RCV_IRQ, 0b11) |
                RegisterFieldValue::<u32>::new(AUX_MU_IER_REG::RX_ENABLE, 0x1) |
                RegisterFieldValue::<u32>::new(AUX_MU_IER_REG::TX_ENABLE, 0x1)
            );
        }
    }
}

pub(crate) fn uart1_disable_interrupts(i_type: InterruptType) {
    match i_type {
        InterruptType::Receive => {
            AUX_MU_IER_REG::Register.write(AUX_MU_IER_REG::RX_ENABLE, 0x0);
        },
        InterruptType::Transmit => {
            AUX_MU_IER_REG::Register.write(AUX_MU_IER_REG::TX_ENABLE, 0x0);
        },
        InterruptType::RecieveTransmit => {
            AUX_MU_IER_REG::Register.set(0x0);
        }
    }
}

pub(crate) fn uart1_acknowledge_interrupt() {
    if AUX_IRQ::Register.get() == 0x1 {
        AUX_IRQ::Register.set(0x1);
    }
}

// specify the AUX registers
define_registers! [
    AUX_IRQ:          ReadWrite<u32> @ AUX_BASE + 0x00,
    AUX_ENABLES:      ReadWrite<u32> @ AUX_BASE + 0x04 => [
        MINIUART_ENABLE OFFSET(0),
        SPI1_ENABLE OFFSET(1),
        SPI2_ENABLE OFFSET(2)
    ],
    AUX_MU_IO_REG:    ReadWrite<u32> @ AUX_BASE + 0x40,
    AUX_MU_IER_REG:   ReadWrite<u32> @ AUX_BASE + 0x44 => [
        RX_ENABLE OFFSET(0),
        TX_ENABLE OFFSET(1),
        RCV_IRQ   OFFSET(2) BITS(2) // set always 0b11 if interrupts shall be received
    ],
    AUX_MU_IIR_REG:   ReadWrite<u32> @ AUX_BASE + 0x48 => [
        IRQPENDING OFFSET(0),
        IRQID_FIFOCLR OFFSET(1) BITS(2),
        FIFO_ENABLES OFFSET(6) BITS(2)
    ],
    AUX_MU_LCR_REG:   ReadWrite<u32> @ AUX_BASE + 0x4C => [
        DATASIZE OFFSET(0) BITS(2),
        BREAK OFFSET(6),
        DLAB OFFSET(7)
    ],
    AUX_MU_MCR_REG:   ReadWrite<u32> @ AUX_BASE + 0x50,
    AUX_MU_LSR_REG:   ReadOnly<u32> @ AUX_BASE + 0x54 => [
        DATAREADY  OFFSET(0),
        RCVOVERRUN OFFSET(1),
        TRANSEMPTY OFFSET(5),
        TRANSIDLE  OFFSET(6)
    ],
    AUX_MU_MSR_REG:   ReadWrite<u32> @ AUX_BASE + 0x58,
    AUX_MU_CNTL_REG:  ReadWrite<u32> @ AUX_BASE + 0x60 => [
        RCV_ENABLE OFFSET(0),
        TRANS_ENABLE OFFSET(1),
        AUTO_FLOW_RTS OFFSET(2),
        AUTO_FLOW_CTS OFFSET(3),
        AUTO_RTS_LEVEL OFFSET(4) BITS(2),
        RTS_ASSERT OFFSET(6),
        CTS_ASSERT OFFSET(7)

    ],
    AUX_MU_STAT_REG:  ReadWrite<u32> @ AUX_BASE + 0x64,
    AUX_MU_BAUD_REG:  ReadWrite<u32> @ AUX_BASE + 0x68
];