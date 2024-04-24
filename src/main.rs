use time::sys_nanos;

pub mod program;
pub mod time;

const TEST_PROGRAM: &str = "TestProgram_PFS154.bin";
const CLOCK_HZ: f64 = 1_000_000.0;

fn main() {
    // TODO: Prettier error logging instead of unwrapping
    let program_bytes = program::read(TEST_PROGRAM).unwrap();
    for b in program_bytes {
        print!("{b:02X} ");
    }
    println!();

    let mut start_nanos = sys_nanos().unwrap();

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let nano_sleep = ((1.0 / CLOCK_HZ) * 1_000_000_000.0) as u128;

    let mut i: i64 = 0;
    loop {
        let now_nanos = sys_nanos().unwrap();
        if (now_nanos - start_nanos) > nano_sleep {
            i += 1;

            if i % 1_000_000 == 0 {
                println!("Clock has ticked: {i} times");
            }

            start_nanos = now_nanos;
        }
    }
}
