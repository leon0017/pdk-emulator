use std::iter;

use crate::time;

const NANOS_IN_SEC: f64 = 1_000_000_000.0;

#[allow(clippy::upper_case_acronyms, dead_code)]
pub struct CPU {
    pub rom: Vec<u16>,
    pub ram: Vec<u8>,
    pub io: Vec<u8>,
    freq_pause_nanos: u128,
    freq_hz: u64,
    cpu_cycle: u64,                  // Current CPU cycle
    pc: u32,                         // Program counter
    a: u16,                          // A register
    t16: u16,                        // Timer T16 value
    global_interrupts_enabled: bool, // Are global interrupts enabled
    interrupts_active: bool,         // Internal status that cpu started interrupt
}

fn zero_vec<T>(len: usize) -> Vec<T>
where
    T: Default + Clone,
{
    iter::repeat(T::default()).take(len).collect()
}

impl CPU {
    #[must_use]
    pub fn new(rom_size_words: usize, ram_size_bytes: usize, io_size_bytes: usize) -> Self {
        CPU {
            rom: zero_vec(rom_size_words),
            ram: zero_vec(ram_size_bytes),
            io: zero_vec(io_size_bytes),
            freq_pause_nanos: 1_000, // 1Mhz
            freq_hz: 1_000_000,
            cpu_cycle: 0,
            pc: 0x0000_0000,
            a: 0x0000,
            t16: 0x0000,
            global_interrupts_enabled: false,
            interrupts_active: false,
        }
    }

    pub fn set_clock_speed(&mut self, hz: u64) {
        #[allow(
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation,
            clippy::cast_precision_loss
        )]
        let nano_sleep = ((1.0 / hz as f64) * NANOS_IN_SEC) as u128;
        self.freq_pause_nanos = nano_sleep;
        self.freq_hz = hz;
    }

    // TODO: Clock loop could probably be improved, figure out a way to use sleep instead of recursively checking time
    pub fn main_clock_loop(&mut self) -> ! {
        let mut start_nanos = time::sys_nanos().unwrap();

        let mut i: u64 = 0;
        loop {
            let now_nanos = time::sys_nanos().unwrap();
            if (now_nanos - start_nanos) > self.freq_pause_nanos {
                // TODO: Run current instruction

                if i % self.freq_hz == 0 {
                    println!("Clock has ticked: {i} times");
                }

                i += 1;

                start_nanos = now_nanos;
            }
        }
    }
}
