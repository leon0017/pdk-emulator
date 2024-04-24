use time::sys_nanos;

pub mod program;
pub mod time;

const TEST_PROGRAM: &str = "TestProgram_PFS154.bin";
const CLOCK_HZ: f64 = 1_000_000.0;
const NANOS_IN_SEC: f64 = 1_000_000_000.0;

fn main() {
    // TODO: Prettier error logging instead of unwrapping
    let program_bytes = program::read(TEST_PROGRAM).unwrap();
    for b in program_bytes {
        print!("{b:02X} ");
    }
    println!();

    let mut start_nanos = sys_nanos().unwrap();

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let nano_sleep = ((1.0 / CLOCK_HZ) * NANOS_IN_SEC) as u128;

    // TODO: Clock loop could probably be improved, figure out a way to use sleep instead of recursively checking time
    let mut i: i64 = 0;
    loop {
        let now_nanos = sys_nanos().unwrap();
        if (now_nanos - start_nanos) > nano_sleep {
            if i % 1_000_000 == 0 {
                println!("Clock has ticked: {i} times");
            }

            i += 1;

            start_nanos = now_nanos;
        }
    }
}
