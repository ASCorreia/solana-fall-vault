use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct VaultState {
    pub vault_bump: u8,     // The bump seed for the vault account PDA
    pub bump: u8,           // The bump seed for the VaultState PDA itself
    pub max_withdraw: u64,  // Per-transaction withdrawal ceiling. Must stay last:
                            // tests read the bumps at raw byte offsets 8 and 9.
}
