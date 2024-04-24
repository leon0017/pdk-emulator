use program::read_program;

pub mod program;

const TEST_PROGRAM: &'static str = "TestProgram_PFS154.bin";

fn main() {
    let program_bytes = read_program(&TEST_PROGRAM).unwrap();
    for b in program_bytes {
        print!("{b:02X} ");
    }
    println!();
}
