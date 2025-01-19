use solana_nostd_entrypoint::{InstructionC, NoStdAccountInfo};

use crate::{invoke_signed::invoke_signed, ProgramResult};

pub struct Transfer<'a> {
    pub from: &'a NoStdAccountInfo,
    pub to: &'a NoStdAccountInfo,
    pub lamports: u64,
}

impl<'a> Transfer<'a> {
    pub fn invoke_signed(
        &self,
        signer_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        let account_metas =
            [self.from.to_meta_c_signer(), self.to.to_meta_c()];
        let account_infos =
            [self.from.to_info_c(), self.to.to_info_c()];

        let mut instruction_data = [0; 12];
        instruction_data[0] = 2;
        instruction_data[4..12]
            .copy_from_slice(&self.lamports.to_le_bytes());

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
