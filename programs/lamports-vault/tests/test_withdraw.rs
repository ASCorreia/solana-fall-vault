mod common;

use {
    common::{
        build_deposit_ix, build_withdraw_ix, fund, initialize_vault, send, setup_svm, vault_pda,
        DEFAULT_MAX_WITHDRAW, ONE_SOL,
    },
    lamports_vault::error::ErrorCode,
    solana_keypair::Keypair,
    solana_signer::Signer,
    solana_transaction::{InstructionError, TransactionError},
};

/// Per-transaction ceiling used by the withdrawal-limit tests below.
const MAX_WITHDRAW: u64 = 2 * ONE_SOL;
/// Deposited by those tests so the vault always holds far more than the cap;
/// any rejection is therefore the limit, never insufficient funds.
const LIMIT_TEST_DEPOSIT: u64 = 5 * ONE_SOL;

#[test]
fn withdraw_returns_lamports_to_user() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    initialize_vault(&mut svm, &user, DEFAULT_MAX_WITHDRAW);

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

    initialize_vault(&mut svm, &user, DEFAULT_MAX_WITHDRAW);

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

    initialize_vault(&mut svm, &owner, DEFAULT_MAX_WITHDRAW);
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
fn withdraw_below_max_withdraw_succeeds() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    initialize_vault(&mut svm, &user, MAX_WITHDRAW);
    send(
        &mut svm,
        &user,
        &[build_deposit_ix(&user.pubkey(), LIMIT_TEST_DEPOSIT)],
        &[],
    )
    .expect("deposit should succeed");

    let (vault, _) = vault_pda(&user.pubkey());
    let vault_before = svm.get_balance(&vault).unwrap_or_default();

    let amount = MAX_WITHDRAW - ONE_SOL;
    send(
        &mut svm,
        &user,
        &[build_withdraw_ix(&user.pubkey(), amount)],
        &[],
    )
    .expect("withdrawing below the limit should succeed");

    let vault_after = svm.get_balance(&vault).unwrap_or_default();
    assert_eq!(
        vault_before - vault_after,
        amount,
        "vault should shrink by exactly the withdrawn amount"
    );
}

#[test]
fn withdraw_exactly_max_withdraw_succeeds() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    initialize_vault(&mut svm, &user, MAX_WITHDRAW);
    send(
        &mut svm,
        &user,
        &[build_deposit_ix(&user.pubkey(), LIMIT_TEST_DEPOSIT)],
        &[],
    )
    .expect("deposit should succeed");

    let (vault, _) = vault_pda(&user.pubkey());
    let vault_before = svm.get_balance(&vault).unwrap_or_default();

    // The check is `amount <= max_withdraw`, so the limit itself is allowed.
    send(
        &mut svm,
        &user,
        &[build_withdraw_ix(&user.pubkey(), MAX_WITHDRAW)],
        &[],
    )
    .expect("withdrawing exactly the limit should succeed");

    let vault_after = svm.get_balance(&vault).unwrap_or_default();
    assert_eq!(
        vault_before - vault_after,
        MAX_WITHDRAW,
        "vault should shrink by exactly the limit"
    );
}

#[test]
fn withdraw_one_lamport_above_max_withdraw_fails() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    initialize_vault(&mut svm, &user, MAX_WITHDRAW);
    send(
        &mut svm,
        &user,
        &[build_deposit_ix(&user.pubkey(), LIMIT_TEST_DEPOSIT)],
        &[],
    )
    .expect("deposit should succeed");

    let (vault, _) = vault_pda(&user.pubkey());
    let vault_before = svm.get_balance(&vault).unwrap_or_default();

    let err = send(
        &mut svm,
        &user,
        &[build_withdraw_ix(&user.pubkey(), MAX_WITHDRAW + 1)],
        &[],
    )
    .expect_err("withdrawing one lamport above the limit must fail");

    // Assert on the exact error rather than just `is_err()`: the vault holds
    // LIMIT_TEST_DEPOSIT, so a bare failure check would also pass if the
    // rejection came from insufficient funds instead of the limit.
    assert_eq!(
        err.err,
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(u32::from(ErrorCode::ExceedsMaxWithdraw))
        ),
        "withdraw should be rejected by the limit check, not by anything else"
    );

    let vault_after = svm.get_balance(&vault).unwrap_or_default();
    assert_eq!(
        vault_after, vault_before,
        "a rejected withdraw must not move any lamports"
    );
}
