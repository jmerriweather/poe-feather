#![no_std]

pub mod net;

//pub use w5500_hl;

use w5500_hl::Common;
pub use w5500_hl::Udp;
pub use w5500_hl::Tcp;
pub use w5500_hl::ll::{
    LinkStatus,
    PhyCfg,
    Registers,
    Socket,
    Mode,
    SocketMode,
    Interrupt,
    SocketInterrupt,
    SocketInterruptMask,
    OperationMode,
    DuplexStatus,
    SpeedStatus
};

use net::{
    Eui48Addr,
    Ipv4Addr,
    SocketAddrV4
};

use {
    embedded_hal::{
        blocking::{i2c::WriteRead, spi::Transfer, spi::Write},
        digital::v2::OutputPin,
    },
    w5500_hl::ll::{
        blocking::vdm::W5500
    }
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

    pub fn initialise(&mut self, ip: &Ipv4Addr, gateway: &Ipv4Addr, subnet_mask: &Ipv4Addr) -> Result<(), Error<SpiError, PinError, I2CError>> {

        let mac = self.get_mac_address().map_err(Error::I2C)?;

        self.w5500.set_shar(&mac)?;
        self.w5500.set_sipr(&ip)?;
        self.w5500.set_gar(&gateway)?;
        self.w5500.set_subr(&subnet_mask)?;

        Ok(())
    }

    pub fn wait_for_linkup<D>(&mut self, delay: &mut D, timeout_ms: usize) -> Result<(), Error<SpiError, PinError, I2CError>>
    where
        D:  embedded_hal::blocking::delay::DelayMs<u8>,
    {
        let mut attempts: usize = 0;
        loop {
            let phy_cfg: PhyCfg = self.w5500.phycfgr()?;
            if phy_cfg.lnk() == LinkStatus::Up {
                break;
            }
            assert!(attempts < timeout_ms / 100, "Failed to link up in timeout");
            delay.delay_ms(100);
            attempts += 1;
        }
        Ok(())
    }

    pub fn get_phy_configuration(&mut self) -> Result<PhyCfg, Error<SpiError, PinError, I2CError>> {
        Ok(self.w5500.phycfgr()?)
    }

    pub fn set_phy_configuration(&mut self, phycfg: PhyCfg) -> Result<(), Error<SpiError, PinError, I2CError>> {
        Ok(self.w5500.set_phycfgr(phycfg)?)
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

    pub fn get_version(&mut self) -> Result<u8, Error<SpiError, PinError, I2CError>> {
        Ok(self.w5500.version()?)
    }

    pub fn set_mac_address(&mut self, mac: &Eui48Addr) -> Result<(), Error<SpiError, PinError, I2CError>> {
        Ok(self.w5500.set_shar(mac)?)
    }

    pub fn set_ip_address(&mut self, ip: &Ipv4Addr) -> Result<(), Error<SpiError, PinError, I2CError>> {
        Ok(self.w5500.set_sipr(ip)?)
    }

    pub fn set_gateway(&mut self, gateway: &Ipv4Addr) -> Result<(), Error<SpiError, PinError, I2CError>> {
        Ok(self.w5500.set_gar(gateway)?)
    }

    pub fn set_subnet_mask(&mut self, subnet_mask: &Ipv4Addr) -> Result<(), Error<SpiError, PinError, I2CError>> {
        Ok(self.w5500.set_subr(subnet_mask)?)
    }

    // UDP
    pub fn udp_bind(&mut self, socket: Socket, port: u16) -> Result<(), Error<SpiError, PinError, I2CError>> {
        Ok(self.w5500.udp_bind(socket, port)?)
    }

    pub fn udp_recv_from(
        &mut self,
        socket: Socket,
        buf: &mut [u8],
    ) -> nb::Result<(usize, SocketAddrV4), w5500_hl::ll::blocking::vdm::Error<SpiError, PinError>> {
        self.w5500.udp_recv_from(socket, buf)
    }

    pub fn udp_peek_from(
        &mut self,
        socket: Socket,
        buf: &mut [u8],
    ) -> nb::Result<(usize, SocketAddrV4), w5500_hl::ll::blocking::vdm::Error<SpiError, PinError>> {
        self.w5500.udp_peek_from(socket, buf)
    }

    pub fn udp_peek_from_header(&mut self, socket: Socket) -> nb::Result<(usize, SocketAddrV4), w5500_hl::ll::blocking::vdm::Error<SpiError, PinError>> {
        self.w5500.udp_peek_from_header(socket)
    }

    pub fn udp_send_to(&mut self, socket: Socket, buf: &[u8], addr: &SocketAddrV4) -> Result<usize, Error<SpiError, PinError, I2CError>> {
        Ok(self.w5500.udp_send_to(socket, buf, addr)?)
    }

    pub fn udp_send(&mut self, socket: Socket, buf: &[u8]) -> Result<usize, Error<SpiError, PinError, I2CError>> {
        Ok(self.w5500.udp_send(socket, buf)?)
    }

    // TCP
    pub fn tcp_connect(&mut self, socket: Socket, port: u16, addr: &SocketAddrV4) -> Result<(), Error<SpiError, PinError, I2CError>> {
        Ok(self.w5500.tcp_connect(socket, port, addr)?)
    }

    pub fn tcp_listen(&mut self, socket: Socket, port: u16) -> Result<(), Error<SpiError, PinError, I2CError>> {
        Ok(self.w5500.tcp_listen(socket, port)?)
    }

    pub fn tcp_read(&mut self, socket: Socket, buf: &mut [u8]) -> Result<usize, Error<SpiError, PinError, I2CError>> {
        Ok(self.w5500.tcp_read(socket, buf)?)
    }

    pub fn tcp_write(&mut self, socket: Socket, buf: &[u8]) -> Result<usize, Error<SpiError, PinError, I2CError>> {
        Ok(self.w5500.tcp_write(socket, buf)?)
    }

    pub fn tcp_disconnect(&mut self, socket: Socket) -> Result<(), Error<SpiError, PinError, I2CError>> {
        Ok(self.w5500.tcp_disconnect(socket)?)
    }

    // Common
    pub fn local_addr(&mut self, socket: Socket) -> Result<SocketAddrV4, Error<SpiError, PinError, I2CError>> {
        Ok(self.w5500.local_addr(socket)?)
    }

    pub fn close(&mut self, socket: Socket) -> Result<(), Error<SpiError, PinError, I2CError>> {
        Ok(self.w5500.close(socket)?)
    }

    pub fn is_state_closed(&mut self, socket: Socket) -> Result<bool, Error<SpiError, PinError, I2CError>> {
        Ok(self.w5500.is_state_closed(socket)?)
    }

    pub fn is_state_tcp(&mut self, socket: Socket) -> Result<bool, Error<SpiError, PinError, I2CError>> {
        Ok(self.w5500.is_state_tcp(socket)?)
    }

    pub fn is_state_udp(&mut self, socket: Socket) -> Result<bool, Error<SpiError, PinError, I2CError>> {
        Ok(self.w5500.is_state_udp(socket)?)
    }
}
