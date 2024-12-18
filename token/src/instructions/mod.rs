pub mod close_account;
pub mod initialize_account_3;
pub mod sync_native;
pub mod transfer;
pub mod transfer_checked;

pub mod token_program {
    use solana_nostd_entrypoint::solana_program::pubkey::Pubkey;

    pub const ID: Pubkey = solana_nostd_entrypoint::solana_program::pubkey!(
        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    );
}

pub mod token_2022 {
    use solana_nostd_entrypoint::solana_program::pubkey::Pubkey;

    pub const ID: Pubkey = solana_nostd_entrypoint::solana_program::pubkey!(
        "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
    );
}
