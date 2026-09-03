mod common;

use {
    common::{
        build_deposit_ix,
        build_withdraw_ix,
        fund,
        initialize_vault,
        send,
        setup_svm,
        vault_pda,
        ONE_SOL,
    },
    solana_keypair::Keypair,
    solana_signer::Signer,
};

#[test]
fn withdraw_under_max_withdraw_succeeds() {
    let mut svm = setup_svm();

    let user = Keypair::new();

    fund(
        &mut svm,
        &user.pubkey(),
        10 * ONE_SOL,
    );

    // This vault has a 2 SOL withdrawal ceiling.
    initialize_vault(
        &mut svm,
        &user,
        2 * ONE_SOL,
    );

    // First deposit enough funds into the vault.
    let deposit_amount = 5 * ONE_SOL;

    let deposit_ix = build_deposit_ix(
        &user.pubkey(),
        deposit_amount,
    );

    send(
        &mut svm,
        &user,
        &[deposit_ix],
        &[],
    )
    .expect("deposit should succeed");

    let (vault, _) = vault_pda(&user.pubkey());

    let vault_before = svm
        .get_balance(&vault)
        .unwrap();

    let user_before = svm
        .get_balance(&user.pubkey())
        .unwrap();

    // Withdraw 1 SOL, which is below the 2 SOL limit.
    let withdraw_amount = ONE_SOL;

    let withdraw_ix = build_withdraw_ix(
        &user.pubkey(),
        withdraw_amount,
    );

    send(
        &mut svm,
        &user,
        &[withdraw_ix],
        &[],
    )
    .expect("withdrawal below max_withdraw should succeed");

    let vault_after = svm
        .get_balance(&vault)
        .unwrap();

    let user_after = svm
        .get_balance(&user.pubkey())
        .unwrap();

    assert_eq!(
        vault_before - vault_after,
        withdraw_amount,
        "vault should lose exactly the withdrawn amount"
    );

    // The user receives the withdrawal amount.
    // Transaction fees are also deducted from the user's balance,
    // so check that the net increase is at least the withdrawal.
    assert!(
        user_after + withdraw_amount >= user_before,
        "user should receive the withdrawn lamports"
    );
}

#[test]
fn withdraw_exactly_at_max_withdraw_succeeds() {
    let mut svm = setup_svm();

    let user = Keypair::new();

    fund(
        &mut svm,
        &user.pubkey(),
        10 * ONE_SOL,
    );

    // Set the per-vault withdrawal limit to exactly 2 SOL.
    let max_withdraw = 2 * ONE_SOL;

    initialize_vault(
        &mut svm,
        &user,
        max_withdraw,
    );

    // Deposit enough to cover the withdrawal.
    let deposit_amount = 5 * ONE_SOL;

    let deposit_ix = build_deposit_ix(
        &user.pubkey(),
        deposit_amount,
    );

    send(
        &mut svm,
        &user,
        &[deposit_ix],
        &[],
    )
    .expect("deposit should succeed");

    let (vault, _) = vault_pda(&user.pubkey());

    let vault_before = svm
        .get_balance(&vault)
        .unwrap();

    // Withdraw exactly the configured maximum.
    let withdraw_ix = build_withdraw_ix(
        &user.pubkey(),
        max_withdraw,
    );

    send(
        &mut svm,
        &user,
        &[withdraw_ix],
        &[],
    )
    .expect("withdrawal exactly at max_withdraw should succeed");

    let vault_after = svm
        .get_balance(&vault)
        .unwrap();

    assert_eq!(
        vault_before - vault_after,
        max_withdraw,
        "vault should lose exactly max_withdraw lamports"
    );
}

