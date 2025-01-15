#![allow(unexpected_cfgs)]
use {
    solana_msg::sol_log,
    solana_nostd_entrypoint::{
        basic_panic_impl, entrypoint_nostd, noalloc_allocator,
        InstructionC, NoStdAccountInfo,
    },
    solana_program_error::{ProgramError, ProgramResult},
    solana_pubkey::Pubkey,
};

const SYS_PROGRAM_ID: Pubkey =
    solana_pubkey::pubkey!("11111111111111111111111111111111");

entrypoint_nostd!(process_instruction, 32);

solana_pubkey::declare_id!(
    "EWUt9PAjn26zCUALRRt56Gutaj52Bpb8ifbf7GZX3h1k"
);

noalloc_allocator!();
basic_panic_impl!();

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[NoStdAccountInfo],
    _data: &[u8],
) -> ProgramResult {
    sol_log("nostd");

    // Unpack accounts
    let [user, config, _rem @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Transfer has discriminant 2_u32 (little endian), followed u64
    // lamport amount
    let mut instruction_data = [0; 12];
    instruction_data[0] = 2;
    instruction_data[4..12]
        .copy_from_slice(&100_000_000_u64.to_le_bytes());

    // Instruction accounts are are from, to
    let instruction_accounts = [user.to_meta_c(), config.to_meta_c()];

    // Build instruction expected by sol_invoke_signed_c
    let instruction = InstructionC {
        program_id: &SYS_PROGRAM_ID,
        accounts: instruction_accounts.as_ptr(),
        accounts_len: instruction_accounts.len() as u64,
        data: instruction_data.as_ptr(),
        data_len: instruction_data.len() as u64,
    };

    // Get infos and seeds
    let infos = [user.to_info_c(), config.to_info_c()];
    let seeds: &[&[&[u8]]] = &[];

    // Invoke system program
    #[cfg(target_os = "solana")]
    unsafe {
        solana_program::syscalls::sol_invoke_signed_c(
            &instruction as *const InstructionC as *const u8,
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
