use solana_nostd_entrypoint::{
    solana_program::pubkey::Pubkey, InstructionC, NoStdAccountInfo,
};

use crate::{invoke_signed::invoke_signed, ProgramResult};

pub struct CreateAccount<'a> {
    pub from: &'a NoStdAccountInfo,
    pub to: &'a NoStdAccountInfo,
    pub lamports: u64,
    pub space: u64,
    pub owner: &'a Pubkey,
}

impl<'a> CreateAccount<'a> {
    pub fn invoke_signed(
        &self,
        signer_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        let account_metas =
            [self.from.to_meta_c_signer(), self.to.to_meta_c_signer()];
        let account_infos =
            [self.from.to_info_c(), self.to.to_info_c()];

        let mut instruction_data = [0; 52];
        // create account instruction has a '0' discriminator
        instruction_data[4..12]
            .copy_from_slice(&self.lamports.to_le_bytes());
        instruction_data[12..20]
            .copy_from_slice(&self.space.to_le_bytes());
        instruction_data[20..52].copy_from_slice(self.owner.as_ref());

        let instruction = InstructionC {
            program_id: &crate::ID,
            accounts: account_metas.as_ptr(),
            accounts_len: account_metas.len() as u64,
            data: instruction_data.as_slice().as_ptr(),
            data_len: instruction_data.len() as u64,
        };

        invoke_signed(&instruction, &account_infos, signer_seeds)
    }
}
