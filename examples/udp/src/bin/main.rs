#![no_main]
#![no_std]

use udp as _; // global logger + panicking-behavior + memory layout

use nrf52840_hal::{
  self as hal,
  gpio::{p0::Parts as P0Parts, Level},
  spim::{self, Spim},
  Timer,
  twim::{self, Twim},
};

use poe_featherwing::{
  PoeFeatherWing,
  net::{
    Ipv4Addr,
    SocketAddrV4
  },
  Udp,
  Socket
};

/// W5500 static IPv4
const W5500_IP: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 222);
/// W5500 gateway IP
const GATEWAY: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 1);
/// W5500 subnet mask
const SUBNET_MASK: Ipv4Addr = Ipv4Addr::new(255, 255, 255, 0);
/// Target static IPv4
const TARGET_IP: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 183);

#[cortex_m_rt::entry]
fn main() -> ! {
    let board = hal::pac::Peripherals::take().unwrap();

    let p0_pins = P0Parts::new(board.P0);

    let scl = p0_pins.p0_11.into_floating_input().degrade();
    let sda = p0_pins.p0_12.into_floating_input().degrade();
    let twim = Twim::new(board.TWIM0, twim::Pins { scl, sda }, twim::Frequency::K100);

    let mosi = p0_pins.p0_13.into_push_pull_output(Level::Low).degrade();
    let miso = p0_pins.p0_15.into_floating_input().degrade();
    let clk = p0_pins.p0_14.into_push_pull_output(Level::Low).degrade();

    let spi_pins = spim::Pins {
      sck: clk,
      miso: Some(miso),
      mosi: Some(mosi),
    };

    let spi = Spim::new(
      board.SPIM2,
      spi_pins,
      spim::Frequency::K500,
      spim::MODE_0,
      0,
    );

    let mut delay = Timer::new(board.TIMER0);

    let cs_w5500 = p0_pins.p0_27.into_push_pull_output(Level::Low).degrade();

    let mut poe_feather = PoeFeatherWing::new(spi, cs_w5500, twim);
    let mac = poe_feather.get_mac_address().unwrap();
    defmt::info!("MAC Address {:?}", mac.octets);

    poe_feather.initialise(&W5500_IP, &GATEWAY, &SUBNET_MASK).unwrap();

    defmt::debug!("Polling for link up");
    poe_feather.wait_for_linkup(&mut delay, 5000).unwrap();

    let version: u8 = poe_feather.get_version().unwrap();

    defmt::info!("w5500 version {=u8}", version);

    let mut millis: u64 = 0;
    loop {

      if (millis % 1000000) == 0 {
        // open Socket0 as a UDP socket on port 1234
        poe_feather.w5500.udp_bind(Socket::Socket0, 1234).unwrap();

        // send 4 bytes to 192.168.2.4:8080, and get the number of bytes transmitted
        let data: &[u8] = b"Hello World!";
        let destination = SocketAddrV4::new(TARGET_IP, 32522);
        let tx_bytes = poe_feather.w5500.udp_send_to(Socket::Socket0, &data, &destination).unwrap();

        defmt::info!("sent {=usize}", tx_bytes);
      }

      millis = millis.wrapping_add(1);
    }
}