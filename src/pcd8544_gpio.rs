use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;

use crate::Pcd8544;

pub struct Pcd8544Gpio<CLK, DIN, DC, CS> {
    clk: CLK,
    din: DIN,
    dc: DC,
    cs: CS,
}

impl<CLK, DIN, DC, CS> Pcd8544Gpio<CLK, DIN, DC, CS>
where
    CLK: OutputPin,
    DIN: OutputPin,
    DC: OutputPin,
    CS: OutputPin,
{
    pub fn new<T: OutputPin, R: DelayNs>(
        clk: CLK,
        din: DIN,
        dc: DC,
        cs: CS,
        rst: &mut T,
        delay: &mut R,
    ) -> Pcd8544Gpio<CLK, DIN, DC, CS> {
        let _ = rst.set_low();
        delay.delay_ms(10);
        let _ = rst.set_high();

        let mut pcd = Pcd8544Gpio { clk, din, dc, cs };
        pcd.init();
        pcd
    }

    fn send(&mut self, byte: u8) {
        for bit in (0..8).rev() {
            if (byte & (1 << bit)) != 0 {
                let _ = self.din.set_high();
            } else {
                let _ = self.din.set_low();
            }

            let _ = self.clk.set_high();
            let _ = self.clk.set_low();
        }
    }
}

impl<CLK, DIN, DC, CS> Pcd8544 for Pcd8544Gpio<CLK, DIN, DC, CS>
where
    CLK: OutputPin,
    DIN: OutputPin,
    DC: OutputPin,
    CS: OutputPin,
{
    fn command(&mut self, cmd: u8) {
        let _ = self.dc.set_low();
        let _ = self.cs.set_low();
        self.send(cmd);
        let _ = self.cs.set_high();
    }

    fn data(&mut self, data: &[u8]) {
        let _ = self.dc.set_high();
        let _ = self.cs.set_low();
        for byte in data {
            self.send(*byte);
        }
        let _ = self.cs.set_high();
    }
}
