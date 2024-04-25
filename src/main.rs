use crate::cpu::CPU;

pub mod cpu;
pub mod program;
pub mod time;

const TEST_PROGRAM: &str = "TestProgram_PFS154.bin";

const ROM_SIZE_WORDS: usize = 2048;
const RAM_SIZE_BYTES: usize = 128;
const IO_SIZE_BYTES: usize = 64;

fn main() {
    // TODO: Prettier error logging instead of unwrapping
    let program_bytes = program::read(TEST_PROGRAM).unwrap();
    for b in program_bytes {
        print!("{b:02X} ");
    }
    println!();

    let mut cpu = CPU::new(ROM_SIZE_WORDS, RAM_SIZE_BYTES, IO_SIZE_BYTES);
    cpu.set_clock_speed(1_000_000);
    cpu.main_clock_loop();
}
