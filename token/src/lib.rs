use solana_nostd_entrypoint::solana_program::pubkey::Pubkey;
use solana_program::program_error::ProgramError;

pub mod instructions;
pub mod invoke_signed;

pub const ID: Pubkey = solana_nostd_entrypoint::solana_program::pubkey!(
    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
);

pub type ProgramResult = Result<(), ProgramError>;
