mod common;

use {
    common::{
        build_close_ix, build_deposit_ix, build_withdraw_ix, fund, initialize_vault, send,
        setup_svm, vault_pda, vault_state_pda, ONE_SOL,
    },
    solana_keypair::Keypair,
    solana_signer::Signer,
};

#[test]
fn close_returns_all_lamports_and_deletes_vault_state() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    initialize_vault(&mut svm, &user);

    let deposit_amount = 3 * ONE_SOL;
    send(
        &mut svm,
        &user,
        &[build_deposit_ix(&user.pubkey(), deposit_amount)],
        &[],
    )
    .expect("deposit should succeed");

    let (vault, _) = vault_pda(&user.pubkey());
    let (vault_state, _) = vault_state_pda(&user.pubkey());

    let vault_before = svm.get_balance(&vault).unwrap_or_default();
    let state_rent = svm
        .get_account(&vault_state)
        .expect("vault_state should exist before close")
        .lamports;
    let user_before = svm.get_balance(&user.pubkey()).unwrap_or_default();

    send(&mut svm, &user, &[build_close_ix(&user.pubkey())], &[])
        .expect("close should succeed");

    // The vault is fully drained and the vault_state account is gone.
    assert_eq!(
        svm.get_balance(&vault).unwrap_or_default(),
        0,
        "vault should be emptied by close"
    );
    let state_after = svm.get_account(&vault_state);
    assert!(
        state_after.is_none() || state_after.as_ref().unwrap().lamports == 0,
        "vault_state should be closed"
    );

    // The user gets back the vault balance plus the vault_state rent, minus fees.
    let user_after = svm.get_balance(&user.pubkey()).unwrap_or_default();
    let returned = user_after - user_before;
    assert!(
        returned <= vault_before + state_rent,
        "user cannot receive more than the vault balance plus the state rent"
    );
    assert!(
        returned > deposit_amount,
        "user should get back at least the deposited amount"
    );
}

#[test]
fn close_without_initialize_fails() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    let res = send(&mut svm, &user, &[build_close_ix(&user.pubkey())], &[]);
    assert!(
        res.is_err(),
        "close without prior initialize must fail because vault_state does not exist"
    );
}

#[test]
fn close_twice_fails() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    initialize_vault(&mut svm, &user);

    send(&mut svm, &user, &[build_close_ix(&user.pubkey())], &[])
        .expect("first close should succeed");

    let res = send(&mut svm, &user, &[build_close_ix(&user.pubkey())], &[]);
    assert!(res.is_err(), "closing an already closed vault must fail");
}

#[test]
fn close_by_non_owner_fails() {
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

    // The attacker has no vault of their own, so the PDAs their instruction
    // targets do not exist and the close must fail. The owner's vault is
    // untouched.
    let res = send(&mut svm, &attacker, &[build_close_ix(&attacker.pubkey())], &[]);
    assert!(res.is_err(), "an attacker must not be able to close a vault");

    let (owner_vault, _) = vault_pda(&owner.pubkey());
    assert!(
        svm.get_balance(&owner_vault).unwrap_or_default() > 0,
        "owner's vault must be untouched"
    );
}

#[test]
fn operations_after_close_fail() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    fund(&mut svm, &user.pubkey(), 10 * ONE_SOL);

    initialize_vault(&mut svm, &user);
    send(&mut svm, &user, &[build_close_ix(&user.pubkey())], &[])
        .expect("close should succeed");

    // vault_state is gone, so both deposit and withdraw must now fail.
    let deposit_res = send(
        &mut svm,
        &user,
        &[build_deposit_ix(&user.pubkey(), ONE_SOL)],
        &[],
    );
    assert!(deposit_res.is_err(), "deposit after close must fail");

    let withdraw_res = send(
        &mut svm,
        &user,
        &[build_withdraw_ix(&user.pubkey(), ONE_SOL)],
        &[],
    );
    assert!(withdraw_res.is_err(), "withdraw after close must fail");
}
