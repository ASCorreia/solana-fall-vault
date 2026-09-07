use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("You have exceeded the maximum withdraw limit")]
    CustomError,
}
