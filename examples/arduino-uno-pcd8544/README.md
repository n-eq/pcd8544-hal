arduino-uno-pcd8544
===================

Example application for the [PCD8544] LCD driver on an Arduino Uno board.
The code initializes the display using `PCD8544Gpio` struct using pins 3 to 7.
It then draws the Rust logo on the display.


## Requirements
1. Install prerequisites as described in [avr-hal](https://github.com/Rahix/avr-hal#quickstart).

## Wiring

![alt text](fritzing.png "Wiring diagram for the example")

| PIN NAME  | ARDUINO PIN  | RESISTOR VALUE |
|-----------|------------- |----------------|
| RST       | 3            | 10kΩ           |
| CE        | 4            | 1kΩ            |
| DC        | 5            | 10kΩ           |
| DIN       | 6            | 10kΩ           |
| CLK       | 7            | 10kΩ           |
| VCC       | 3V3          | ∅              |
| BL        | 3V3          | 330Ω           |
| GND       | GND          | ∅              |

## Using another AVR-based board

Using an alternative board is simple. You'll need to modify the values in .cargo/config.toml, Ravedude.toml accordingly.
