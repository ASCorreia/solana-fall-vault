mod common;

use {
    common::{
        build_deposit_ix, build_initialize_ix, build_withdraw_ix, fund, initialize_vault, send,
        setup_svm, vault_pda, ONE_SOL,
    },
    solana_keypair::Keypair,
    solana_signer::Signer,
};

/// Per-transaction ceiling used across these tests.
const LIMIT: u64 = 2 * ONE_SOL;

/// Set up a funded, initialized vault with `LIMIT` as its withdrawal ceiling
/// and enough deposited lamports that a withdrawal can only ever fail because
/// of the limit, never for lack of funds.
fn vault_with_limit() -> (litesvm::LiteSVM, Keypair) {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 20 * ONE_SOL);

    initialize_vault(&mut svm, &user, LIMIT);

    send(
        &mut svm,
        &user,
        &[build_deposit_ix(&user.pubkey(), 10 * ONE_SOL)],
        &[],
    )
    .expect("deposit should succeed");

    (svm, user)
}

#[test]
fn withdraw_under_the_limit_succeeds() {
    let (mut svm, user) = vault_with_limit();
    let (vault, _) = vault_pda(&user.pubkey());

    let before = svm.get_balance(&vault).unwrap_or_default();
    let amount = LIMIT - 1;

    send(
        &mut svm,
        &user,
        &[build_withdraw_ix(&user.pubkey(), amount)],
        &[],
    )
    .expect("a withdrawal below the limit must succeed");

    let after = svm.get_balance(&vault).unwrap_or_default();
    assert_eq!(
        before - after,
        amount,
        "vault should shrink by exactly the withdrawn amount"
    );
}

#[test]
fn withdraw_exactly_at_the_limit_succeeds() {
    let (mut svm, user) = vault_with_limit();
    let (vault, _) = vault_pda(&user.pubkey());

    let before = svm.get_balance(&vault).unwrap_or_default();

    // The check is `amount <= max_withdraw`, so the boundary itself is allowed.
    send(
        &mut svm,
        &user,
        &[build_withdraw_ix(&user.pubkey(), LIMIT)],
        &[],
    )
    .expect("a withdrawal exactly at the limit must succeed");

    let after = svm.get_balance(&vault).unwrap_or_default();
    assert_eq!(before - after, LIMIT, "vault should shrink by exactly the limit");
}

#[test]
fn withdraw_over_the_limit_fails() {
    let (mut svm, user) = vault_with_limit();
    let (vault, _) = vault_pda(&user.pubkey());

    let before = svm.get_balance(&vault).unwrap_or_default();

    // One lamport over the ceiling. The vault holds far more than this, so the
    // only possible reason to fail is the limit itself.
    let err = send(
        &mut svm,
        &user,
        &[build_withdraw_ix(&user.pubkey(), LIMIT + 1)],
        &[],
    )
    .expect_err("a withdrawal above the limit must be rejected");

    // Assert on the exact error, not merely that something failed - otherwise
    // an unrelated failure would make this test pass for the wrong reason.
    // ExceedsMaxWithdraw is the second variant in ErrorCode, so 6000 + 1.
    let detail = format!("{:?}", err.err);
    assert!(
        detail.contains("Custom(6001)"),
        "expected ExceedsMaxWithdraw (custom error 6001), got {detail}"
    );
    assert!(
        err.meta
            .logs
            .iter()
            .any(|l| l.contains("ExceedsMaxWithdraw")),
        "program logs should name the constraint that rejected the withdrawal"
    );

    let after = svm.get_balance(&vault).unwrap_or_default();
    assert_eq!(
        before, after,
        "a rejected withdrawal must leave the vault balance untouched"
    );
}

#[test]
fn limit_is_persisted_on_vault_state() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    initialize_vault(&mut svm, &user, LIMIT);

    let (vault_state, _) = common::vault_state_pda(&user.pubkey());
    let data = svm.get_account(&vault_state).unwrap().data;

    // Layout: [0..8] discriminator, [8] vault_bump, [9] bump, [10..18] max_withdraw.
    // The new field sits after the bumps so existing byte-offset assertions hold.
    let stored = u64::from_le_bytes(data[10..18].try_into().unwrap());
    assert_eq!(stored, LIMIT, "max_withdraw should be stored little-endian at byte 10");
}

#[test]
fn each_vault_carries_its_own_limit() {
    let mut svm = setup_svm();
    let alice = Keypair::new();
    let bob = Keypair::new();
    fund(&mut svm, &alice.pubkey(), 20 * ONE_SOL);
    fund(&mut svm, &bob.pubkey(), 20 * ONE_SOL);

    initialize_vault(&mut svm, &alice, ONE_SOL);
    initialize_vault(&mut svm, &bob, 5 * ONE_SOL);

    for (who, amount) in [(&alice, 5 * ONE_SOL), (&bob, 5 * ONE_SOL)] {
        send(
            &mut svm,
            who,
            &[build_deposit_ix(&who.pubkey(), 8 * ONE_SOL)],
            &[],
        )
        .expect("deposit should succeed");

        let res = send(
            &mut svm,
            who,
            &[build_withdraw_ix(&who.pubkey(), amount)],
            &[],
        );

        if who.pubkey() == alice.pubkey() {
            assert!(res.is_err(), "alice's 1 SOL limit must reject a 5 SOL withdrawal");
        } else {
            assert!(res.is_ok(), "bob's 5 SOL limit must allow a 5 SOL withdrawal");
        }
    }

    // Sanity: the instruction builder itself is limit-agnostic.
    let _ = build_initialize_ix(&alice.pubkey(), ONE_SOL);
}
