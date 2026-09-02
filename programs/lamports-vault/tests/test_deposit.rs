mod common;

use {
    common::{
        build_deposit_ix, fund, initialize_vault, send, setup_svm, vault_pda, ONE_SOL,
    },
    solana_keypair::Keypair,
    solana_signer::Signer,
};

#[test]
fn deposit_increases_vault_balance() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    initialize_vault(&mut svm, &user);

    let (vault, _) = vault_pda(&user.pubkey());
    let vault_before = svm.get_balance(&vault).unwrap_or_default();
    let user_before = svm.get_balance(&user.pubkey()).unwrap_or_default();

    let amount = 2 * ONE_SOL;
    let ix = build_deposit_ix(&user.pubkey(), amount);
    send(&mut svm, &user, &[ix], &[]).expect("deposit should succeed");

    let vault_after = svm.get_balance(&vault).unwrap_or_default();
    let user_after = svm.get_balance(&user.pubkey()).unwrap_or_default();

    assert_eq!(
        vault_after - vault_before,
        amount,
        "vault should grow by exactly the deposited amount"
    );
    // User should be debited at least the amount; the difference also includes tx fees.
    assert!(
        user_before - user_after >= amount,
        "user should be debited at least the deposit amount"
    );
}

#[test]
fn multiple_deposits_accumulate() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    initialize_vault(&mut svm, &user);

    let (vault, _) = vault_pda(&user.pubkey());
    let vault_before = svm.get_balance(&vault).unwrap_or_default();

    let deposits = [ONE_SOL / 2, ONE_SOL, ONE_SOL / 4];
    let mut total = 0u64;
    for amount in deposits {
        let ix = build_deposit_ix(&user.pubkey(), amount);
        send(&mut svm, &user, &[ix], &[]).expect("deposit should succeed");
        total += amount;
    }

    let vault_after = svm.get_balance(&vault).unwrap_or_default();
    assert_eq!(
        vault_after - vault_before,
        total,
        "vault should accumulate all deposits"
    );
}

#[test]
fn deposit_without_initialize_fails() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    // No initialize_vault call: vault_state PDA does not exist yet.
    let ix = build_deposit_ix(&user.pubkey(), ONE_SOL);
    let res = send(&mut svm, &user, &[ix], &[]);
    assert!(
        res.is_err(),
        "deposit without prior initialize must fail because vault_state does not exist"
    );
}

#[test]
fn deposit_more_than_balance_fails() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 2 * ONE_SOL);

    initialize_vault(&mut svm, &user);

    // Try to deposit way more than the user has.
    let ix = build_deposit_ix(&user.pubkey(), 100 * ONE_SOL);
    let res = send(&mut svm, &user, &[ix], &[]);
    assert!(
        res.is_err(),
        "deposit larger than the user's balance must fail"
    );
}

#[test]
fn deposit_zero_lamports_succeeds_and_is_a_noop() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    initialize_vault(&mut svm, &user);

    let (vault, _) = vault_pda(&user.pubkey());
    let vault_before = svm.get_balance(&vault).unwrap_or_default();

    let ix = build_deposit_ix(&user.pubkey(), 0);
    send(&mut svm, &user, &[ix], &[]).expect("zero-lamport deposit should succeed");

    let vault_after = svm.get_balance(&vault).unwrap_or_default();
    assert_eq!(vault_after, vault_before, "vault balance should be unchanged");
}
