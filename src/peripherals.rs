use crate::cpu::CPU;

// Assumes that `addr`` is within range of IO address space
#[must_use]
pub fn read(cpu: &CPU, addr: u8) -> u8 {
    // TODO: Actually read from peripheral
    println!("Periphal read: addr: {addr}");
    cpu.io[usize::from(addr)]
}

// Assumes that `addr`` is within range of IO address space
pub fn write(cpu: &mut CPU, addr: u8, data: u8) {
    // TODO: Actually write to periphal
    println!("Periphal write: addr: {addr}, data: {data}");
    cpu.io[usize::from(addr)] = data;
}
