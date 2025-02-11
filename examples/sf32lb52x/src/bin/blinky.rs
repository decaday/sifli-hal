#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;
// use panic_halt as _;

use sifli_hal;
use sifli_hal::gpio;

#[entry]
fn main() -> ! {
    let p = sifli_hal::init(Default::default());
    let mut pin = gpio::Output::new(p.PA26, gpio::Level::Low);
    info!("Hello World!");

    loop {
        info!("tick");
        pin.set_high();
        cortex_m::asm::delay(50_000_000);
        pin.set_low();
        cortex_m::asm::delay(50_000_000);
    }
}
