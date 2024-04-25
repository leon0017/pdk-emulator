use crate::cpu::CPU;

pub mod arch;
pub mod cpu;
pub mod peripherals;
pub mod program;
pub mod tests;
pub mod time;

const TEST_PROGRAM: &str = "TestProgram_PFS154.bin";

const ROM_SIZE_WORDS: usize = 2048;
const RAM_SIZE_BYTES: usize = 128;
const IO_SIZE_BYTES: usize = 64;

#[must_use]
pub fn default_cpu() -> CPU {
    CPU::new(ROM_SIZE_WORDS, RAM_SIZE_BYTES, IO_SIZE_BYTES, arch::pdk14())
}

fn main() {
    // TODO: Prettier error logging instead of unwrapping
    let program_bytes = program::read(TEST_PROGRAM).unwrap();

    let mut cpu = CPU::new(ROM_SIZE_WORDS, RAM_SIZE_BYTES, IO_SIZE_BYTES, arch::pdk14());

    println!("PROGRAM:");
    for b in &program_bytes {
        print!("{b:02X} ");
    }
    println!();

    cpu.load_program(program_bytes.as_slice()).unwrap();

    println!("ROM:");
    for b in &cpu.rom {
        print!("{b:04X} ");
    }
    println!();

    cpu.set_clock_speed(1_000_000);
    cpu.main_clock_loop();
}
