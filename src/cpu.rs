use core::panic;
use snafu::prelude::*;
use std::iter;

use crate::{arch::PDKArch, peripherals, time};

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
    arch: PDKArch,                   // CPU architecture (e.g. pdk14, pdk15...)
}

fn zero_vec<T>(len: usize) -> Vec<T>
where
    T: Default + Clone,
{
    iter::repeat(T::default()).take(len).collect()
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ProgramError {
    #[snafu(display(
        "The program is too large to be loaded into ROM, size in bytes: '{program_len_bytes}'"
    ))]
    ProgramTooLarge { program_len_bytes: usize },
}

enum RAMOperation {
    Read,
    Write,
}

enum CPUError {
    IllegalROMAccess {
        addr: u16,
    },
    IllegalRAMAccess {
        addr: u8,
        ram_operation: RAMOperation,
    },
    IllegalIOAccess {
        addr: u8,
    },
    IndirectPointer {
        // Indirect pointer outside of chip memory
        addr: u16,
    },
    IllegalOpcode {
        addr: u16,
        opcode: u16,
    },
    InvalidStackPointer {
        // If stack pointer is negative or larger than u8
        invalid_stack_pointer: i16,
    },
}

impl CPUError {
    pub fn throw(self) -> ! {
        match self {
            Self::IllegalROMAccess { addr } => panic!("Illegal ROM access at address: '{addr}'"),
            Self::IllegalRAMAccess {
                addr,
                ram_operation,
            } => {
                panic!(
                    "Illegal RAM access at address: '{addr}' doing operation: '{}'",
                    match ram_operation {
                        RAMOperation::Read => "Read",
                        RAMOperation::Write => "Write",
                    }
                )
            }
            Self::IllegalIOAccess { addr } => panic!("Illegal IO access at address: '{addr}'"),
            Self::IndirectPointer { addr } => {
                panic!("Indirect pointer outside of chip memory: '{addr}'")
            }
            Self::IllegalOpcode { addr, opcode } => {
                panic!("Illegal opcode '{opcode}' at address: '{addr}'")
            }
            Self::InvalidStackPointer {
                invalid_stack_pointer,
            } => {
                panic!("New stack pointer is illegal: '{invalid_stack_pointer}'")
            }
        }
    }
}

impl CPU {
    /// # Panics
    /// Panics when `ram_size_bytes` or `io_size_bytes` is above 256
    #[must_use]
    pub fn new(
        rom_size_words: usize,
        ram_size_bytes: usize,
        io_size_bytes: usize,
        arch: PDKArch,
    ) -> Self {
        assert!(ram_size_bytes <= 256);
        assert!(io_size_bytes <= 256);

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
            arch,
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

        let mut skip_cycles = 0;
        let mut i: u64 = 0;
        loop {
            let now_nanos = time::sys_nanos().unwrap();
            if (now_nanos - start_nanos) > self.freq_pause_nanos {
                i += 1;

                if i % self.freq_hz == 0 {
                    println!("Clock has ticked: {i} times");
                }

                if skip_cycles > 0 {
                    // TODO: Don't skip cycles if the emulation is running behind.
                    skip_cycles -= 1;
                    start_nanos = now_nanos;
                    continue;
                }

                // Run current instruction, -1 as the instruction handler always adds a clock cycle
                skip_cycles = (self.arch.instruction_handler)(self) - 1;

                start_nanos = now_nanos;
            }
        }
    }

    pub fn load_program(&mut self, program: &[u8]) -> Result<(), ProgramError> {
        // Multiply rom len by two to convert amount of words to amount of bytes
        ensure!(
            program.len() <= self.rom.len() * 2,
            ProgramTooLargeSnafu {
                program_len_bytes: program.len()
            }
        );

        // Clear ROM just in case
        for v in &mut self.rom {
            *v = 0x0000;
        }

        for (i, chunk) in program.chunks(2).enumerate() {
            let word: u16 = (u16::from(chunk[1]) << 8) | u16::from(chunk[0]);
            self.rom[i] = word;
        }

        Ok(())
    }

    #[must_use]
    #[inline]
    pub fn flags(&self) -> u8 {
        unsafe { std::ptr::read_volatile(&self.io[0x00]) }
    }

    #[inline]
    pub fn flags_set(&mut self, data: u8) {
        unsafe { std::ptr::write_volatile(&mut self.io[0x00], data) }
    }

    #[must_use]
    #[inline]
    pub fn stack_pointer(&self) -> u8 {
        unsafe { std::ptr::read_volatile(&self.io[0x02]) }
    }

    // Returns stack pointer to write the new stack item at (stack pointer before increment)
    #[inline]
    pub fn stack_pointer_inc(&mut self, inc: u8) -> u8 {
        let stack_pointer = self.stack_pointer();
        unsafe { std::ptr::write_volatile(&mut self.io[0x02], inc + stack_pointer) }
        stack_pointer
    }

