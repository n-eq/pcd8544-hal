# PC8544

This Rust library provides [embedded-hal](https://github.com/rust-embedded/embedded-hal) support for PCD8544-based LCD displays,
commonly used in Nokia 5110 and similar devices. \
_It is based on [the work](https://github.com/dancek/pcd8544-hal/) by Hannu Hartikainen._

## Features

- [x] 100% safe
- [x] SPI support
- [x] GPIO support
- [ ] Vertical scrolling (up/down)
- [ ] Special characters support (line feed, carriage return)

## Model support

Any microcontroller HAL with embedded-hal support should work with this driver

## Usage
Add this line to your `Cargo.toml`'s `[dependencies]` section:
```toml
[dependencies]
pcd8544 = "0.1.0"
```

An example of using the library with an Arduino Uno is provided under [examples/arduino-uno-pcd8544](examples/arduino-uno-pcd8544).

