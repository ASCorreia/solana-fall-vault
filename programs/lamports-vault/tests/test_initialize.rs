mod common;

use {
    common::{
        build_initialize_ix, fund, send, setup_svm, vault_pda, vault_state_pda, ONE_SOL,
    },
    solana_keypair::Keypair,
    solana_signer::Signer,
};

#[test]
fn initialize_creates_vault_state_and_funds_vault() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    fund(&mut svm, &payer.pubkey(), 10 * ONE_SOL);

    let (vault_state, expected_state_bump) = vault_state_pda(&payer.pubkey());
    let (vault, expected_vault_bump) = vault_pda(&payer.pubkey());

    let ix = build_initialize_ix(&payer.pubkey());
    send(&mut svm, &payer, &[ix], &[]).expect("initialize should succeed");

    let state_account = svm
        .get_account(&vault_state)
        .expect("vault_state account should exist");
    assert_eq!(state_account.owner, lamports_vault::id());
    assert!(state_account.lamports > 0, "vault_state must be rent-exempt");

    // Bytes 0..8 are the Anchor discriminator; bytes 8 and 9 hold the bumps in
    // VaultState field order: `vault_bump` (byte 8) then `bump` (byte 9).
    let data = &state_account.data;
    assert!(data.len() >= 10, "vault_state account too small");
    assert_eq!(data[8], expected_vault_bump, "vault_bump mismatch");
    assert_eq!(data[9], expected_state_bump, "state_bump mismatch");

    let vault_balance = svm.get_balance(&vault).unwrap_or_default();
    assert!(
        vault_balance > 0,
        "vault should have been funded with rent-exempt lamports"
    );
}

#[test]
fn initialize_twice_for_same_payer_fails() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    fund(&mut svm, &payer.pubkey(), 10 * ONE_SOL);

    let ix = build_initialize_ix(&payer.pubkey());
    send(&mut svm, &payer, &[ix.clone()], &[]).expect("first initialize should succeed");

    // Re-initializing with the same payer reuses the same PDAs and must fail
    // because `init` requires the account to not already exist.
    let result = send(&mut svm, &payer, &[ix], &[]);
    assert!(
        result.is_err(),
        "initializing the same vault twice must fail"
    );
}

#[test]
fn initialize_with_separate_payers_creates_independent_vaults() {
    let mut svm = setup_svm();
    let alice = Keypair::new();
    let bob = Keypair::new();
    fund(&mut svm, &alice.pubkey(), 10 * ONE_SOL);
    fund(&mut svm, &bob.pubkey(), 10 * ONE_SOL);

    send(&mut svm, &alice, &[build_initialize_ix(&alice.pubkey())], &[])
        .expect("alice init should succeed");
    send(&mut svm, &bob, &[build_initialize_ix(&bob.pubkey())], &[])
        .expect("bob init should succeed");

    let (alice_vault, _) = vault_pda(&alice.pubkey());
    let (bob_vault, _) = vault_pda(&bob.pubkey());
    assert_ne!(alice_vault, bob_vault, "vaults should be independent PDAs");
    assert!(svm.get_account(&alice_vault).is_some());
    assert!(svm.get_account(&bob_vault).is_some());
}
