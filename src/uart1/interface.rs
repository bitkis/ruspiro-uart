/***********************************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **********************************************************************************************************************/

//! # Low-Level UART interface implementation
//!

use ruspiro_gpio::GPIO;
use ruspiro_register::{define_mmio_register, RegisterFieldValue};
use ruspiro_timer as timer;

use crate::InterruptType;

// Peripheral MMIO base address - depends on the right feature
#[cfg(feature = "ruspiro_pi3")]
const PERIPHERAL_BASE: u32 = 0x3F00_0000;

// AUX MMIO base address
const AUX_BASE: u32 = PERIPHERAL_BASE + 0x0021_5000;

// initialize the UART1 peripheral of the Raspberry Pi3. This will reserve 2 GPIO pins for UART1 usage.
// Those pins actually are GPIO14 and 15.
pub(crate) fn uart1_init(clock_rate: u32, baud_rate: u32) -> Result<(), &'static str> {
    GPIO.take_for(|gpio| {
        gpio.get_pin(14)
            .map(|pin| pin.into_alt_f5().into_pud_disabled())
            .map_err(|_| "GPIO error")?;
        gpio.get_pin(15)
            .map(|pin| pin.into_alt_f5().into_pud_disabled())
            .map_err(|_| "GPIO error")?;
        Ok(())
    })
    .map(|_| {
        AUX_ENABLES::Register.write(AUX_ENABLES::MINIUART_ENABLE, 0x1); // enable mini UART
        AUX_MU_IER_REG::Register.set(0x0); // disable interrupts
        AUX_MU_CNTL_REG::Register.set(0x0); // disable transmitter and receiver (to set new baud rate)
        AUX_MU_LCR_REG::Register.write(AUX_MU_LCR_REG::DATASIZE, 0x3); // set 8bit data transfer mode
        AUX_MU_MCR_REG::Register.set(0x0); // set UART_RTS line to high (ready to send)
        AUX_MU_IER_REG::Register.set(0x0); // disable interrupts
        AUX_MU_IIR_REG::Register //.set(0xC6);
            .write_value(
                RegisterFieldValue::<u32>::new(AUX_MU_IIR_REG::IRQID_FIFOCLR, 0b11)
                    | RegisterFieldValue::<u32>::new(AUX_MU_IIR_REG::FIFO_ENABLES, 0b11),
            ); // clear recieve/transmit FIFO, set FIFO as always enabled
        AUX_MU_BAUD_REG::Register.set(clock_rate / (8 * baud_rate) - 1); // set the baud rate based on the core clock rate

        AUX_MU_CNTL_REG::Register //.set(0x3);
            .write_value(
                RegisterFieldValue::<u32>::new(AUX_MU_CNTL_REG::RCV_ENABLE, 0x1)
                    | RegisterFieldValue::<u32>::new(AUX_MU_CNTL_REG::TRANS_ENABLE, 0x1),
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
    let data: [u8; 1] = [c as u8];
    uart1_send_data(&data);
}

// send a character string to the UART1 peripheral
pub(crate) fn uart1_send_string(s: &str) {
    uart1_send_data(s.as_bytes());
}

// send byte data to the UART1 peripheral
pub(crate) fn uart1_send_data(data: &[u8]) {
    for byte in data {
        // wait for the transmitter to be empty
        while AUX_MU_LSR_REG::Register.read(AUX_MU_LSR_REG::TRANSEMPTY) == 0 {
            timer::sleepcycles(10);
        }
        AUX_MU_IO_REG::Register.set(*byte as u32);
    }
}

// wait to receive 1 byte from uart and return it
// if timeout is > 0 return timeout error if nothing was available for this many time
// timeout is given in multiples of 1000 CPU cycles
pub(crate) fn uart1_receive_data(timeout: u32) -> Result<u8, &'static str> {
    let mut count = 0;
    while AUX_MU_LSR_REG::Register.read(AUX_MU_LSR_REG::DATAREADY) == 0
        && (timeout == 0 || count < timeout)
    {
        timer::sleepcycles(1000);
        count += 1;
    }
    if timeout != 0 && count >= timeout {
        Err("Timeout")
    } else {
        Ok((AUX_MU_IO_REG::Register.get() & 0xFF) as u8)
    }
}

pub(crate) fn uart1_enable_interrupts(i_type: InterruptType) {
    match i_type {
        InterruptType::Receive => {
            AUX_MU_IER_REG::Register.write_value(
                RegisterFieldValue::<u32>::new(AUX_MU_IER_REG::RCV_IRQ, 0b11)
                    | RegisterFieldValue::<u32>::new(AUX_MU_IER_REG::RX_ENABLE, 0x1),
            );
        }
        InterruptType::Transmit => {
            AUX_MU_IER_REG::Register.write_value(
                RegisterFieldValue::<u32>::new(AUX_MU_IER_REG::RCV_IRQ, 0b11)
                    | RegisterFieldValue::<u32>::new(AUX_MU_IER_REG::TX_ENABLE, 0x1),
            );
        }
        InterruptType::RecieveTransmit => {
            AUX_MU_IER_REG::Register.write_value(
                RegisterFieldValue::<u32>::new(AUX_MU_IER_REG::RCV_IRQ, 0b11)
                    | RegisterFieldValue::<u32>::new(AUX_MU_IER_REG::RX_ENABLE, 0x1)
                    | RegisterFieldValue::<u32>::new(AUX_MU_IER_REG::TX_ENABLE, 0x1),
            );
        }
    }
}

pub(crate) fn uart1_disable_interrupts(i_type: InterruptType) {
    match i_type {
        InterruptType::Receive => {
            AUX_MU_IER_REG::Register.write(AUX_MU_IER_REG::RX_ENABLE, 0x0);
        }
        InterruptType::Transmit => {
            AUX_MU_IER_REG::Register.write(AUX_MU_IER_REG::TX_ENABLE, 0x0);
        }
        InterruptType::RecieveTransmit => {
            AUX_MU_IER_REG::Register.set(0x0);
        }
    }
}

pub(crate) fn uart1_get_interrupt_status() -> u32 {
    AUX_MU_IIR_REG::Register.read(AUX_MU_IIR_REG::IRQPENDING)
        | (AUX_MU_IIR_REG::Register.read(AUX_MU_IIR_REG::IRQID_FIFOCLR) << 1)
}

// specify the AUX registers
define_mmio_register! [
    AUX_IRQ<ReadOnly<u32>@(AUX_BASE)>,
    AUX_ENABLES<ReadWrite<u32>@(AUX_BASE + 0x04)> {
        MINIUART_ENABLE OFFSET(0),
        SPI1_ENABLE OFFSET(1),
        SPI2_ENABLE OFFSET(2)
    },
    AUX_MU_IO_REG<ReadWrite<u32>@(AUX_BASE + 0x40)>,
    AUX_MU_IER_REG<ReadWrite<u32>@(AUX_BASE + 0x44)> {
        RX_ENABLE OFFSET(0),
        TX_ENABLE OFFSET(1),
        RCV_IRQ   OFFSET(2) BITS(2) // set always 0b11 if interrupts shall be received
    },
    AUX_MU_IIR_REG<ReadWrite<u32>@(AUX_BASE + 0x48)> {
        IRQPENDING OFFSET(0),
        IRQID_FIFOCLR OFFSET(1) BITS(2),
        FIFO_ENABLES OFFSET(6) BITS(2)
    },
    AUX_MU_LCR_REG<ReadWrite<u32>@(AUX_BASE + 0x4C)> {
        DATASIZE OFFSET(0) BITS(2),
        BREAK OFFSET(6),
        DLAB OFFSET(7)
    },
    AUX_MU_MCR_REG<ReadWrite<u32>@(AUX_BASE + 0x50)>,
    AUX_MU_LSR_REG<ReadOnly<u32>@(AUX_BASE + 0x54)> {
        DATAREADY  OFFSET(0),
        RCVOVERRUN OFFSET(1),
        TRANSEMPTY OFFSET(5),
        TRANSIDLE  OFFSET(6)
    },
    AUX_MU_MSR_REG<ReadWrite<u32>@(AUX_BASE + 0x58)>,
    AUX_MU_CNTL_REG<ReadWrite<u32>@(AUX_BASE + 0x60)> {
        RCV_ENABLE OFFSET(0),
        TRANS_ENABLE OFFSET(1),
        AUTO_FLOW_RTS OFFSET(2),
        AUTO_FLOW_CTS OFFSET(3),
        AUTO_RTS_LEVEL OFFSET(4) BITS(2),
        RTS_ASSERT OFFSET(6),
        CTS_ASSERT OFFSET(7)

    },
    AUX_MU_STAT_REG<ReadWrite<u32>@(AUX_BASE + 0x64)>,
    AUX_MU_BAUD_REG<ReadWrite<u32>@(AUX_BASE + 0x68)>
];
