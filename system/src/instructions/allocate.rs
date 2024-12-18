use solana_nostd_entrypoint::{InstructionC, NoStdAccountInfo};

use crate::{invoke_signed::invoke_signed, ProgramResult};

pub struct Allocate<'a> {
    pub account: &'a NoStdAccountInfo,
    pub space: u64,
}

impl<'a> Allocate<'a> {
    pub fn invoke_signed(
        &self,
        signer_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        let account_metas = [self.account.to_meta_c_signer()];
        let account_infos = [self.account.to_info_c()];

        let mut instruction_data = [0; 12];
        instruction_data[0] = 8;
        instruction_data[4..12]
            .copy_from_slice(&self.space.to_le_bytes());

        let instruction = InstructionC {
            program_id: &crate::ID,
            accounts: account_metas.as_ptr(),
            accounts_len: account_metas.len() as u64,
            data: instruction_data.as_ptr(),
            data_len: instruction_data.len() as u64,
        };

        invoke_signed(&instruction, &account_infos, signer_seeds)
    }
}
