#![no_std]

use embedded_hal::{
    blocking::{i2c::WriteRead, spi::Transfer, spi::Write},
    digital::v2::OutputPin,
};
use w5500_hl::ll::{
    blocking::vdm::W5500,
    net::{Eui48Addr, Ipv4Addr, SocketAddrV4},
    LinkStatus, PhyCfg, Registers, Socket,
};

pub struct PoeFeather<SPI, CS, I2C> {
    /// SPI bus.
    spi: SPI,
    /// GPIO for chip select.
    cs: CS,
    /// i2c for mac address retrival.
    i2c: I2C,
}

#[derive(Debug)]
pub enum Error<SpiError, PinError, I2CError> {
    /// SPI bus error wrapper.
    Spi(SpiError),
    /// GPIO pin error wrapper.
    Pin(PinError),
    /// I2C error wrapper.
    I2C(I2CError),
}

impl<SPI, CS, I2C, SpiError, PinError, I2CError> PoeFeather<SPI, CS, I2C>
where
    SPI: Transfer<u8, Error = SpiError> + Write<u8, Error = SpiError>,
    CS: OutputPin<Error = PinError>,
    I2C: WriteRead<Error = I2CError>,
{
    pub fn new(spi: SPI, cs: CS, i2c: I2C) -> Self {
        PoeFeather { spi, cs, i2c }
    }

    pub fn free(self) -> (SPI, CS, I2C) {
        (self.spi, self.cs, self.i2c)
    }

    pub fn get_mac_address(&mut self) -> Result<Eui48Addr, <I2C as WriteRead>::Error> {
        let registers: &[u8; 6] = &[0xFA, 0xFB, 0xFC, 0xFD, 0xFE, 0xFF];
        let mut collect: [u8; 6] = [0; 6];

        for (index, item) in registers.into_iter().enumerate() {
            let mut rxbuf = [0; 1];
            self.i2c.write_read(0x50, &[*item], &mut rxbuf).ok();
            collect[index] = rxbuf[0];
        }

        Ok(Eui48Addr::from(collect))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
