#![deny(unsafe_code)]
#![deny(warnings)]
#![no_std]

mod font;
mod pcd8544_gpio;
mod pcd8544_spi;

pub use pcd8544_gpio::Pcd8544Gpio;
pub use pcd8544_spi::Pcd8544Spi;

const DISPLAY_WIDTH: usize = 84;
const DISPLAY_HEIGHT: usize = 6;
const BUFFER_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;

pub trait Pcd8544 {
    fn command(&mut self, command: u8);
    fn data(&mut self, data: &[u8]);

    fn init(&mut self) {
        // chip active (PD=0); horizontal addressing mode (V = 0); use extended instruction set (H = 1)
        self.command(0x21);
        // try 0xB1 (for 3.3V red SparkFun), 0xB8 (for 3.3V blue SparkFun), 0xBF if your display is too dark, or 0x80 to 0xFF if experimenting
        self.command(0xB1);
        // temp coefficient (0)
        self.command(0x04);
        // LCD bias mode 1:48
        self.command(0x13);

        // we must send 0x20 before modifying the display control mode
        self.command(0x20);
        // set display control to normal mode (pixels are on when data is 1), inverse mode=0x0D
        self.command(0x0C);

        self.clear();
    }

    fn print_char(&mut self, c: u8) {
        let i = (c as usize) - 0x20;

        self.data(&font::ASCII[i]);
        self.data(&[0x00]);
    }

    fn print(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e => self.print_char(byte),
                // not part of printable ASCII range
                _ => self.print_char(b'?'),
            }
        }
    }

    fn set_position(&mut self, x: u8, y: u8) {
        assert!(usize::from(x) < DISPLAY_WIDTH);
        assert!(usize::from(y) < DISPLAY_HEIGHT);

        self.command(0x40 + y);
        self.command(0x80 + x);
    }

    // note: data direction is vertical: [1 2 3 4 5 6]
    // 1 3 5
    // 2 4 6
    fn draw_buffer(&mut self, buffer: &[u8; 6 * 84]) {
        self.command(0x22); // vertical addressing
        self.set_position(0, 0);
        self.data(buffer);
        self.command(0x20); // horizontal addressing
        self.set_position(0, 0);
    }

    fn clear(&mut self) {
        self.set_position(0, 0);
        self.data(&[0u8; BUFFER_SIZE]);
        self.set_position(0, 0);
    }
}
