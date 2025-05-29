#![deny(unsafe_code)]
#![deny(warnings)]
#![no_std]

mod chars;
mod pcd8544_gpio;
mod pcd8544_spi;

pub use pcd8544_gpio::Pcd8544Gpio;
pub use pcd8544_spi::Pcd8544Spi;

const DISPLAY_ROW_COUNT: u8 = 6;
const DISPLAY_COL_COUNT: u8 = 84;
const DISPLAY_BYTES: usize = 504;

mod consts {
    pub(crate) const X_ADDR: u8 = 0x80;
    pub(crate) const Y_ADDR: u8 = 0x40;
    pub(crate) const FUNCTION_SET: u8 = 0x20;
    pub(crate) const FUNCTION_SET_H_ADDRESSING: u8 = 0b00;
    pub(crate) const FUNCTION_SET_V_ADDRESSING: u8 = 0b10;
    pub(crate) const EXTENDED_INSTRUCTION_SET: u8 = 0x01;
    pub(crate) const VOP: u8 = 0x80;
    pub(crate) const TEMP_COEFF: u8 = 0b100;
    pub(crate) const BIAS: u8 = 0x10;
    pub(crate) const DISPLAY_CONTROL: u8 = 0x08;
    pub(crate) const DISPLAY_CONF_NORMAL: u8 = 0b100;

    pub(crate) const VOID_SCREEN: [u8; super::DISPLAY_BYTES] = [0u8; super::DISPLAY_BYTES];
}

struct DisplayBuffer {
    data: [u8; DISPLAY_BYTES],
}

impl Default for DisplayBuffer {
    fn default() -> Self {
        Self {
            data: consts::VOID_SCREEN,
        }
    }
}

pub struct Pcd8544Driver<B: Pcd8544Backend, D: embedded_hal::delay::DelayNs> {
    backend: B,
    buffer: DisplayBuffer,
    xpos: u8,
    ypos: u8,
    delay: D,
}

pub trait Pcd8544Backend {
    fn command(&mut self, command: u8);
    fn data(&mut self, data: &[u8]);
}

impl<B: Pcd8544Backend, D: embedded_hal::delay::DelayNs> Pcd8544Driver<B, D> {
    pub fn new(backend: B, delay: D) -> Self {
        Self {
            backend,
            delay,
            buffer: DisplayBuffer::default(),
            xpos: 0,
            ypos: 0,
        }
    }

    pub fn init(&mut self) {
        // chip active (PD=0); horizontal addressing mode (V = 0); use extended instruction set (H = 1)
        self.backend
            .command(consts::FUNCTION_SET + consts::EXTENDED_INSTRUCTION_SET);
        // try 0xB1 (for 3.3V red SparkFun), 0xB8 (for 3.3V blue SparkFun), 0xBF if your display is too dark, or 0x80 to 0xFF if experimenting
        self.backend.command(consts::VOP + 0b00111000);
        // temp coefficient (0)
        self.backend.command(consts::TEMP_COEFF);
        // LCD bias mode 1:48
        self.backend.command(consts::BIAS + 0b011);

        // we must send 0x20 before modifying the display control mode
        self.backend.command(consts::FUNCTION_SET);
        // set display control to normal mode (pixels are on when data is 1), inverse mode=0x0D
        self.backend
            .command(consts::DISPLAY_CONTROL + consts::DISPLAY_CONF_NORMAL);

        self.clear();
    }

