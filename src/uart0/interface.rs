/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **********************************************************************************************************************/

//! # Low-Level Uart0 interface implementation
//! 

use ruspiro_gpio::GPIO;
use ruspiro_register::{define_registers, RegisterFieldValue};
use ruspiro_timer as timer;

use crate::UartResult;

// Peripheral MMIO base address - depends on the right feature
#[cfg(feature="ruspiro_pi3")]
const PERIPHERAL_BASE: u32 = 0x3F00_0000;

// UART0 MMIO base address
const UART0_BASE: u32 =  PERIPHERAL_BASE + 0x0020_1000;


/// Initialize the Uart0 based on the given core rate and baud rate.
/// For the time beeing the Uart0 will be bridged to the Raspberry Pi
/// bluetooth chip.
/// TODO: enable the GPIO pins to be used to be passed from outside
///       Is there a way to do some compile time checks, that only valid pins
///       are passed?
pub(crate) fn init(clock_rate: u32, baud_rate: u32) -> UartResult<()> {
    GPIO.take_for(|gpio| {
        gpio.get_pin(32).map(|pin| pin.to_alt_f3())?;
        gpio.get_pin(33).map(|pin| pin.to_alt_f3())?;
        Ok(())
    }).and_then(|_| {
        let baud16:u32 = baud_rate*16;
        let int_div:u32 = clock_rate / baud16;
        let frac_div2 = (clock_rate % baud16) * 8 / baud_rate;
        let frac_div = (frac_div2 / 2) + (frac_div2 % 2);

        // configure UART0
        UART0_CR::Register.set(0);
        UART0_IMSC::Register.set(0x0);
        UART0_ICR::Register.set(0x7FF);
        UART0_IBRD::Register.set(int_div);
        UART0_FBRD::Register.set(frac_div);
        UART0_IFLS::Register.write(UART0_IFLS::RXIFSEL, Ifsel::Filled_1_8 as u32);
        UART0_LCRH::Register.write_value(
            RegisterFieldValue::<u32>::new(UART0_LCRH::WLEN, Wlen::DataLen8 as u32)
            | RegisterFieldValue::<u32>::new(UART0_LCRH::FEN, 0x1)
        );
        UART0_CR::Register.write_value(
            RegisterFieldValue::<u32>::new(UART0_CR::UART_EN, 0x1) |
            RegisterFieldValue::<u32>::new(UART0_CR::TXE, 0x1) |
            RegisterFieldValue::<u32>::new(UART0_CR::RXE, 0x1)
        );

        UART0_IMSC::Register.write_value(
            RegisterFieldValue::<u32>::new(UART0_IMSC::INT_RX, 0x1) |
            RegisterFieldValue::<u32>::new(UART0_IMSC::INT_RT, 0x1) |
            RegisterFieldValue::<u32>::new(UART0_IMSC::INT_OE, 0x1)
        );
        
        // UART0 is now ready to be used
        Ok(())
    })
}

pub(crate) fn release() {
    GPIO.take_for(|gpio| {
        gpio.free_pin(32);
        gpio.free_pin(33);
    });
}

pub(crate) fn write_byte(data: u8) {
    // wait until Uart0 is ready to accept writes
    while UART0_FR::Register.read(UART0_FR::TXFF) == 1 { timer::sleepcycles(10); }
    UART0_DR::Register.set(data as u32);
}

pub(crate) fn read_byte() -> Option<u8> {
    /*if UART0_FR::Register.read(UART0_FR::RXFE) == 1 {
        None
    } else {
        Some((UART0_DR::Register.get() & 0xFF) as u8)
    }*/
    while UART0_FR::Register.read(UART0_FR::RXFE) == 1 { timer::sleepcycles(10); }
    Some((UART0_DR::Register.get() & 0xFF) as u8)
}

#[allow(dead_code, non_camel_case_types)]
enum Ifsel {
    Filled_1_8 = 0,
    Filled_1_4 = 1,
    Filled_1_2 = 2,
    Filled_3_4 = 3,
    Filled_7_8 = 4
}

#[allow(dead_code)]
enum Wlen {
    DataLen8    = 3,
    DataLen7    = 2,
    DataLen6    = 1,
    DataLen5    = 0
}

define_registers![
    UART0_DR:       ReadWrite<u32> @ UART0_BASE + 0x00,
    UART0_RSRECR:   ReadWrite<u32> @ UART0_BASE + 0x04,
    UART0_FR:       ReadWrite<u32> @ UART0_BASE + 0x18 => [
        TXFE    OFFSET(7),
        RXFF    OFFSET(6),
        TXFF    OFFSET(5),
        RXFE    OFFSET(4),
        BUSY    OFFSET(3)
    ],
    UART0_IBRD:     ReadWrite<u32> @ UART0_BASE + 0x24,
    UART0_FBRD:     ReadWrite<u32> @ UART0_BASE + 0x28,
    UART0_LCRH:     ReadWrite<u32> @ UART0_BASE + 0x2C => [
        SPS     OFFSET(7),
        WLEN    OFFSET(5) BITS(2),
        FEN     OFFSET(4),
        STP2    OFFSET(3),
        EPS     OFFSET(2),
        PEN     OFFSET(1),
        BRK     OFFSET(0)
    ],
    UART0_CR:       ReadWrite<u32> @ UART0_BASE + 0x30 => [
        CTSEN   OFFSET(15),
        RTSEN   OFFSET(14),
        OUT2    OFFSET(13),
        OUT1    OFFSET(12),
        RTS     OFFSET(11),
        DTR     OFFSET(10),
        RXE     OFFSET(9),
        TXE     OFFSET(8),
        LBE     OFFSET(7),
        UART_EN OFFSET(0)
    ],
    UART0_IFLS:     ReadWrite<u32> @ UART0_BASE + 0x34 => [
        RXIFSEL OFFSET(3) BITS(3),
        TXIFSEL OFFSET(0) BITS(3)
    ],
    UART0_IMSC:     ReadWrite<u32> @ UART0_BASE + 0x38 => [
        INT_OE      OFFSET(10), // Overrun error
        INT_BE      OFFSET(9),
        INT_PE      OFFSET(8),
        INT_FE      OFFSET(7),
        INT_RT      OFFSET(6), // receive timeout means: FIFO is not empty and no more data is received during a 32bit period
        INT_TX      OFFSET(5), // transit FiFo reached water mark
        INT_RX      OFFSET(4), // receive FiFo reached water mark
        INT_DSRM    OFFSET(3),
        INT_DCDM    OFFSET(2),
        INT_CTSM    OFFSET(1)     
    ],
    UART0_RIS:      ReadWrite<u32> @ UART0_BASE + 0x3C,
    UART0_MIS:      ReadWrite<u32> @ UART0_BASE + 0x40,
    UART0_ICR:      ReadWrite<u32> @ UART0_BASE + 0x44
];