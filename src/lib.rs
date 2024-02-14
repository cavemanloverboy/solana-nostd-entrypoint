#![no_std]

pub use solana_program;

pub mod entrypoint_nostd;
pub use entrypoint_nostd::*;

#[cfg(feature = "example-program")]
pub mod entrypoint {
    use super::*;
    use solana_program::pubkey::Pubkey;

    #[no_mangle]
    pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
        let (program_id, accounts, instruction_data) =
            unsafe { crate::entrypoint_nostd::deserialize_nostd::<32>(input) };
        match process_instruction4c(&program_id, &accounts, &instruction_data) {
            Ok(()) => solana_program::entrypoint::SUCCESS,
            Err(error) => error.into(),
        }
    }

    pub const ID: Pubkey = solana_program::pubkey!("EWUt9PAjn26zCUALRRt56Gutaj52Bpb8ifbf7GZX3h1k");

    use solana_program::{entrypoint::ProgramResult, log, program_error::ProgramError};

    pub fn process_instruction4c(
        _program_id: &Pubkey,
        accounts: &[NoStdAccountInfo4],
        _data: &[u8],
    ) -> ProgramResult {
        log::sol_log("nostd_c");

        // Unpack accounts
        let [user, config, system_program, _rem @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        use solana_program::system_program;

        let mut instruction_data = [0; 12];
        instruction_data[0] = 2;
        instruction_data[4..12].copy_from_slice(&100_000_000_u64.to_le_bytes());

        let instruction_accounts = [user.to_meta_c(), config.to_meta_c()];

        let instruction = InstructionC {
            program_id: &system_program::ID,
            accounts: instruction_accounts.as_ptr(),
            accounts_len: instruction_accounts.len() as u64,
            data: instruction_data.as_ptr(),
            data_len: instruction_data.len() as u64,
        };
        let infos = [
            user.to_info_c(),
            config.to_info_c(),
            system_program.to_info_c(),
        ];
        let seeds: &[&[&[u8]]] = &[];
        #[cfg(target_os = "solana")]
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
        #[cfg(not(target_os = "solana"))]
        core::hint::black_box(&(&instruction, &infos, &seeds));

        Ok(())
    }
}
