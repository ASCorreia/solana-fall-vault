use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
    #[msg("This withdrawal is more red flag than your ex")]
    TooMuchDrama,

}
