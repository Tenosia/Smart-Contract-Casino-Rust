use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum EscrowError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,
    /// Not Rent Exempt
    #[error("Not Rent Exempt")]
    NotRentExempt,
    /// Expected Amount Mismatch
    #[error("Expected Amount Mismatch")]
    ExpectedAmountMismatch,
    /// Amount Overflow
    #[error("Amount Overflow")]
    AmountOverflow,
    /// Invalid Account
    #[error("Invalid Account")]
    InvalidAccount,
    /// Invalid Amount
    #[error("Invalid Amount")]
    InvalidAmount,
    /// Invalid Owner
    #[error("Invalid Owner")]
    InvalidOwner,
    #[error("Invalid PDA Seeds")]
    InvalidPdaSeeds,
    /// Escrow Not Initialized
    #[error("Escrow Not Initialized")]
    EscrowNotInitialized,
    /// Invalid Result Value
    #[error("Invalid Result Value")]
    InvalidResultValue,
    /// Minimum Amount Not Met
    #[error("Minimum Amount Not Met")]
    MinimumAmountNotMet,
    /// Both Players Must Deposit
    #[error("Both Players Must Deposit")]
    BothPlayersMustDeposit,
    /// Invalid Account Owner
    #[error("Invalid Account Owner")]
    InvalidAccountOwner,
    /// Invalid System Program
    #[error("Invalid System Program")]
    InvalidSystemProgram,
    /// Invalid PDA Account
    #[error("Invalid PDA Account")]
    InvalidPdaAccount,

}

impl From<EscrowError> for ProgramError {
    fn from(e: EscrowError) -> Self {
        ProgramError::Custom(e as u32)
    }
}