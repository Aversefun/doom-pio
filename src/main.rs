//! PIO DOOM shim
#![no_std]
#![no_main]

use hal::Sio;
use hal::gpio::{FunctionPio0, Pin};
use hal::pac;
use hal::pio::PIOExt;
use panic_halt as _;
use pio::{Instruction, SideSet};
use rp2040_hal as hal;

#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

static DOOM_DAT: &[u8] = include_bytes!("../DOOM.dat");

#[derive(Clone, Copy, Debug, Default)]
struct Input {
    turn_left: bool,
    turn_right: bool,
    forwards: bool,
    backwards: bool,
    strafe_left: bool,
    strafe_right: bool,
    fire: bool,
    use_open: bool,
    run: bool,
}

impl Input {
    fn get_bitmask(self) -> u32 {
        let mut out = 0u32;
        if self.turn_left {
            out |= 0b1;
        }
        if self.turn_right {
            out |= 0b10;
        }
        if self.forwards {
            out |= 0b100;
        }
        if self.backwards {
            out |= 0b1000;
        }
        if self.strafe_left {
            out |= 0b10000;
        }
        if self.strafe_right {
            out |= 0b100000;
        }
        if self.fire {
            out |= 0b1000000;
        }
        if self.use_open {
            out |= 0b10000000;
        }
        if self.run {
            out |= 0b100000000;
        }
        out
    }
}

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

    let nop_program_with_defines = pio_proc::pio_asm!(".wrap_target", "nop", ".wrap",);
    let nop_program = nop_program_with_defines.program;

    let program = unsafe {
        core::mem::transmute::<&'static [u8], &'static [u16]>(include_bytes!("../DOOM.bin"))
    };
    let prg_loc = 0usize;

    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let installed = pio.install(&nop_program).unwrap();
    let (int, frac) = (1, 0); // Same speed as system clock
    let (sm, mut rx, mut tx) = rp2040_hal::pio::PIOBuilder::from_installed_program(installed)
        .set_pins(led_pin_id, 1)
        .clock_divisor_fixed_point(int, frac)
        .build(sm0);
    let mut running_sm = sm.start();

    let mut screen = [[0u16; 64]; 128];

    let mut temp_data = [0u32; 2usize.pow(13)];

    let mut used = [false; 2usize.pow(13)];
    let mut input = Input::default();

    loop {
        running_sm.exec_instruction(
            Instruction::decode(program[prg_loc], SideSet::new(false, 0, false)).unwrap(),
        );

        if !rx.is_empty() {
            let instr = rx.read().unwrap();
            let cmd = (instr & (0b111u32 << 29)) >> 29;
            match cmd {
                0b011 => {
                    // Output to screen.

                    let color = instr as u16 & u16::MAX;
                    let x = (instr & (0b1111111 << 22)) >> 22;
                    let y = (instr & (0b111111 << 16)) >> 16;
                    screen[x as usize][y as usize] = color;
                }
                0b100 => {
                    // Read data.

                    let is_static = (instr & (1 << 28)) == (1 << 28);
                    let addr = instr & 0b1111111111111111111111111111;

                    let out = if is_static {
                        u32::from_be_bytes([
                            DOOM_DAT[addr as usize],
                            DOOM_DAT[addr as usize + 1],
                            DOOM_DAT[addr as usize + 2],
                            DOOM_DAT[addr as usize + 3],
                        ])
                    } else {
                        temp_data[addr as usize]
                    };

                    tx.write(out);
                }
                0b101 => {
                    // Write data RAM.

                    let data = instr as u16 & u16::MAX;
                    let addr = instr & 0b1111111111110000000000000000;

                    temp_data[addr as usize] = data as u32;
                    used[addr as usize] = true;
                }
                0b110 => {
                    // Allocate memory.

                    let length = instr as u8 & u8::MAX;

                    let mut contiguous = 0usize;
                    let mut addr = 0usize;
                    for (i, used) in used.iter().enumerate() {
                        if !used {
                            contiguous += 1;
                        }
                        if contiguous >= length as usize {
                            addr = i;
                            break;
                        }
                        if *used {
                            contiguous = 0;
                        }
                    }

                    tx.write(addr as u32);
                }
                0b111 => {
                    // Read input

                    tx.write(input.get_bitmask());
                }
                _ => unreachable!(),
            }
        }
    }
}
