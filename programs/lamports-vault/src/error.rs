use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
        #[msg("Amount exceeds the per-transaction withdrawal limit")]
    ExceedsMaxWithdraw,
}
