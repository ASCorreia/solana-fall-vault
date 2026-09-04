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

    initialize_vault(&mut svm, &user, 10 * ONE_SOL);

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

    let withdraw_amount = ONE_SOL;
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
fn withdraw_under_max_withdraw_succeeds() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    let max_withdraw = 2 * ONE_SOL;
    initialize_vault(&mut svm, &user, max_withdraw);

    send(
        &mut svm,
        &user,
        &[build_deposit_ix(&user.pubkey(), 3 * ONE_SOL)],
        &[],
    )
    .expect("deposit should succeed");

    send(
        &mut svm,
        &user,
        &[build_withdraw_ix(&user.pubkey(), ONE_SOL)],
        &[],
    )
    .expect("withdraw under the maximum should succeed");
}

#[test]
fn withdraw_at_max_withdraw_succeeds() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    let max_withdraw = 2 * ONE_SOL;
    initialize_vault(&mut svm, &user, max_withdraw);

    send(
        &mut svm,
        &user,
        &[build_deposit_ix(&user.pubkey(), 3 * ONE_SOL)],
        &[],
    )
    .expect("deposit should succeed");

    send(
        &mut svm,
        &user,
        &[build_withdraw_ix(&user.pubkey(), max_withdraw)],
        &[],
    )
    .expect("withdraw at the maximum should succeed");
}

#[test]
fn withdraw_one_lamport_over_max_withdraw_fails() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    let max_withdraw = 2 * ONE_SOL;
    initialize_vault(&mut svm, &user, max_withdraw);

    send(
        &mut svm,
        &user,
        &[build_deposit_ix(&user.pubkey(), 3 * ONE_SOL)],
        &[],
    )
    .expect("deposit should succeed");

    let res = send(
        &mut svm,
        &user,
        &[build_withdraw_ix(&user.pubkey(), max_withdraw + 1)],
        &[],
    );
    let failure = res.expect_err("withdraw one lamport over the maximum must fail");
    assert!(
        failure
            .meta
            .logs
            .iter()
            .any(|log| log.contains("Withdrawal amount exceeds the maximum allowed limit.")),
        "failure logs should identify the withdrawal limit error: {:?}",
        failure.meta.logs
    );
}

#[test]
fn withdraw_more_than_vault_holds_fails() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    initialize_vault(&mut svm, &user, 10 * ONE_SOL);

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

    initialize_vault(&mut svm, &owner, 10 * ONE_SOL);
    send(
        &mut svm,
        &owner,
        &[build_deposit_ix(&owner.pubkey(), 2 * ONE_SOL)],
        &[],
    )
    .expect("owner deposit should succeed");

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
