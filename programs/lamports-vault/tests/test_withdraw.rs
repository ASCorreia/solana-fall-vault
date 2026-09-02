mod common;

use {
    crate::common::BASE_FEE, common::{
        ONE_SOL, assert_anchor_error, build_deposit_ix, build_withdraw_ix, fund, initialize_vault,
        send, setup_svm, vault_pda,
    }, lamports_vault::error::ErrorCode, solana_keypair::Keypair, solana_signer::Signer,
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

#[test]
fn withdraw_under_limit() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    fund(&mut svm, &payer.pubkey(), 10 * ONE_SOL);

    initialize_vault(&mut svm, &payer);
    send(
        &mut svm,
        &payer,
        &[build_deposit_ix(&payer.pubkey(), 2 * ONE_SOL)],
        &[],
    )
    .expect("payer deposit should succeed");

    let balance_before: u64 = svm.get_balance(&payer.pubkey()).expect("Expect to have a balance");
    let withdraw_amount = ONE_SOL;

    let ix = build_withdraw_ix(&payer.pubkey(), withdraw_amount);
    let _ = send(&mut svm, &payer, &[ix.clone()], &[]);

    let balance_after: u64 = svm.get_balance(&payer.pubkey()).expect("Expect to have a balance");
    let (payer_vault, _) = vault_pda(&payer.pubkey());
    
    assert!(svm.get_account(&payer_vault).is_some());

    assert_eq!(
        balance_after - balance_before,
        withdraw_amount - BASE_FEE,
        "Payer should get funds back"
    );
    
}

#[test]
fn withdraw_at_limit() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    fund(&mut svm, &payer.pubkey(), 10 * ONE_SOL);

    initialize_vault(&mut svm, &payer);
    send(
        &mut svm,
        &payer,
        &[build_deposit_ix(&payer.pubkey(), 5 * ONE_SOL)],
        &[],
    )
    .expect("payer deposit should succeed");

    let balance_before: u64 = svm.get_balance(&payer.pubkey()).expect("Expect to have a balance");
    let withdraw_amount = 2 * ONE_SOL; // vault max withdraw at 2 SOL in initialize_vault

    let ix = build_withdraw_ix(&payer.pubkey(), withdraw_amount);
    let _ = send(&mut svm, &payer, &[ix.clone()], &[]);

    let balance_after: u64 = svm.get_balance(&payer.pubkey()).expect("Expect to have a balance");
    let (payer_vault, _) = vault_pda(&payer.pubkey());
    
    assert!(svm.get_account(&payer_vault).is_some());

    assert_eq!(
        balance_after - balance_before,
        withdraw_amount - BASE_FEE,
        "Payer should get funds back"
    );
    
}

#[test]
fn withdraw_above_limit() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    fund(&mut svm, &payer.pubkey(), 10 * ONE_SOL);

    initialize_vault(&mut svm, &payer);
    send(
        &mut svm,
        &payer,
        &[build_deposit_ix(&payer.pubkey(), 5 * ONE_SOL)],
        &[],
    )
    .expect("payer deposit should succeed");

    let balance_before: u64 = svm.get_balance(&payer.pubkey()).expect("Expect to have a balance");
    let withdraw_amount = 2 * ONE_SOL + 1; // vault max withdraw at 2 SOL in initialize_vault

    let ix = build_withdraw_ix(&payer.pubkey(), withdraw_amount);
    let e = send(&mut svm, &payer, &[ix.clone()], &[])
        .expect_err("withdraw above the limit must fail");
    assert_anchor_error(&e, ErrorCode::ExceedsMaxWithdraw);

    let balance_after: u64 = svm.get_balance(&payer.pubkey()).expect("Expect to have a balance");
    let (payer_vault, _) = vault_pda(&payer.pubkey());
    
    assert!(svm.get_account(&payer_vault).is_some());

    assert_eq!(balance_before - BASE_FEE, balance_after, "Balances should be the same minus BASE_FEE")
    
}