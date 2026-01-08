use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

use crate::{error::EscrowError, instruction::EscrowInstruction, state::Escrow};

const MINIMUM_DEPOSIT: u64 = 1000; // Minimum deposit amount in lamports
const FEE_PERCENTAGE: u64 = 19; // 1.9% = 19/1000
const ADMIN_PERCENTAGE: u64 = 1; // 0.1% = 1/1000
const ESCROW_PERCENTAGE: u64 = 980; // 98% = 980/1000

pub struct Processor;

impl Processor {
    pub fn process(
        accounts: &[AccountInfo],
        instruction_data: &[u8],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let instruction = EscrowInstruction::unpack(instruction_data)?;

        match instruction {
            EscrowInstruction::InitEscrow { is_creator, amount } => {
                Self::process_init_escrow(accounts, is_creator, amount, program_id)
            }
            EscrowInstruction::WithdrawEscrow { result, amount } => {
                Self::process_withdraw(accounts, result, amount, program_id)
            }
        }
    }

    fn process_init_escrow(
        accounts: &[AccountInfo],
        is_creator: u8,
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let sender = next_account_info(account_info_iter)?;
        if !sender.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if amount < MINIMUM_DEPOSIT {
            return Err(EscrowError::MinimumAmountNotMet.into());
        }

        let fee_account = next_account_info(account_info_iter)?;
        let admin_account = next_account_info(account_info_iter)?;
        let escrow_account = next_account_info(account_info_iter)?;
        let pda_account = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
        let system_program_account = next_account_info(account_info_iter)?;

        // Validate system program
        if *system_program_account.key != solana_program::system_program::id() {
            return Err(EscrowError::InvalidSystemProgram.into());
        }

        // Validate escrow account ownership
        if escrow_account.owner != program_id {
            return Err(EscrowError::InvalidAccountOwner.into());
        }

        // Validate PDA account
        let (expected_pda, _nonce) = Pubkey::find_program_address(&[b"chess"], program_id);
        if *pda_account.key != expected_pda {
            return Err(EscrowError::InvalidPdaAccount.into());
        }

        if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()) {
            return Err(EscrowError::NotRentExempt.into());
        }

        // Calculate fees with overflow protection
        let fee_amount = amount
            .checked_mul(FEE_PERCENTAGE)
            .and_then(|x| x.checked_div(1000))
            .ok_or(EscrowError::AmountOverflow)?;

        let admin_amount = amount
            .checked_mul(ADMIN_PERCENTAGE)
            .and_then(|x| x.checked_div(1000))
            .ok_or(EscrowError::AmountOverflow)?;

        let escrow_amount = amount
            .checked_mul(ESCROW_PERCENTAGE)
            .and_then(|x| x.checked_div(1000))
            .ok_or(EscrowError::AmountOverflow)?;

        // Verify total matches (with rounding tolerance)
        let total_distributed = fee_amount
            .checked_add(admin_amount)
            .and_then(|x| x.checked_add(escrow_amount))
            .ok_or(EscrowError::AmountOverflow)?;

        if total_distributed > amount {
            return Err(EscrowError::AmountOverflow.into());
        }

        // Update escrow state
        {
            let mut escrow_info = Escrow::unpack_unchecked(&escrow_account.data.borrow())?;

            if is_creator == 1 {
                if escrow_info.is_initialized && escrow_info.creator_pubkey != Pubkey::default() {
                    return Err(EscrowError::InvalidAccount.into());
                }
                escrow_info.is_initialized = true;
                escrow_info.creator_pubkey = *sender.key;
                // Store full deposit amount (not escrow amount) for consistency with withdraw logic
                escrow_info.amount = amount;
            } else {
                if !escrow_info.is_initialized {
                    return Err(EscrowError::EscrowNotInitialized.into());
                }
                if escrow_info.competitor_pubkey != Pubkey::default() {
                    return Err(EscrowError::InvalidAccount.into());
                }
                escrow_info.competitor_pubkey = *sender.key;
                // Add full deposit amount (not escrow amount) to existing amount
                escrow_info.amount = escrow_info
                    .amount
                    .checked_add(amount)
                    .ok_or(EscrowError::AmountOverflow)?;
            }

            Escrow::pack(escrow_info, &mut escrow_account.try_borrow_mut_data()?)?;
        }

        // Transfer fees
        Self::transfer_sol(
            &[
                sender.clone(),
                fee_account.clone(),
                system_program_account.clone(),
            ],
            fee_amount,
        )?;

