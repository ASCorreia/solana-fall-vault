mod common;

use {
    crate::common::vault_state_pda,
    common::{build_close_ix, fund, initialize_vault, send, setup_svm, vault_pda, ONE_SOL},
    solana_keypair::Keypair,
    solana_signer::Signer,
};

use anchor_lang::Discriminator;
use anchor_lang::Space;
use lamports_vault::VaultState;

#[test]
fn close_vault() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    initialize_vault(&mut svm, &user);

    let (vault, _) = vault_pda(&user.pubkey());

    let (vault_state, _) = vault_state_pda(&user.pubkey());

    let vault_before = svm.get_balance(&vault).unwrap_or_default();
    let user_before = svm.get_balance(&user.pubkey()).unwrap_or_default();
    let vault_state_before = svm.get_balance(&vault_state).unwrap_or_default();

    const FEE: u64 = 5000; // 1 signature

    let state_len = VaultState::DISCRIMINATOR.len() + VaultState::INIT_SPACE;
    let state_rent = svm.minimum_balance_for_rent_exemption(state_len);

    let vault_rent = svm.minimum_balance_for_rent_exemption(0);

    println!(
        "BEFORE user: {}, vault: {}, vault_rent: {}",
        user_before, vault_before, vault_rent
    );

    let ix = build_close_ix(&user.pubkey());
    send(&mut svm, &user, &[ix], &[]).expect("close should succeed");

    let vault_after = svm.get_balance(&vault).unwrap_or_default();
    let user_after = svm.get_balance(&user.pubkey()).unwrap_or_default();
    let vault_state_after = svm.get_balance(&vault_state).unwrap_or_default();

    println!(
        "AFTER user: {}, vault: {}, {}",
        user_after, vault_after, vault_rent
    );

    assert_eq!(
        vault_state_before, state_rent,
        "Vault state should only hold rent required"
    );

    // Invariant 1: lamports are conserved, minus the tx fee
    assert_eq!(
        vault_before + vault_state_before + user_before,
        vault_after + vault_state_after + user_after + FEE,
        "lamports not conserved"
    );

    // Invariant 2: both PDAs drained
    assert_eq!(vault_after, 0, "vault should be empty");
    assert_eq!(vault_state_after, 0, "vault_state should be closed");

    //Invariant 3 user should have greater amount than before
    assert!(
        user_after > user_before,
        "User should have more amount than when he started"
    );
}
