mod common;

use {
    common::{
        build_deposit_ix, build_initialize_ix, build_withdraw_ix, fund, initialize_vault, send,
        setup_svm, vault_pda, ONE_SOL,
    },
    solana_keypair::Keypair,
    solana_signer::Signer,
};

#[test]
fn withdraw_returns_lamports_to_user() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    initialize_vault(&mut svm, &user);

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

    initialize_vault(&mut svm, &user);

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

    initialize_vault(&mut svm, &owner);
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

// Sets up a vault whose per-transaction cap is `cap` and funds it well past
// that cap, so the limit is the only thing a withdraw can trip over.
fn setup_capped_vault(cap: u64) -> (litesvm::LiteSVM, Keypair) {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    send(
        &mut svm,
        &user,
        &[build_initialize_ix(&user.pubkey(), cap)],
        &[],
    )
    .expect("initialize should succeed");
    send(
        &mut svm,
        &user,
        &[build_deposit_ix(&user.pubkey(), 5 * ONE_SOL)],
        &[],
    )
    .expect("deposit should succeed");

    (svm, user)
}

#[test]
fn withdraw_below_the_cap_succeeds() {
    let (mut svm, user) = setup_capped_vault(ONE_SOL);

    send(
        &mut svm,
        &user,
        &[build_withdraw_ix(&user.pubkey(), ONE_SOL / 4)],
        &[],
    )
    .expect("withdrawing less than the cap should succeed");
}

#[test]
fn withdraw_exactly_the_cap_succeeds() {
    let (mut svm, user) = setup_capped_vault(ONE_SOL);

    // The check is inclusive, so hitting the cap exactly is still allowed.
    send(
        &mut svm,
        &user,
        &[build_withdraw_ix(&user.pubkey(), ONE_SOL)],
        &[],
    )
    .expect("withdrawing exactly the cap should succeed");
}

#[test]
fn withdraw_above_the_cap_fails() {
    let (mut svm, user) = setup_capped_vault(ONE_SOL);
    let (vault, _) = vault_pda(&user.pubkey());
    let before = svm.get_balance(&vault).unwrap_or_default();

    let err = send(
        &mut svm,
        &user,
        &[build_withdraw_ix(&user.pubkey(), ONE_SOL + 1)],
        &[],
    )
    .expect_err("withdrawing over the cap must fail");

    let msg = format!("{err:?}");
    assert!(
        msg.contains("MaxWithdrawExceeded") || msg.contains("over the per-transaction"),
        "expected MaxWithdrawExceeded, got {msg}"
    );
    assert_eq!(
        svm.get_balance(&vault).unwrap_or_default(),
        before,
        "a rejected withdraw must leave the vault untouched"
    );
}

#[test]
fn cap_applies_per_transaction_not_in_total() {
    let (mut svm, user) = setup_capped_vault(ONE_SOL);

    // Two withdraws of the full cap are fine: nothing accumulates across
    // instructions.
    for _ in 0..2 {
        send(
            &mut svm,
            &user,
            &[build_withdraw_ix(&user.pubkey(), ONE_SOL)],
            &[],
        )
        .expect("each withdraw within the cap should succeed");
    }
}

#[test]
fn zero_cap_blocks_every_withdraw() {
    let (mut svm, user) = setup_capped_vault(0);

    let res = send(
        &mut svm,
        &user,
        &[build_withdraw_ix(&user.pubkey(), 1)],
        &[],
    );
    assert!(
        res.is_err(),
        "a vault initialized with max_withdraw = 0 must reject any withdraw"
    );
}
