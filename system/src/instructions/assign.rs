use solana_nostd_entrypoint::{InstructionC, NoStdAccountInfo};
use solana_program::pubkey::Pubkey;

use crate::{invoke_signed::invoke_signed, ProgramResult};

pub struct Assign<'a> {
    pub account: &'a NoStdAccountInfo,
    pub owner: &'a Pubkey,
}

impl<'a> Assign<'a> {
    pub fn invoke_signed(
        &self,
        signer_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        let account_metas = [self.account.to_meta_c_signer()];
        let account_infos = [self.account.to_info_c()];

        let mut instruction_data = [0; 36];
        instruction_data[0] = 1;
        instruction_data[4..36].copy_from_slice(self.owner.as_ref());

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
