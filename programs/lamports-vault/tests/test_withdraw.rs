mod common;

use {
    common::{
        build_deposit_ix, build_withdraw_ix, fund, initialize_vault, send, setup_svm, vault_pda,
        ONE_SOL,
    },
    solana_keypair::Keypair,
    solana_signer::Signer,
};

// A valid withdrawal - under the cap, suceeds.
#[test]
fn valid_withdrawal() {
    let mut svm = setup_svm();
    let user = Keypair::new();

    // user wallet address funded with 10000 * 1e9 lamports
    fund(&mut svm, &user.pubkey(), 10000 * ONE_SOL);

    /* Vault init config:
    payer: Vault_initializer
    max_withdraw: 100 * ONE_SOL
     */

    initialize_vault(&mut svm, &user, 100 * ONE_SOL);

    let deposit_amount = 50 * ONE_SOL;

    /* User Deposit config:
       payer: user,
       deposit_amount: 50 * ONE_SOL
    */

    send(
        &mut svm,
        &user,
        &[build_deposit_ix(&user.pubkey(), deposit_amount)],
        &[],
    )
    .expect("deposit should succeed");

    let withdraw_amount = 30 * ONE_SOL;

    /* User Withdraw config:
       payer: user,
       withdraw_amount: 30 * ONE_SOL
    */

    // (PARTIAL WITHDRAWAL)
    send(
        &mut svm,
        &user,
        &[build_withdraw_ix(&user.pubkey(), withdraw_amount)],
        &[],
    )
    .expect("withdraw should succeed");

    let max_withdraw = 100 * ONE_SOL;

    assert!(
        withdraw_amount < max_withdraw,
        "Withdraw amount should be under the cap"
    );
}

// Exactly at the limit -- amount == max_withdraw, succeeds
#[test]
fn withdraw_exactly_at_the_cap() {
    let mut svm = setup_svm();
    let user = Keypair::new();

    // user wallet address funded with 10000 * 1e9 lamports
    fund(&mut svm, &user.pubkey(), 10000 * ONE_SOL);

    /* Vault init config:
    payer: Vault_initializer
    max_withdraw: 100 * ONE_SOL
     */

    initialize_vault(&mut svm, &user, 100 * ONE_SOL);

    let deposit_amount = 120 * ONE_SOL;

    /* User Deposit config:
       payer: user,
       deposit_amount: 120 * ONE_SOL
    */

    send(
        &mut svm,
        &user,
        &[build_deposit_ix(&user.pubkey(), deposit_amount)],
        &[],
    )
    .expect("deposit should succeed");

    let withdraw_amount = 100 * ONE_SOL;

    /* User Withdraw config:
       payer: user,
       withdraw_amount: 100 * ONE_SOL
    */

    // (PARTIAL WITHDRAWAL and exactly at the cap)
    send(
        &mut svm,
        &user,
        &[build_withdraw_ix(&user.pubkey(), withdraw_amount)],
        &[],
    )
    .expect("withdraw should succeed");

    let max_withdraw = 100 * ONE_SOL;

    assert!(
        withdraw_amount <= max_withdraw,
        "Withdraw amount should be under or Equal the cap"
    );
}

// Over the limit -- one lamport more, fails
#[test]
fn withdraw_over_the_cap() {
    let mut svm = setup_svm();
    let user = Keypair::new();

    // user wallet address funded with 10000 * 1e9 lamports
    fund(&mut svm, &user.pubkey(), 10000 * ONE_SOL);

    /* Vault init config:
    payer: Vault_initializer
    max_withdraw: 100 * ONE_SOL
     */

    initialize_vault(&mut svm, &user, 100 * ONE_SOL);

    let deposit_amount = 120 * ONE_SOL;

    /* User Deposit config:
       payer: user,
       deposit_amount: 120 * ONE_SOL
    */

    send(
        &mut svm,
        &user,
        &[build_deposit_ix(&user.pubkey(), deposit_amount)],
        &[],
    )
    .expect("deposit should succeed");

    let withdraw_amount = (100 * ONE_SOL) + 1;

    /* User Withdraw config:
       payer: user,
       withdraw_amount: (100 * ONE_SOL) + 1
    */

    // (PARTIAL WITHDRAWAL and exactly 1 unit lamport over the cap)
    let res = send(
        &mut svm,
        &user,
        &[build_withdraw_ix(&user.pubkey(), withdraw_amount)],
        &[],
    );

    let max_withdraw = 100 * ONE_SOL;

    assert!(
        withdraw_amount > max_withdraw,
        "Withdraw amount should be over the cap"
    );
    assert!(res.is_err(), "withdraw over the cap should fail");
}

#[test]
fn withdraw_returns_lamports_to_user() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    initialize_vault(
        &mut svm,
        &user,
        100u64
            .checked_mul(ONE_SOL)
            .expect("vault seed amount should not overflow"),
    );

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

    let vault_after: u64 = svm.get_balance(&vault).unwrap_or_default();
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
