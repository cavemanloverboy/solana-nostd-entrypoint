use solana_nostd_entrypoint::{AccountInfoC, InstructionC};

use crate::ProgramResult;

pub fn invoke_signed(
    instruction: &InstructionC,
    infos: &[AccountInfoC],
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    #[cfg(target_os = "solana")]
    unsafe {
        solana_program::syscalls::sol_invoke_signed_c(
            instruction as *const InstructionC as *const u8,
            infos.as_ptr() as *const u8,
            infos.len() as u64,
            seeds.as_ptr() as *const u8,
            seeds.len() as u64,
        );
    }

    // For clippy
    #[cfg(not(target_os = "solana"))]
    core::hint::black_box(&(&instruction, &infos, &seeds));

    Ok(())
}