    pub fn print(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e => self.print_char(byte),
                b'\n' => self.new_line(),
                _ => self.print_char(b'?'),
            }
        }
    }

    pub fn print_char(&mut self, c: u8) {
        let glyph = &chars::CHAR_AS_PIXEL_ARRAY[c as usize - 0x20];

        // Write the 5 glyph bytes
        for b in glyph {
            self.backend.data(&[*b]);
            self.buffer.data[self.offset()] = *b;
            self.inc_cursor();
        }

        // Write the space byte
        self.backend.data(&[0x00]);
        self.buffer.data[self.offset()] = 0x00;
        self.inc_cursor();
    }

    pub fn clear(&mut self) {
        self.reset_cursor();
        self.backend.data(&consts::VOID_SCREEN);
        self.buffer.data.copy_from_slice(&consts::VOID_SCREEN);
        self.reset_cursor();
    }

    pub fn draw_buffer(&mut self, buffer: &[u8; DISPLAY_BYTES]) {
        self.backend
            .command(consts::FUNCTION_SET + consts::FUNCTION_SET_V_ADDRESSING);
        self.reset_cursor();
        self.backend.data(buffer);
        self.backend
            .command(consts::FUNCTION_SET + consts::FUNCTION_SET_H_ADDRESSING);
        for i in 0..DISPLAY_BYTES {
            self.buffer.data[i] = buffer[i];
        }
        self.reset_cursor();
    }

    pub fn set_cursor(&mut self, x: u8, y: u8) {
        if x >= DISPLAY_COL_COUNT || y >= DISPLAY_ROW_COUNT {
            return;
        }

        self.backend.command(consts::X_ADDR as u8 + x);
        self.backend.command(consts::Y_ADDR as u8 + y);
        self.xpos = x;
        self.ypos = y;
    }

    pub fn scroll(&mut self) {
        let (prevx, prevy) = (self.xpos, self.ypos);

        // we want to reset the sceren without resetting the buffer
        self.reset_cursor();
        self.backend.data(&consts::VOID_SCREEN);
        self.reset_cursor();

        // Create the modified buffer
        self.buffer
            .data
            .copy_within(DISPLAY_COL_COUNT as usize..DISPLAY_BYTES, 0);
        for byte in self
            .buffer
            .data
            .iter_mut()
            .skip(DISPLAY_COL_COUNT as usize * 5)
        {
            *byte = 0;
        }

        // Send the new buffer to the controller, since the last row was cleared,
        // we only need to send the first 5 lines
        self.backend
            .data(&self.buffer.data[0..DISPLAY_COL_COUNT as usize * 5]);

        // restore cursor positions, only ypos is decremented (unless it was at
        // the first row = 0)
        self.set_cursor(prevx, prevy.saturating_sub(1));
    }

    pub fn power_down(&mut self) {
        // we want to reset the sceren without resetting the buffer
        self.reset_cursor();
        self.backend.data(&consts::VOID_SCREEN);
        self.reset_cursor();

        self.backend.data(&[0b00100100]);
    }

    pub fn wake_up(&mut self) {
        self.backend.data(&[0b00100000]);
        // self.draw_buffer(self.buffer.data);
    }

    pub fn blink(&mut self, nb_times: u8) {
        for _ in 0..nb_times {
            self.reset_cursor();
            for b in self.buffer.data {
                let flip = !b;
                self.backend.data(&[flip]);
            }

            self.delay.delay_ms(200);

            self.reset_cursor();
            self.backend.data(&self.buffer.data);
        }
    }

    fn inc_cursor(&mut self) {
        self.xpos += 1;
        if self.xpos >= DISPLAY_COL_COUNT {
            self.xpos = 0;
            if self.ypos + 1 >= DISPLAY_ROW_COUNT {
                let (posx, posy) = (self.xpos, self.ypos);
                self.scroll();
                self.set_cursor(posx, posy);
            } else {
                self.ypos += 1;
            }
        }
        self.set_cursor(self.xpos, self.ypos);
    }

    fn new_line(&mut self) {
        if self.ypos + 1 == DISPLAY_ROW_COUNT {
            self.scroll();
        }
        self.set_cursor(0, (self.ypos + 1) % DISPLAY_ROW_COUNT);
    }

    fn reset_cursor(&mut self) {
        self.set_cursor(0, 0);
    }

    #[inline(always)]
    fn offset(&self) -> usize {
        self.ypos as usize * DISPLAY_COL_COUNT as usize + self.xpos as usize
    }
}
