use solana_nostd_entrypoint::solana_program::pubkey::Pubkey;
use solana_program::program_error::ProgramError;

pub mod instructions;
pub mod invoke_signed;

pub const ID: Pubkey = solana_nostd_entrypoint::solana_program::pubkey!(
    "11111111111111111111111111111111"
);

pub type ProgramResult = Result<(), ProgramError>;