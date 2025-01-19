use solana_nostd_entrypoint::{
    solana_program::entrypoint::ProgramResult, InstructionC,
    NoStdAccountInfo,
};

use crate::invoke_signed::invoke_signed;

pub struct SyncNative<'a> {
    pub token_program: &'a NoStdAccountInfo,
    pub token: &'a NoStdAccountInfo,
}

impl<'a> SyncNative<'a> {
    pub fn invoke_signed(
        &self,
        signer_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        let account_metas = [self.token.to_meta_c()];
        let account_infos = [self.token.to_info_c()];

        let instruction_data = &[17];

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
