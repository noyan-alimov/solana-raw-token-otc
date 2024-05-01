use solana_program::{account_info::{next_account_info, AccountInfo}, entrypoint::ProgramResult, program::invoke, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey, rent::Rent, system_instruction::create_account, system_program, sysvar::Sysvar};
use spl_token::{instruction::{initialize_account3, transfer}, state::{Account as TokenAccount, Mint}};

use crate::{error::SwapError, state::Swap};


pub fn process_create_swap(program_id: &Pubkey, accounts: &[AccountInfo], offered_amount: u64, desired_amount: u64) -> ProgramResult {
    // Get accounts
    let account_info_iter = &mut accounts.iter();
    let creator = next_account_info(account_info_iter)?;
    let mint_offered = next_account_info(account_info_iter)?;
    let ata_creator_offered = next_account_info(account_info_iter)?;
    let mint_desired = next_account_info(account_info_iter)?;
    let swap = next_account_info(account_info_iter)?;
    let ata_escrow = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    // Check creator
    if !creator.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !creator.is_writable {
        return Err(SwapError::AccountNotWritable.into());
    }

    // Check mint_offered
    if mint_offered.owner != token_program.key {
        return Err(ProgramError::IncorrectProgramId);
    }
    Mint::unpack(&mint_offered.data.borrow())?;

    // Check ata_creator_offered
    if ata_creator_offered.owner != token_program.key {
        return Err(ProgramError::IncorrectProgramId);
    }
    if !ata_creator_offered.is_writable {
        return Err(SwapError::AccountNotWritable.into());
    }
    let ata_creator_offered_data = TokenAccount::unpack(&ata_creator_offered.data.borrow())?;
    if ata_creator_offered_data.mint != *mint_offered.key {
        return Err(ProgramError::InvalidAccountData);
    }
    if ata_creator_offered_data.owner != *creator.key {
        return Err(ProgramError::InvalidAccountData);
    }

    // Check mint_desired
    if mint_desired.owner != token_program.key {
        return Err(ProgramError::IncorrectProgramId);
    }
    Mint::unpack(&mint_desired.data.borrow())?;

    // Check token program
    if token_program.key != &spl_token::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Check system program
    if system_program.key != &system_program::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Check swap and escrow
    if !swap.is_writable {
        return Err(SwapError::AccountNotWritable.into());
    }
    let mut swap_data = Swap::unpack_unchecked(&swap.data.borrow())?;
    let (swap_pda, swap_bump) = Pubkey::find_program_address(&[b"swap", ata_creator_offered.key.as_ref()], program_id);
    let (escrow_pda, escrow_bump) = Pubkey::find_program_address(&[b"escrow", ata_creator_offered.key.as_ref()], program_id);
    if *swap.key != swap_pda {
        return Err(ProgramError::InvalidArgument);
    }
    if *ata_escrow.key != escrow_pda {
        return Err(ProgramError::InvalidArgument);
    }

    // Create swap
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(Swap::LEN);
    let create_swap_ixn = create_account(creator.key, swap.key, lamports, Swap::LEN.try_into().unwrap(), program_id);
    invoke(&create_swap_ixn, &[creator.clone(), swap.clone(), system_program.clone()])?;

    // Create escrow
    let lamports = rent.minimum_balance(TokenAccount::LEN);
    let create_escrow_ixn = create_account(creator.key, ata_escrow.key, lamports, TokenAccount::LEN.try_into().unwrap(), program_id);
    invoke(&create_escrow_ixn, &[creator.clone(), ata_escrow.clone(), system_program.clone()])?;
    // Initialize escrow
    let initialize_escrow_ixn = initialize_account3(token_program.key, ata_escrow.key, mint_offered.key, swap.key)?;
    invoke(&initialize_escrow_ixn, &[ata_escrow.clone(), mint_offered.clone(), token_program.clone()])?;

    // Initialize swap
    swap_data.is_initialized = true;
    swap_data.creator = *creator.key;
    swap_data.offered_mint = *mint_offered.key;
    swap_data.desired_mint = *mint_desired.key;
    swap_data.ata_creator_offered = *ata_creator_offered.key;
    swap_data.escrow = *ata_escrow.key;
    swap_data.offered_amount = offered_amount;
    swap_data.desired_amount = desired_amount;
    swap_data.swap_bump = swap_bump;
    swap_data.escrow_bump = escrow_bump;
    Swap::pack(swap_data, &mut swap.data.borrow_mut())?;

    // Transfer offered tokens to escrow
    let transfer_ixn = transfer(token_program.key, ata_creator_offered.key, ata_escrow.key, creator.key, &[creator.key], offered_amount)?;
    invoke(&transfer_ixn, &[ata_creator_offered.clone(), ata_escrow.clone(), creator.clone(), token_program.clone()])?;

    Ok(())
}

pub fn process_cancel_swap(accounts: &[AccountInfo]) -> ProgramResult {
    Ok(())
}