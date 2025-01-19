use solana_nostd_entrypoint::{
    solana_program::entrypoint::ProgramResult, InstructionC,
    NoStdAccountInfo,
};

use crate::invoke_signed::invoke_signed;

pub struct Transfer<'a> {
    pub token_program: &'a NoStdAccountInfo,
    pub from: &'a NoStdAccountInfo,
    pub to: &'a NoStdAccountInfo,
    pub authority: &'a NoStdAccountInfo,
    pub amount: u64,
}

impl<'a> Transfer<'a> {
    pub fn invoke_signed(
        &self,
        signer_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        let account_metas = [
            self.from.to_meta_c(),
            self.to.to_meta_c(),
            self.authority.to_meta_c_signer(),
        ];
        let account_infos = [
            self.from.to_info_c(),
            self.to.to_info_c(),
            self.authority.to_info_c(),
            self.token_program.to_info_c(),
        ];

        let mut instruction_data = [0u8; 9];
        instruction_data[0] = 3; // Transfer instruction discriminator
        instruction_data[1..9]
            .copy_from_slice(&self.amount.to_le_bytes());

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
