use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;

use crate::Pcd8544Backend;

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
        rst: Option<&mut T>,
        delay: &mut R,
    ) -> Self {
        if let Some(r) = rst {
            let _ = r.set_low();
            delay.delay_ns(1);
            let _ = r.set_high();
        }

        Self { clk, din, dc, cs }
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

impl<CLK, DIN, DC, CS> Pcd8544Backend for Pcd8544Gpio<CLK, DIN, DC, CS>
where
    CLK: OutputPin,
    DIN: OutputPin,
    DC: OutputPin,
    CS: OutputPin,
{
    fn command(&mut self, byte: u8) {
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

    fn data(&mut self, data: &[u8]) {
        let _ = self.dc.set_high();
        let _ = self.cs.set_low();
        for byte in data {
            self.send(*byte);
        }
        let _ = self.cs.set_high();
    }
}
