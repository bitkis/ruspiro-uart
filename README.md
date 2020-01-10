# UART RusPiRo crate

This crate provides access to the Uart0(Pl011) and Uart1 (miniUART) peripheral of the Raspberry Pi. This is quite helpful during bare metal
development to use a terminal console connected to the miniUART of the Raspberry Pi to get some debug information printed
while the program is executed on the device. Especialy if the program is in a state where there is no other output option and blinking
LEDs are not sufficient.

[![Travis-CI Status](https://api.travis-ci.org/RusPiRo/ruspiro-uart.svg?branch=master)](https://travis-ci.org/RusPiRo/ruspiro-uart)
[![Latest Version](https://img.shields.io/crates/v/ruspiro-uart.svg)](https://crates.io/crates/ruspiro-uart)
[![Documentation](https://docs.rs/ruspiro-uart/badge.svg)](https://docs.rs/ruspiro-uart)
[![License](https://img.shields.io/crates/l/ruspiro-uart.svg)](https://github.com/RusPiRo/ruspiro-uart#license)

## Usage
To use the crate just add the following dependency to your ``Cargo.toml`` file:
```
[dependencies]
ruspiro-uart = { version = "0.3", features = ["ruspiro_pi3"] }
```

Once done the access to the UART abstraction is available in your rust files like so:
```
use ruspiro_uart::Uart1;

fn demo() {
    let mut uart = Uart1::new();
    if uart.initialize(250_000_000, 115_200).is_ok() {
        uart.send_string("This is some string");
    }
}
```

In this example the Uart1 will be no longer be available once it goes out of scope. Whichs makes it a bit cumbersome
to use it in a real world example. Therefore the proposed usage of the UART is to use it as a generic console output
channel. To do so, please refer to the [ruspiro-console crate](https://crates.io/crates/ruspiro-console).
But in case you would like to use the uart without the console abstraction it is recommended to wrap it into a singleton
to guaranty safe cross core access and only 1 time initialization. In the example we pass a fixed core clock rate to
the initialization function. However, the real core clock rate could be optained with a call to the mailbox property
tag interface of the Raspberry Pi (see [ruspiro-mailbox](https://crates.io/crates/ruspiro-mailbox) for details.). This
mailbox crate is not linked into the Uart crate to ensure usability of this crate with as less dependencies as possible.

```
use ruspiro_singleton::Singleton; // don't forget the dependency to be setup
use ruspiro_uart::Uart1;

static UART: Singleton<Uart1> = Singleton::new(Uart1::new());

fn demo() {
    let _ = UART.take_for(|uart| uart.initialize(250_000_000, 115_200)).expect("unable to init uart1");

    print_something("Hello Uart...");
}

fn print_something(s: &str) {
    UART.take_for(|uart| uart.send_string(s));
}
```

## License
Licensed under Apache License, Version 2.0, ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)