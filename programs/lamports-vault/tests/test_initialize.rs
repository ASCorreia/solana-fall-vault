mod common;

use {
    anchor_lang::AnchorDeserialize,
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

    let ix = build_initialize_ix(&payer.pubkey(), 2 * ONE_SOL);
    send(&mut svm, &payer, &[ix], &[]).expect("initialize should succeed");

    let state_account = svm
        .get_account(&vault_state)
        .expect("vault_state account should exist");
    assert_eq!(state_account.owner, lamports_vault::id());
    assert!(state_account.lamports > 0, "vault_state must be rent-exempt");

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
fn initialize_stores_max_withdraw_in_vault_state() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    fund(&mut svm, &payer.pubkey(), 10 * ONE_SOL);

    let max_withdraw = 3 * ONE_SOL;
    let ix = build_initialize_ix(&payer.pubkey(), max_withdraw);
    send(&mut svm, &payer, &[ix], &[]).expect("initialize should succeed");

    let (vault_state, _) = vault_state_pda(&payer.pubkey());
    let account = svm.get_account(&vault_state).expect("vault_state must exist");
    let state =
        lamports_vault::VaultState::deserialize(&mut &account.data[8..]).unwrap();
    assert_eq!(
        state.max_withdraw, max_withdraw,
        "max_withdraw should be stored exactly as provided"
    );
}

#[test]
fn initialize_with_zero_max_withdraw_stores_zero() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    fund(&mut svm, &payer.pubkey(), 10 * ONE_SOL);

    let ix = build_initialize_ix(&payer.pubkey(), 0);
    send(&mut svm, &payer, &[ix], &[]).expect("initialize with zero max_withdraw should succeed");

    let (vault_state, _) = vault_state_pda(&payer.pubkey());
    let account = svm.get_account(&vault_state).expect("vault_state must exist");
    let state =
        lamports_vault::VaultState::deserialize(&mut &account.data[8..]).unwrap();
    assert_eq!(state.max_withdraw, 0);
}

#[test]
fn initialize_twice_for_same_payer_fails() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    fund(&mut svm, &payer.pubkey(), 10 * ONE_SOL);

    let ix = build_initialize_ix(&payer.pubkey(), ONE_SOL);
    send(&mut svm, &payer, &[ix.clone()], &[]).expect("first initialize should succeed");

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

    send(&mut svm, &alice, &[build_initialize_ix(&alice.pubkey(), ONE_SOL)], &[])
        .expect("alice init should succeed");
    send(&mut svm, &bob, &[build_initialize_ix(&bob.pubkey(), 2 * ONE_SOL)], &[])
        .expect("bob init should succeed");

    let (alice_vault, _) = vault_pda(&alice.pubkey());
    let (bob_vault, _) = vault_pda(&bob.pubkey());
    assert_ne!(alice_vault, bob_vault, "vaults should be independent PDAs");
    assert!(svm.get_account(&alice_vault).is_some());
    assert!(svm.get_account(&bob_vault).is_some());

    let (alice_state_pk, _) = vault_state_pda(&alice.pubkey());
    let (bob_state_pk, _) = vault_state_pda(&bob.pubkey());
    let alice_state = lamports_vault::VaultState::deserialize(
        &mut &svm.get_account(&alice_state_pk).unwrap().data[8..],
    )
    .unwrap();
    let bob_state = lamports_vault::VaultState::deserialize(
        &mut &svm.get_account(&bob_state_pk).unwrap().data[8..],
    )
    .unwrap();
    assert_eq!(alice_state.max_withdraw, ONE_SOL);
    assert_eq!(bob_state.max_withdraw, 2 * ONE_SOL);
}
