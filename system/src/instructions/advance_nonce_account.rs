use solana_nostd_entrypoint::{InstructionC, NoStdAccountInfo};

use crate::{invoke_signed::invoke_signed, ProgramResult};

pub struct AdvanceNonceAccount<'a> {
    /// Nonce account.
    pub account: &'a NoStdAccountInfo,

    /// RecentBlockhashes sysvar.
    pub recent_blockhashes_sysvar: &'a NoStdAccountInfo,

    /// Nonce authority.
    pub authority: &'a NoStdAccountInfo,
}

impl<'a> AdvanceNonceAccount<'a> {
    pub fn invoke_signed(
        &self,
        signer_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        let account_metas = [
            self.account.to_meta_c(),
            self.recent_blockhashes_sysvar
                .to_meta_c(),
            self.authority.to_meta_c_signer(),
        ];

        let account_infos = [
            self.account.to_info_c(),
            self.recent_blockhashes_sysvar
                .to_info_c(),
            self.authority.to_info_c(),
        ];

        let mut instruction_data = [0; 4];
        instruction_data[0] = 4;

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