#[test]
fn withdraw_one_lamport_above_max_withdraw_fails() {
    let mut svm = setup_svm();

    let user = Keypair::new();

    fund(
        &mut svm,
        &user.pubkey(),
        10 * ONE_SOL,
    );

    let max_withdraw = 2 * ONE_SOL;

    initialize_vault(
        &mut svm,
        &user,
        max_withdraw,
    );

    // Deposit enough funds so the failure is specifically caused
    // by max_withdraw rather than insufficient vault balance.
    let deposit_amount = 5 * ONE_SOL;

    let deposit_ix = build_deposit_ix(
        &user.pubkey(),
        deposit_amount,
    );

    send(
        &mut svm,
        &user,
        &[deposit_ix],
        &[],
    )
    .expect("deposit should succeed");

    let (vault, _) = vault_pda(&user.pubkey());

    let vault_before = svm
        .get_balance(&vault)
        .unwrap();

    // One lamport above the configured limit.
    let withdraw_amount = max_withdraw + 1;

    let withdraw_ix = build_withdraw_ix(
        &user.pubkey(),
        withdraw_amount,
    );

    let result = send(
        &mut svm,
        &user,
        &[withdraw_ix],
        &[],
    );

    assert!(
        result.is_err(),
        "withdrawal one lamport above max_withdraw must fail"
    );

    let vault_after = svm
        .get_balance(&vault)
        .unwrap();

    // Because the instruction failed, the vault balance must remain unchanged.
    assert_eq!(
        vault_after,
        vault_before,
        "failed withdrawal must not change the vault balance"
    );
}

#[test]
fn different_vaults_can_have_different_withdrawal_limits() {
    let mut svm = setup_svm();

    let user_a = Keypair::new();
    let user_b = Keypair::new();

    fund(
        &mut svm,
        &user_a.pubkey(),
        10 * ONE_SOL,
    );

    fund(
        &mut svm,
        &user_b.pubkey(),
        10 * ONE_SOL,
    );

    // Each vault gets its own withdrawal ceiling.
    let max_withdraw_a = ONE_SOL;
    let max_withdraw_b = 3 * ONE_SOL;

    initialize_vault(
        &mut svm,
        &user_a,
        max_withdraw_a,
    );

    initialize_vault(
        &mut svm,
        &user_b,
        max_withdraw_b,
    );

    // Give both vaults enough funds.
    let deposit_a = build_deposit_ix(
        &user_a.pubkey(),
        5 * ONE_SOL,
    );

    let deposit_b = build_deposit_ix(
        &user_b.pubkey(),
        5 * ONE_SOL,
    );

    send(
        &mut svm,
        &user_a,
        &[deposit_a],
        &[],
    )
    .expect("user A deposit should succeed");

    send(
        &mut svm,
        &user_b,
        &[deposit_b],
        &[],
    )
    .expect("user B deposit should succeed");

    // User A can withdraw 1 SOL because their limit is 1 SOL.
    let withdraw_a = build_withdraw_ix(
        &user_a.pubkey(),
        ONE_SOL,
    );

    send(
        &mut svm,
        &user_a,
        &[withdraw_a],
        &[],
    )
    .expect("user A withdrawal should succeed");

    // User B can withdraw 3 SOL because their limit is 3 SOL.
    let withdraw_b = build_withdraw_ix(
        &user_b.pubkey(),
        3 * ONE_SOL,
    );

    send(
        &mut svm,
        &user_b,
        &[withdraw_b],
        &[],
    )
    .expect("user B withdrawal should succeed");
}

#[test]
fn withdraw_without_initialize_fails() {
    let mut svm = setup_svm();

    let user = Keypair::new();

    fund(
        &mut svm,
        &user.pubkey(),
        10 * ONE_SOL,
    );

    // The vault has not been initialized, so VaultState does not exist.
    let withdraw_ix = build_withdraw_ix(
        &user.pubkey(),
        ONE_SOL,
    );

    let result = send(
        &mut svm,
        &user,
        &[withdraw_ix],
        &[],
    );

    assert!(
        result.is_err(),
        "withdrawal without initialization must fail"
    );
}

#[test]
fn withdraw_more_than_vault_balance_fails() {
    let mut svm = setup_svm();

    let user = Keypair::new();

    fund(
        &mut svm,
        &user.pubkey(),
        10 * ONE_SOL,
    );

    // Set a high withdrawal ceiling so that the expected failure
    // comes from insufficient vault balance.
    initialize_vault(
        &mut svm,
        &user,
        100 * ONE_SOL,
    );

    // Only deposit 1 SOL.
    let deposit_ix = build_deposit_ix(
        &user.pubkey(),
        ONE_SOL,
    );

    send(
        &mut svm,
        &user,
        &[deposit_ix],
        &[],
    )
    .expect("deposit should succeed");

    // Try to withdraw 2 SOL when only 1 SOL was deposited.
    let withdraw_ix = build_withdraw_ix(
        &user.pubkey(),
        2 * ONE_SOL,
    );

    let result = send(
        &mut svm,
        &user,
        &[withdraw_ix],
        &[],
    );

    assert!(
        result.is_err(),
        "withdrawal greater than vault balance must fail"
    );
}
