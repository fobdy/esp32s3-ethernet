//! This example demonstrates how to configure an SPI based Ethernet adapter

fn main() {
    example::main().unwrap();
}

pub mod example {
    use esp_idf_svc::{
        eth,
        eventloop::EspSystemEventLoop,
        hal::{prelude::Peripherals, spi, units::FromValueType},
        ipv4,
        log::EspLogger,
        ping,
        sys::EspError,
    };
    use log::{info, warn};

    pub fn main() -> anyhow::Result<()> {
        esp_idf_svc::sys::link_patches();
        EspLogger::initialize_default();

        let peripherals = Peripherals::take()?;
        let pins = peripherals.pins;
        let sysloop = EspSystemEventLoop::take()?;

        let spi = spi::SpiDriver::new(
            peripherals.spi2,
            pins.gpio13,
            pins.gpio11,
            Some(pins.gpio12),
            &spi::SpiDriverConfig::new().dma(spi::Dma::Auto(4096)),
        )?;

        let eth_driver = eth::EthDriver::new_spi(
            spi,
            pins.gpio10,
            Some(pins.gpio14),
            Some(pins.gpio9),
            eth::SpiEthChipset::W5500,
            20_u32.MHz().into(),
            Some(&[0x02, 0x00, 0x00, 0x12, 0x34, 0x56]),
            None,
            sysloop.clone(),
        )?;

        let eth = eth::EspEth::wrap(eth_driver)?;

        let mut eth = eth::BlockingEth::wrap(eth, sysloop.clone())?;

        info!("Starting eth...");

        eth.start()?;

        info!("Waiting for DHCP lease...");

        eth.wait_netif_up()?;

        let ip_info = eth.eth().netif().get_ip_info()?;

        info!("Eth DHCP info: {:?}", ip_info);

        ping(ip_info.subnet.gateway)?;

        Ok(())
    }

    fn ping(ip: ipv4::Ipv4Addr) -> Result<(), EspError> {
        info!("About to do some pings for {:?}", ip);

        let ping_summary = ping::EspPing::default().ping(ip, &Default::default())?;
        if ping_summary.transmitted != ping_summary.received {
            warn!("Pinging IP {} resulted in timeouts", ip);
        }

        info!("Pinging done");

        Ok(())
    }
}
