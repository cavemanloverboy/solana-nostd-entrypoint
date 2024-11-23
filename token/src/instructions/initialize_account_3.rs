use solana_nostd_entrypoint::{
    solana_program::entrypoint::ProgramResult,
    solana_program::pubkey::Pubkey, InstructionC, NoStdAccountInfo,
};

use crate::invoke_signed::invoke_signed;

pub struct InitializeAccount3<'a> {
    pub token_program: &'a NoStdAccountInfo,
    pub token: &'a NoStdAccountInfo,
    pub mint: &'a NoStdAccountInfo,
    pub owner: &'a Pubkey,
}

impl<'a> InitializeAccount3<'a> {
    pub fn invoke_signed(
        &self,
        signer_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        let account_metas =
            [self.token.to_meta_c(), self.mint.to_meta_c()];
        let account_infos =
            [self.token.to_info_c(), self.mint.to_info_c()];

        // Preallocate vec with exact size: 1 (discriminator) + 32 (pubkey)
        let mut instruction_data = Vec::with_capacity(33);
        instruction_data.push(18);
        instruction_data.extend_from_slice(self.owner.as_ref());

        let instruction = &InstructionC {
            program_id: self.token_program.key(),
            accounts: account_metas.as_ptr(),
            accounts_len: account_metas.len() as u64,
            data: instruction_data.as_slice().as_ptr(),
            data_len: instruction_data.len() as u64,
        };

        invoke_signed(&instruction, &account_infos, signer_seeds)
    }
}
