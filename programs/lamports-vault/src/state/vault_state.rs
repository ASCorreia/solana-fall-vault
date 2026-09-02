use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct VaultState {
    pub vault_bump: u8,     // The bump seed for the vault account PDA
    pub bump: u8,           // The bump seed for the VaultState PDA itself
}