    // Returns stack pointer to write the new stack item at (stack pointer after decrement)
    #[inline]
    pub fn stack_pointer_dec(&mut self, dec: u8) -> u8 {
        let new_stack_pointer = self.stack_pointer() - dec;
        unsafe { std::ptr::write_volatile(&mut self.io[0x02], new_stack_pointer) }
        new_stack_pointer
    }

    #[inline]
    fn stack_pointer_set(&mut self, new_stack_pointer: u8) {
        unsafe { std::ptr::write_volatile(&mut self.io[0x02], new_stack_pointer) }
    }

    #[inline]
    unsafe fn ram_get_unsafe(&self, addr: u8) -> u8 {
        self.ram[usize::from(addr)]
    }

    #[inline]
    unsafe fn ram_write_unsafe(&mut self, addr: u8, data: u8) {
        self.ram[usize::from(addr)] = data;
    }

    #[must_use]
    pub fn io_get(&self, addr: u8) -> u8 {
        if usize::from(addr) >= self.io.len() {
            CPUError::IllegalIOAccess { addr }.throw()
        }

        peripherals::read(self, addr)
    }

    pub fn io_write(&mut self, addr: u8, data: u8) {
        if usize::from(addr) >= self.io.len() {
            CPUError::IllegalIOAccess { addr }.throw()
        }

        peripherals::write(self, addr, data);
    }

    #[must_use]
    #[inline]
    pub fn ram_get(&self, addr: u8) -> u8 {
        let addr_usize = usize::from(addr);

        if addr_usize >= self.ram.len() {
            CPUError::IllegalRAMAccess {
                addr,
                ram_operation: RAMOperation::Read,
            }
            .throw()
        }

        self.ram[addr_usize]
    }

    #[inline]
    pub fn ram_write(&mut self, addr: u8, data: u8) {
        let addr_usize = usize::from(addr);

        if addr_usize >= self.ram.len() {
            CPUError::IllegalRAMAccess {
                addr,
                ram_operation: RAMOperation::Write,
            }
            .throw()
        }

        self.ram[addr_usize] = data;
    }

    /// # Panics
    /// Panics when stack pointer is somehow above i16 max value
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    pub fn stack_push8(&mut self, data: u8) {
        let new_stack_pointer = u16::from(self.stack_pointer()) + 1;

        if usize::from(new_stack_pointer) >= self.ram.len() {
            CPUError::InvalidStackPointer {
                invalid_stack_pointer: i16::try_from(new_stack_pointer).unwrap(),
            }
            .throw()
        }

        let new_stack_pointer = new_stack_pointer as u8;
        unsafe { self.ram_write_unsafe(new_stack_pointer - 1, data) }
        self.stack_pointer_set(new_stack_pointer);
    }

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    #[inline]
    pub fn stack_pop8(&mut self) -> u8 {
        let new_stack_pointer = i16::from(self.stack_pointer()) - 1;
        if new_stack_pointer < 0 {
            CPUError::InvalidStackPointer {
                invalid_stack_pointer: new_stack_pointer,
            }
            .throw()
        }

        let new_stack_pointer = new_stack_pointer as u8;
        self.stack_pointer_set(new_stack_pointer);
        unsafe { self.ram_get_unsafe(new_stack_pointer) }
    }

    /// # Panics
    /// Panics when stack pointer is somehow above i16 max value
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    pub fn stack_push16(&mut self, data: u16) {
        let new_stack_pointer = u16::from(self.stack_pointer()) + 2;

        if usize::from(new_stack_pointer) >= self.ram.len() {
            CPUError::InvalidStackPointer {
                invalid_stack_pointer: i16::try_from(new_stack_pointer).unwrap(),
            }
            .throw()
        }

        let new_stack_pointer = new_stack_pointer as u8;
        unsafe { self.ram_write_unsafe(new_stack_pointer - 2, data as u8) }
        unsafe { self.ram_write_unsafe(new_stack_pointer - 1, (data >> 8) as u8) }
        self.stack_pointer_set(new_stack_pointer);
    }

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    #[inline]
    pub fn stack_pop16(&mut self) -> u16 {
        let new_stack_pointer = i16::from(self.stack_pointer()) - 2;
        if new_stack_pointer < 0 {
            CPUError::InvalidStackPointer {
                invalid_stack_pointer: new_stack_pointer,
            }
            .throw()
        }

        let new_stack_pointer = new_stack_pointer as u8;
        self.stack_pointer_set(new_stack_pointer);
        let mut data: u16;
        data = unsafe { u16::from(self.ram_get_unsafe(new_stack_pointer + 1)) };
        data <<= 8;
        data |= unsafe { u16::from(self.ram_get_unsafe(new_stack_pointer)) };
        data
    }
}
