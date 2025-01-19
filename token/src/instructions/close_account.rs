use solana_nostd_entrypoint::{
    solana_program::entrypoint::ProgramResult, InstructionC,
    NoStdAccountInfo,
};

use crate::invoke_signed::invoke_signed;

pub struct CloseAccount<'a> {
    pub token_program: &'a NoStdAccountInfo,
    pub account: &'a NoStdAccountInfo,
    pub destination: &'a NoStdAccountInfo,
    pub authority: &'a NoStdAccountInfo,
}

impl<'a> CloseAccount<'a> {
    pub fn invoke_signed(
        &self,
        signer_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        let instruction_data = &[9];

        let account_metas = [
            self.account.to_meta_c(),
            self.destination.to_meta_c(),
            self.authority.to_meta_c_signer(),
        ];

        let account_infos = [
            self.account.to_info_c(),
            self.destination.to_info_c(),
            self.authority.to_info_c(),
        ];

        let instruction = InstructionC {
            program_id: self.token_program.key(),
            accounts: account_metas.as_ptr(),
            accounts_len: account_metas.len() as u64,
            data: instruction_data.as_ptr(),
            data_len: instruction_data.len() as u64,
        };

        invoke_signed(&instruction, &account_infos, signer_seeds)
    }
}
