use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum SwapError {
    #[error("Account is not writable")]
    AccountNotWritable,

    #[error("Invalid ata creator offered mint")]
    InvalidAtaCreatorOfferedMint,

    #[error("Invalid ata creator offered owner")]
    InvalidAtaCreatorOfferedOwner,

    #[error("Invalid mint account data")]
    InvalidMintAccountData,

    #[error("Invalid token account data")]
    InvalidTokenAccountData,

    #[error("Invalid swap account data")]
    InvalidSwapAccountData,
}

impl From<SwapError> for ProgramError {
    fn from(e: SwapError) -> Self {
        ProgramError::Custom(e as u32)
    }
}