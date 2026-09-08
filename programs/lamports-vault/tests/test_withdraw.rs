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

    initialize_vault(&mut svm, &user, 100 * ONE_SOL);

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
    // User credit equals the withdrawn amount minus the transaction fee.
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
fn withdraw_more_than_vault_holds_fails() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    initialize_vault(&mut svm, &user, 100 * ONE_SOL);

    // Try to withdraw far more than what the vault was seeded with at init.
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

    initialize_vault(&mut svm, &owner, 100 * ONE_SOL);
    send(
        &mut svm,
        &owner,
        &[build_deposit_ix(&owner.pubkey(), 2 * ONE_SOL)],
        &[],
    )
    .expect("owner deposit should succeed");

    // The attacker tries to withdraw from the owner's vault by signing with
    // their own keypair. Because the vault PDA is derived from the user's key,
    // an attacker-built `withdraw` instruction targets a non-existent PDA and
    // therefore cannot drain the owner's vault.
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

#[test]
fn withdraw_under_the_limit_succeeds() {
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

    let (vault, _) = vault_pda(&user.pubkey());
    let vault_before = svm.get_balance(&vault).unwrap_or_default();

    let amount = max_withdraw - ONE_SOL;
    send(
        &mut svm,
        &user,
        &[build_withdraw_ix(&user.pubkey(), amount)],
        &[],
    )
    .expect("withdraw below max_withdraw should succeed");

    let vault_after = svm.get_balance(&vault).unwrap_or_default();
    assert_eq!(
        vault_before - vault_after,
        amount,
        "vault should shrink by exactly the withdrawn amount"
    );
}

#[test]
fn withdraw_at_exactly_the_limit_succeeds() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    let max_withdraw = ONE_SOL;
    initialize_vault(&mut svm, &user, max_withdraw);

    send(
        &mut svm,
        &user,
        &[build_deposit_ix(&user.pubkey(), 3 * ONE_SOL)],
        &[],
    )
    .expect("deposit should succeed");

    let (vault, _) = vault_pda(&user.pubkey());
    let vault_before = svm.get_balance(&vault).unwrap_or_default();

    send(
        &mut svm,
        &user,
        &[build_withdraw_ix(&user.pubkey(), max_withdraw)],
        &[],
    )
    .expect("withdraw of exactly max_withdraw should succeed");

    let vault_after = svm.get_balance(&vault).unwrap_or_default();
    assert_eq!(
        vault_before - vault_after,
        max_withdraw,
        "vault should shrink by exactly the withdrawn amount"
    );
}

#[test]
fn withdraw_above_the_limit_fails() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    let max_withdraw = ONE_SOL;
    initialize_vault(&mut svm, &user, max_withdraw);

    send(
        &mut svm,
        &user,
        &[build_deposit_ix(&user.pubkey(), 3 * ONE_SOL)],
        &[],
    )
    .expect("deposit should succeed");

    let (vault, _) = vault_pda(&user.pubkey());
    let vault_before = svm.get_balance(&vault).unwrap_or_default();

    let failure = send(
        &mut svm,
        &user,
        &[build_withdraw_ix(&user.pubkey(), max_withdraw + 1)],
        &[],
    )
    .expect_err("withdraw above max_withdraw must fail");

    assert!(
        failure
            .meta
            .logs
            .iter()
            .any(|line| line.contains("WithdrawLimitExceeded")),
        "failure must come from the withdraw limit, logs: {:?}",
        failure.meta.logs
    );

    let vault_after = svm.get_balance(&vault).unwrap_or_default();
    assert_eq!(
        vault_after, vault_before,
        "a rejected withdraw must leave the vault balance unchanged"
    );
}

#[test]
fn limit_applies_per_transaction_not_cumulatively() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    let max_withdraw = ONE_SOL;
    initialize_vault(&mut svm, &user, max_withdraw);

    send(
        &mut svm,
        &user,
        &[build_deposit_ix(&user.pubkey(), 3 * ONE_SOL)],
        &[],
    )
    .expect("deposit should succeed");

    let (vault, _) = vault_pda(&user.pubkey());
    let vault_before = svm.get_balance(&vault).unwrap_or_default();

    for _ in 0..3 {
        // Each send repeats the same instruction, so the blockhash must change.
        // Otherwise the transaction is a byte-for-byte duplicate and is rejected
        // as AlreadyProcessed before it reaches the program.
        svm.expire_blockhash();
        send(
            &mut svm,
            &user,
            &[build_withdraw_ix(&user.pubkey(), max_withdraw)],
            &[],
        )
        .expect("each withdraw at the limit should succeed");
    }

    let vault_after = svm.get_balance(&vault).unwrap_or_default();
    assert_eq!(
        vault_before - vault_after,
        3 * max_withdraw,
        "three withdrawals at the limit should all go through"
    );
}
