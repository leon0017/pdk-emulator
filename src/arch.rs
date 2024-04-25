use crate::cpu::CPU;

pub mod pdk14;

// Returns amount of clock cycles the instruction should take (For time emulation)
pub type InstructionHandler = fn(&mut CPU) -> u32;

#[allow(clippy::module_name_repetitions)]
pub struct PDKArch {
    pub instruction_handler: InstructionHandler,
}

#[must_use]
pub fn pdk14() -> PDKArch {
    PDKArch {
        instruction_handler: pdk14::instruction_handler,
    }
}
