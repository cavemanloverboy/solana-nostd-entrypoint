#![cfg(feature = "example-program")]
use solana_program::{
    instruction::{AccountMeta, Instruction},
};
use solana_program_test::ProgramTest;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

#[tokio::test(flavor = "current_thread")]
async fn no_std_integration() {
    let mut program_test = ProgramTest::new(
        "solana_nostd_entrypoint",
        solana_nostd_entrypoint::entrypoint::ID,
        None,
    );
    program_test.prefer_bpf(true);
    let (mut banks, payer, recent_blockhash) = program_test.start().await;

    let other_user = Keypair::new();
    let accounts = [
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(other_user.pubkey(), false),
        AccountMeta::new_readonly(solana_program::system_program::ID, false),
    ];
    let instruction = Instruction {
        program_id: solana_nostd_entrypoint::entrypoint::ID,
        accounts: accounts.to_vec(),
        data: vec![],
    };
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    banks.process_transaction(transaction).await.unwrap();
}
