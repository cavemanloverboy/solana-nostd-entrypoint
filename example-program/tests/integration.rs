use std::path::PathBuf;

use litesvm::LiteSVM;
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer,
    transaction::Transaction,
};

fn read_program() -> Vec<u8> {
    let mut so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    so_path.push("../target/deploy/solana_nostd_example_program.so");

    std::fs::read(so_path).unwrap()
}

#[test]
fn no_std_integration() {
    let mut svm = LiteSVM::new();

    let payer = Keypair::new();
    let payer_pk = payer.pubkey();

    svm.airdrop(&payer_pk, LAMPORTS_PER_SOL)
        .unwrap();

    let program_id = solana_nostd_example_program::ID;
    let program_bytes = read_program();
    svm.add_program(program_id, &program_bytes);

    let other_user = Keypair::new();
    let accounts = [
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(other_user.pubkey(), false),
        AccountMeta::new_readonly(Pubkey::default(), false),
    ];
    let instruction = Instruction {
        program_id,
        accounts: accounts.to_vec(),
        data: vec![],
    };
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer_pk),
        &[&payer],
        svm.latest_blockhash(),
    );

    svm.send_transaction(transaction)
        .unwrap();
}
