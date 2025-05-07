use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiBus;

use crate::Pcd8544;

pub struct Pcd8544Spi<SPI, DC, CS> {
    spi: SPI,
    dc: DC,
    cs: CS,
}

impl<SPI, DC, CS> Pcd8544Spi<SPI, DC, CS>
where
    SPI: SpiBus,
    DC: OutputPin,
    CS: OutputPin,
{
    pub fn new<T: OutputPin, R: DelayNs>(
        spi: SPI,
        dc: DC,
        cs: CS,
        rst: &mut T,
        delay: &mut R,
    ) -> Pcd8544Spi<SPI, DC, CS> {
        let _ = rst.set_low();
        delay.delay_ns(1);
        let _ = rst.set_high();

        let mut pcd = Pcd8544Spi { spi, dc, cs };
        pcd.init();
        pcd
    }
}

impl<SPI, DC, CS> Pcd8544 for Pcd8544Spi<SPI, DC, CS>
where
    SPI: SpiBus,
    DC: OutputPin,
    CS: OutputPin,
{
    fn command(&mut self, cmd: u8) {
        let _ = self.dc.set_low();
        let _ = self.cs.set_low();
        let _ = self.spi.write(&[cmd]);
        let _ = self.cs.set_high();
    }

    fn data(&mut self, data: &[u8]) {
        let _ = self.dc.set_high();
        let _ = self.cs.set_low();
        let _ = self.spi.write(data);
        let _ = self.cs.set_high();
    }
}
