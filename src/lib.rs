#![deny(unsafe_code)]
// #![deny(warnings)]
#![no_std]

mod font;
mod pcd8544_gpio;
mod pcd8544_spi;

pub use pcd8544_gpio::Pcd8544Gpio;
pub use pcd8544_spi::Pcd8544Spi;

const DISPLAY_WIDTH: usize = 84;
const DISPLAY_HEIGHT: usize = 6;
const BUFFER_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;

// #[repr(usize)]
// enum Command {
//     SetXAddress = 0x80,
//     SetYAddress = 0x40,
//     FunctionSet = 0x20, // Basic instruction set, only supported mode currently
// }

pub trait Pcd8544Backend {
    fn command(&mut self, command: u8);
    fn data(&mut self, data: &[u8]);
}

pub struct Pcd8544Driver<B: Pcd8544Backend> {
    backend: B,
    xpos: u8,
    ypos: u8,
    buffer: [0u8; BUFFER_SIZE],
}

impl<B: Pcd8544Backend> Pcd8544Driver<B> {
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            xpos: 0,
            ypos: 0,
        }
    }

    pub fn init(&mut self) {
        self.backend.command(0x21); // chip active; horizontal addressing mode (V = 0); use extended instruction set (H = 1)
                                    // set LCD Vop (contrast), which may require some tweaking:
        self.backend.command(0xB8); // try 0xB1 (for 3.3V red SparkFun), 0xB8 (for 3.3V blue SparkFun), 0xBF if your display is too dark, or 0x80 to 0xFF if experimenting
        self.backend.command(0x04); // set temp coefficient
        self.backend.command(0x14); // LCD bias mode 1:48: try 0x13 or 0x14

        self.backend.command(0x20); // we must send 0x20 before modifying the display control mode
        self.backend.command(0x0C); // set display control to normal mode: 0x0D for inverse

        self.clear();
    }

    fn new_line(&mut self) {

        // move to the next line
        // TODO
        // let ypos = *YPOS.lock();
        if ypos < DISPLAY_HEIGHT - 1 {
            self.set_position(xpos, ypos + 1);
        } else {
            // scroll the display up
            // TODO
            // let buffer = *BUFFER.lock();
            // buffer.copy_within(0..BUFFER_SIZE - DISPLAY_WIDTH, DISPLAY_WIDTH);
            // self.draw_buffer(&buffer);
            self.set_position(0, 0);
        }
    }

    fn print_char(&mut self, c: u8) {
        self.backend.data(&font::ASCII[c as usize - 0x20]);
        self.backend.data(&[0x00]);
    }

    pub fn print(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e => self.print_char(byte),
                b'\n' => self.new_line(),
                // not part of printable ASCII range
                _ => self.print_char(b'?'),
            }
        }
    }

    fn set_position(&mut self, x: u8, y: u8) {
        assert!(usize::from(x) < DISPLAY_WIDTH);
        assert!(usize::from(y) < DISPLAY_HEIGHT);

        // self.backend.command(Command::SetYAddress.into() + y);
        // self.backend.command(Command::SetXAddress | x);
        self.backend.command(0x40 + y);
        self.backend.command(0x80 + y);

        self.xpos = x;
        self.ypos = y;
    }

    // note: data direction is vertical: [1 2 3 4 5 6]
    // 1 3 5
    // 2 4 6
    fn draw_buffer(&mut self, buffer: &[u8; BUFFER_SIZE]) {
        self.backend.command(0x22); // vertical addressing
        self.set_position(0, 0);
        self.backend.data(buffer);
        self.backend.command(0x20); // horizontal addressing
        self.set_position(0, 0);
    }

    fn clear(&mut self) {
        self.set_position(0, 0);
        self.backend.data(&[0u8; BUFFER_SIZE]);
        self.set_position(0, 0);
    }
}
