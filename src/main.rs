//! PIO DOOM shim
#![no_std]
#![no_main]

use hal::Sio;
use hal::gpio::{FunctionPio0, Pin};
use hal::pac;
use hal::pio::PIOExt;
use panic_halt as _;
use rp2040_hal as hal;

#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

#[rp2040_hal::entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();

    let sio = Sio::new(pac.SIO);
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let led: Pin<_, FunctionPio0, _> = pins.gpio23.into_function();
    let led_pin_id = led.id().num;

    /*const MAX_DELAY: u8 = 31;
    let mut a = pio::Assembler::<32>::new();
    let mut wrap_target = a.label();
    let mut wrap_source = a.label();
    a.set(pio::SetDestination::PINDIRS, 1);
    a.bind(&mut wrap_target);
    a.set_with_delay(pio::SetDestination::PINS, 0, MAX_DELAY);
    a.set_with_delay(pio::SetDestination::PINS, 1, MAX_DELAY);
    a.bind(&mut wrap_source);
    let program = a.assemble_with_wrap(wrap_source, wrap_target);*/
    let program_defines = pio_proc::pio_file!("src/pio/doom0.pio");
    let program = program_defines.program;

    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let installed = pio.install(&program).unwrap();
    let (int, frac) = (0, 0);
    let (sm, _, _) = rp2040_hal::pio::PIOBuilder::from_installed_program(installed)
        .set_pins(led_pin_id, 1)
        .clock_divisor_fixed_point(int, frac)
        .build(sm0);
    sm.start();

    loop {
        cortex_m::asm::wfi();
    }
}
