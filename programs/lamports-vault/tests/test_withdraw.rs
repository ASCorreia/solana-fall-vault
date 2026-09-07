mod common;

use {
    common::{
        build_deposit_ix, build_withdraw_ix, fund, initialize_vault, send, setup_svm, vault_pda,
        ONE_SOL,
    },
    solana_keypair::Keypair,
    solana_signer::Signer,
};

#[test]
fn withdraw_returns_lamports_to_user() {
    let mut svm = setup_svm();
    let user = Keypair::new();

    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    // Maximum withdrawal: 1 SOL
    initialize_vault(&mut svm, &user, ONE_SOL);

    // Deposit first so the vault has withdrawable lamports.
    let deposit_amount = 3 * ONE_SOL;

    send(
        &mut svm,
        &user,
        &[build_deposit_ix(&user.pubkey(), deposit_amount)],
        &[],
    )
    .expect("deposit should succeed");

    let (vault, _) = vault_pda(&user.pubkey());

    let vault_before = svm.get_balance(&vault).unwrap_or_default();
    let user_before = svm.get_balance(&user.pubkey()).unwrap_or_default();

    // Withdraw less than the maximum.
    let withdraw_amount = ONE_SOL / 2;

    send(
        &mut svm,
        &user,
        &[build_withdraw_ix(&user.pubkey(), withdraw_amount)],
        &[],
    )
    .expect("withdraw should succeed");

    let vault_after = svm.get_balance(&vault).unwrap_or_default();
    let user_after = svm.get_balance(&user.pubkey()).unwrap_or_default();

    assert_eq!(
        vault_before - vault_after,
        withdraw_amount,
        "vault should shrink by exactly the withdrawn amount"
    );

    assert!(
        user_after > user_before,
        "user balance should increase after withdraw"
    );

    assert!(
        user_after - user_before <= withdraw_amount,
        "user net gain cannot exceed the withdrawn amount (fees)"
    );
}

#[test]
fn withdraw_exactly_at_limit_succeeds() {
    let mut svm = setup_svm();
    let user = Keypair::new();

    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    // Maximum withdrawal: 1 SOL
    initialize_vault(&mut svm, &user, ONE_SOL);

    // Deposit enough funds to allow a 1 SOL withdrawal.
    send(
        &mut svm,
        &user,
        &[build_deposit_ix(&user.pubkey(), 3 * ONE_SOL)],
        &[],
    )
    .expect("deposit should succeed");

    // Exactly at the limit must succeed.
    send(
        &mut svm,
        &user,
        &[build_withdraw_ix(&user.pubkey(), ONE_SOL)],
        &[],
    )
    .expect("withdrawal exactly at max_withdraw should succeed");
}

#[test]
fn withdraw_over_limit_fails() {
    let mut svm = setup_svm();
    let user = Keypair::new();

    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    // Maximum withdrawal: 1 SOL
    initialize_vault(&mut svm, &user, ONE_SOL);

    // Deposit enough funds so the failure is caused by max_withdraw,
    // not by insufficient vault balance.
    send(
        &mut svm,
        &user,
        &[build_deposit_ix(&user.pubkey(), 3 * ONE_SOL)],
        &[],
    )
    .expect("deposit should succeed");

    // 1 SOL + 1 lamport exceeds the limit.
    let res = send(
        &mut svm,
        &user,
        &[build_withdraw_ix(&user.pubkey(), ONE_SOL + 1)],
        &[],
    );

    assert!(res.is_err(), "withdrawal above max_withdraw must fail");
}

#[test]
fn withdraw_more_than_vault_holds_fails() {
    let mut svm = setup_svm();
    let user = Keypair::new();

    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    // Use a high limit here so this test is specifically about
    // insufficient vault balance.
    initialize_vault(&mut svm, &user, 100 * ONE_SOL);

    // The vault only has its initial rent balance.
    let res = send(
        &mut svm,
        &user,
        &[build_withdraw_ix(&user.pubkey(), 100 * ONE_SOL)],
        &[],
    );

    assert!(
        res.is_err(),
        "withdrawing more than the vault holds must fail"
    );
}

#[test]
fn withdraw_without_initialize_fails() {
    let mut svm = setup_svm();
    let user = Keypair::new();

    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    // No initialize_vault call.
    let res = send(
        &mut svm,
        &user,
        &[build_withdraw_ix(&user.pubkey(), ONE_SOL)],
        &[],
    );

    assert!(
        res.is_err(),
        "withdraw without prior initialize must fail because vault_state does not exist"
    );
}

#[test]
fn withdraw_with_wrong_user_fails() {
    let mut svm = setup_svm();

    let owner = Keypair::new();
    let attacker = Keypair::new();

    fund(&mut svm, &owner.pubkey(), 10 * ONE_SOL);
    fund(&mut svm, &attacker.pubkey(), 10 * ONE_SOL);

    initialize_vault(&mut svm, &owner, ONE_SOL);

    send(
        &mut svm,
        &owner,
        &[build_deposit_ix(&owner.pubkey(), 2 * ONE_SOL)],
        &[],
    )
    .expect("owner deposit should succeed");

    // The attacker has no initialized vault_state.
    let res = send(
        &mut svm,
        &attacker,
        &[build_withdraw_ix(&attacker.pubkey(), ONE_SOL)],
        &[],
    );

    assert!(
        res.is_err(),
        "an attacker without an initialized vault must not be able to withdraw"
    );
}
