#![allow(dead_code)]

use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        InstructionData, ToAccountMetas,
    },
    litesvm::{
        types::{FailedTransactionMetadata, TransactionMetadata},
        LiteSVM,
    },
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

pub const VAULT_STATE_SEED: &[u8] = b"vault_state";
pub const VAULT_SEED: &[u8] = b"vault";
pub const ONE_SOL: u64 = 1_000_000_000;
pub const DEFAULT_MAX_WITHDRAW: u64 = 5 * ONE_SOL;


pub fn setup_svm() -> LiteSVM {
    let program_id = lamports_vault::id();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../../target/deploy/lamports_vault.so");
    svm.add_program(program_id, bytes).unwrap();
    svm
}

pub fn fund(svm: &mut LiteSVM, pubkey: &Pubkey, lamports: u64) {
    svm.airdrop(pubkey, lamports).unwrap();
}

pub fn vault_state_pda(user: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[VAULT_STATE_SEED, user.as_ref()], &lamports_vault::id())
}

pub fn vault_pda(user: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[VAULT_SEED, user.as_ref()], &lamports_vault::id())
}

pub fn build_initialize_ix(payer: &Pubkey,  max_withdraw: u64) -> Instruction {
    let (vault_state, _) = vault_state_pda(payer);
    let (vault, _) = vault_pda(payer);

    Instruction::new_with_bytes(
        lamports_vault::id(),
        &lamports_vault::instruction::Initialize { max_withdraw }.data(),
        lamports_vault::accounts::Initialize {
            user: *payer,
            vault_state,
            vault,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

pub fn build_deposit_ix(user: &Pubkey, amount: u64) -> Instruction {
    let (vault_state, _) = vault_state_pda(user);
    let (vault, _) = vault_pda(user);

    Instruction::new_with_bytes(
        lamports_vault::id(),
        &lamports_vault::instruction::Deposit { amount }.data(),
        lamports_vault::accounts::Deposit {
            user: *user,
            vault,
            vault_state,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

pub fn build_withdraw_ix(user: &Pubkey, amount: u64) -> Instruction {
    let (vault_state, _) = vault_state_pda(user);
    let (vault, _) = vault_pda(user);

    Instruction::new_with_bytes(
        lamports_vault::id(),
        &lamports_vault::instruction::Withdraw { amount }.data(),
        lamports_vault::accounts::Withdraw {
            user: *user,
            vault,
            vault_state,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

pub fn send(
    svm: &mut LiteSVM,
    payer: &Keypair,
    instructions: &[Instruction],
    extra_signers: &[&Keypair],
) -> Result<TransactionMetadata, FailedTransactionMetadata> {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(instructions, Some(&payer.pubkey()), &blockhash);

    let mut signers: Vec<&Keypair> = Vec::with_capacity(1 + extra_signers.len());
    signers.push(payer);
    signers.extend_from_slice(extra_signers);

    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &signers).unwrap();
    svm.send_transaction(tx)
}

pub fn initialize_vault(svm: &mut LiteSVM, payer: &Keypair, max_withdraw: u64) {
    let ix = build_initialize_ix(&payer.pubkey(), max_withdraw);
    send(svm, payer, &[ix], &[]).expect("initialize should succeed");
}
