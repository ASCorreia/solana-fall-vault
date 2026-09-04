mod common;

use {
    common::{
        build_deposit_ix, build_initialize_ix, build_withdraw_ix, fund, send, setup_svm,
        vault_pda, ONE_SOL,
    },
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const MAX_WITHDRAW: u64 = ONE_SOL / 2; // 0.5 SOL limit

fn setup_user_with_limit(svm: &mut litesvm::LiteSVM, user: &Keypair, deposit_amount: u64) {
    fund(svm, &user.pubkey(), 10 * ONE_SOL);
    let ix = build_initialize_ix(&user.pubkey(), MAX_WITHDRAW);
    send(svm, user, &[ix], &[]).expect("initialize should succeed");
    if deposit_amount > 0 {
        let dep_ix = build_deposit_ix(&user.pubkey(), deposit_amount);
        send(svm, user, &[dep_ix], &[]).expect("deposit should succeed");
    }
}

#[test]
fn withdraw_within_limit_succeeds() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    setup_user_with_limit(&mut svm, &user, 2 * ONE_SOL);

    let vault = vault_pda(&user.pubkey()).0;
    let vault_before = svm.get_balance(&vault).unwrap();

    let ix = build_withdraw_ix(&user.pubkey(), MAX_WITHDRAW);
    send(&mut svm, &user, &[ix], &[]).expect("withdraw within limit should succeed");

    let vault_after = svm.get_balance(&vault).unwrap();
    assert_eq!(vault_before - vault_after, MAX_WITHDRAW);
}

#[test]
fn withdraw_exactly_at_limit_succeeds() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    setup_user_with_limit(&mut svm, &user, 2 * ONE_SOL);

    let vault = vault_pda(&user.pubkey()).0;
    let vault_before = svm.get_balance(&vault).unwrap();

    let ix = build_withdraw_ix(&user.pubkey(), MAX_WITHDRAW);
    send(&mut svm, &user, &[ix], &[]).expect("withdraw at exact limit should succeed");

    let vault_after = svm.get_balance(&vault).unwrap();
    assert_eq!(vault_before - vault_after, MAX_WITHDRAW);
}

#[test]
fn withdraw_over_limit_fails() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    setup_user_with_limit(&mut svm, &user, 2 * ONE_SOL);

    let over_limit = MAX_WITHDRAW + 1;
    let ix = build_withdraw_ix(&user.pubkey(), over_limit);
    let res = send(&mut svm, &user, &[ix], &[]);
    assert!(res.is_err(), "withdraw exceeding limit must fail");
}
