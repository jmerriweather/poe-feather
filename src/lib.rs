#![no_std]

use embedded_hal::{
    blocking::{i2c::WriteRead, spi::Transfer, spi::Write},
    digital::v2::OutputPin,
};
pub use w5500_hl;
pub use w5500_hl::ll;

use w5500_hl::ll::{
    blocking::vdm::W5500,
    net::{Eui48Addr, Ipv4Addr, SocketAddrV4},
    LinkStatus, PhyCfg, Registers, Socket,
};

pub struct PoeFeatherWing<SPI, CS, I2C> {
    /// i2c for mac address retrival.
    i2c: I2C,
    /// w5500 implementation
    pub w5500: W5500<SPI, CS>,
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

impl<SpiError, PinError, I2CError> From<w5500_hl::ll::blocking::vdm::Error<SpiError, PinError>> for Error<SpiError, PinError, I2CError> {
    fn from(error: w5500_hl::ll::blocking::vdm::Error<SpiError, PinError>) -> Self {
        match error {
            w5500_hl::ll::blocking::vdm::Error::Pin(error) => Error::Pin(error),
            w5500_hl::ll::blocking::vdm::Error::Spi(error) => Error::Spi(error)
        }
    }
}

impl<SPI, CS, I2C, SpiError, PinError, I2CError> PoeFeatherWing<SPI, CS, I2C>
where
    SPI: Transfer<u8, Error = SpiError> + Write<u8, Error = SpiError>,
    CS: OutputPin<Error = PinError>,
    I2C: WriteRead<Error = I2CError>,
{
    pub fn new(spi: SPI, cs: CS, i2c: I2C) -> Self {
        PoeFeatherWing { i2c, w5500: W5500::new(spi, cs) }
    }

    pub fn get_mac_address(&mut self) -> Result<Eui48Addr, I2CError> {
        let registers: &[u8; 6] = &[0xFA, 0xFB, 0xFC, 0xFD, 0xFE, 0xFF];
        let mut collect: [u8; 6] = [0; 6];

        for (index, item) in registers.into_iter().enumerate() {
            let mut rxbuf = [0; 1];
            self.i2c.write_read(0x50, &[*item], &mut rxbuf).ok();
            collect[index] = rxbuf[0];
        }

        Ok(Eui48Addr::from(collect))
    }

    pub fn initialise(&mut self, ip: &Ipv4Addr, gateway: &Ipv4Addr, subnet_mask: &Ipv4Addr) -> Result<(), Error<SpiError, PinError, I2CError>> {

        let mac = self.get_mac_address().map_err(Error::I2C)?;

        self.w5500.set_shar(&mac)?;
        self.w5500.set_sipr(&ip)?;
        self.w5500.set_gar(&gateway)?;
        self.w5500.set_subr(&subnet_mask)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