        Self::transfer_sol(
            &[
                sender.clone(),
                admin_account.clone(),
                system_program_account.clone(),
            ],
            admin_amount,
        )?;

        // Transfer to escrow PDA
        Self::transfer_sol(
            &[
                sender.clone(),
                pda_account.clone(),
                system_program_account.clone(),
            ],
            escrow_amount,
        )?;

        Ok(())
    }

    fn process_withdraw(
        accounts: &[AccountInfo],
        result: u8,
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        // Validate result value (0 = creator wins, 1 = competitor wins)
        if result > 1 {
            return Err(EscrowError::InvalidResultValue.into());
        }

        let account_info_iter = &mut accounts.iter();

        let admin_account = next_account_info(account_info_iter)?;
        if !admin_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let escrow_account = next_account_info(account_info_iter)?;
        let pda_account = next_account_info(account_info_iter)?;
        let system_program_account = next_account_info(account_info_iter)?;
        let creator = next_account_info(account_info_iter)?;
        let withdraw_account = next_account_info(account_info_iter)?;

        // Validate system program
        if *system_program_account.key != solana_program::system_program::id() {
            return Err(EscrowError::InvalidSystemProgram.into());
        }

        // Validate escrow account ownership
        if escrow_account.owner != program_id {
            return Err(EscrowError::InvalidAccountOwner.into());
        }

        // Validate PDA account
        let (expected_pda, nonce) = Pubkey::find_program_address(&[b"chess"], program_id);
        if *pda_account.key != expected_pda {
            return Err(EscrowError::InvalidPdaAccount.into());
        }

        // Unpack and validate escrow state
        let escrow_info = Escrow::unpack_unchecked(&escrow_account.data.borrow())?;

        if !escrow_info.is_initialized {
            return Err(EscrowError::EscrowNotInitialized.into());
        }

        // Verify both players have deposited (competitor must have joined)
        if escrow_info.competitor_pubkey == Pubkey::default() {
            return Err(EscrowError::BothPlayersMustDeposit.into());
        }

        // Validate creator account
        if escrow_info.creator_pubkey != *creator.key {
            return Err(EscrowError::InvalidAccount.into());
        }

        // Validate competitor account if result = 1
        if result == 1 {
            let competitor = next_account_info(account_info_iter)?;
            if escrow_info.competitor_pubkey != *competitor.key {
                return Err(EscrowError::InvalidAccount.into());
            }
            // Verify withdraw account matches competitor
            if *withdraw_account.key != escrow_info.competitor_pubkey {
                return Err(EscrowError::InvalidAccount.into());
            }
        } else {
            // Verify withdraw account matches creator
            if *withdraw_account.key != escrow_info.creator_pubkey {
                return Err(EscrowError::InvalidAccount.into());
            }
        }

        // Validate amount matches escrow amount
        if amount != escrow_info.amount {
            return Err(EscrowError::InvalidAmount.into());
        }

        // Calculate withdrawal amount (98% of total)
        let withdrawal_amount = amount
            .checked_mul(ESCROW_PERCENTAGE)
            .and_then(|x| x.checked_div(1000))
            .ok_or(EscrowError::AmountOverflow)?;

        // Transfer funds from PDA to winner
        let sol_ix = system_instruction::transfer(
            pda_account.key,
            withdraw_account.key,
            withdrawal_amount,
        );

        // CRITICAL FIX: Check return value of invoke_signed
        invoke_signed(
            &sol_ix,
            &[
                pda_account.clone(),
                withdraw_account.clone(),
                system_program_account.clone(),
            ],
            &[&[&b"chess"[..], &[nonce]]],
        )?;

        Ok(())
    }

    fn transfer_sol(accounts: &[AccountInfo], lamports: u64) -> ProgramResult {
        if lamports == 0 {
            return Ok(());
        }

        let account_info_iter = &mut accounts.iter();

        let source_acc = next_account_info(account_info_iter)?;
        let dest_acc = next_account_info(account_info_iter)?;
        let system_program_acc = next_account_info(account_info_iter)?;

        // Validate system program
        if *system_program_acc.key != solana_program::system_program::id() {
            return Err(EscrowError::InvalidSystemProgram.into());
        }

        let sol_ix = system_instruction::transfer(source_acc.key, dest_acc.key, lamports);
        invoke(
            &sol_ix,
            &[
                source_acc.clone(),
                dest_acc.clone(),
                system_program_acc.clone(),
            ],
        )?;

        Ok(())
    }
}
