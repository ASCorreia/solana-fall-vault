mod common;

use {
    common::{
        build_close_ix, build_deposit_ix, fund, initialize_vault, send, setup_svm, vault_pda,
        vault_state_pda, ONE_SOL,
    },
    solana_keypair::Keypair,
    solana_signer::Signer,
};

#[test]
fn close_returns_vault_funds_and_removes_accounts() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);
    initialize_vault(&mut svm, &user);
    send(
        &mut svm,
        &user,
        &[build_deposit_ix(&user.pubkey(), 2 * ONE_SOL)],
        &[],
    )
    .expect("deposit should succeed");

    let user_before = svm.get_balance(&user.pubkey()).unwrap();
    let (vault, _) = vault_pda(&user.pubkey());
    let (vault_state, _) = vault_state_pda(&user.pubkey());
    let vault_balance = svm.get_balance(&vault).unwrap();

    send(&mut svm, &user, &[build_close_ix(&user.pubkey())], &[])
        .expect("close should succeed");

    let user_after = svm.get_balance(&user.pubkey()).unwrap();
    assert!(user_after > user_before, "user should receive vault lamports");
    assert!(
        user_after - user_before >= vault_balance,
        "user should receive the vault balance"
    );
    assert_eq!(svm.get_balance(&vault).unwrap_or_default(), 0);
    assert!(svm.get_account(&vault_state).is_none());
}

#[test]
fn close_without_initialize_fails() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 2 * ONE_SOL);

    let result = send(&mut svm, &user, &[build_close_ix(&user.pubkey())], &[]);
    assert!(result.is_err(), "an uninitialized vault cannot be closed");
}