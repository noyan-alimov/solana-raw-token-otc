use solana_program::{account_info::{next_account_info, AccountInfo}, entrypoint::ProgramResult, program::{invoke, invoke_signed}, program_error::ProgramError, program_pack::{IsInitialized, Pack}, pubkey::Pubkey, rent::Rent, system_instruction::create_account, system_program, sysvar::Sysvar};
use spl_associated_token_account::tools::account::create_pda_account;
use spl_token::{instruction::{close_account, initialize_account3, transfer}, state::{Account as TokenAccount, Mint}};

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
    Mint::unpack(&mint_offered.data.borrow())
        .map_err(|_| SwapError::InvalidMintAccountData)?;

    // Check ata_creator_offered
    if ata_creator_offered.owner != token_program.key {
        return Err(ProgramError::IncorrectProgramId);
    }
    if !ata_creator_offered.is_writable {
        return Err(SwapError::AccountNotWritable.into());
    }
    let ata_creator_offered_data = TokenAccount::unpack(&ata_creator_offered.data.borrow())
        .map_err(|_| SwapError::InvalidTokenAccountData)?;
    if ata_creator_offered_data.mint != *mint_offered.key {
        return Err(SwapError::InvalidAtaCreatorOfferedMint.into());
    }
    if ata_creator_offered_data.owner != *creator.key {
        return Err(SwapError::InvalidAtaCreatorOfferedOwner.into());
    }

    // Check mint_desired
    if mint_desired.owner != token_program.key {
        return Err(ProgramError::IncorrectProgramId);
    }
    Mint::unpack(&mint_desired.data.borrow())
        .map_err(|_| SwapError::InvalidMintAccountData)?;

    // Check token program
    if token_program.key != &spl_token::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Check system program
    if system_program.key != &system_program::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Check swap and escrow
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
    invoke_signed(
        &create_swap_ixn,
        &[creator.clone(), swap.clone(), system_program.clone()],
        &[&[&b"swap"[..], ata_creator_offered.key.as_ref(), &[swap_bump]]]
    )?;

    // Initialize swap
    let mut swap_data = Swap::unpack_unchecked(&swap.data.borrow())
        .map_err(|_| SwapError::InvalidSwapAccountData)?;
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
    Swap::pack(swap_data, &mut swap.data.borrow_mut())
        .map_err(|_| SwapError::InvalidSwapAccountData)?;

    // Create escrow
    create_pda_account(
        creator,
        &rent,
        TokenAccount::LEN,
        token_program.key,
        system_program,
        ata_escrow,
        &[&b"escrow"[..], ata_creator_offered.key.as_ref(), &[escrow_bump]]
    )?;
    // Initialize escrow
    let initialize_escrow_ixn = initialize_account3(token_program.key, ata_escrow.key, mint_offered.key, swap.key)?;
    invoke(
        &initialize_escrow_ixn,
        &[ata_escrow.clone(), mint_offered.clone(), token_program.clone()]
    )?;

    // Transfer offered tokens to escrow
    let transfer_ixn = transfer(token_program.key, ata_creator_offered.key, ata_escrow.key, creator.key, &[creator.key], offered_amount)?;
    invoke(&transfer_ixn, &[ata_creator_offered.clone(), ata_escrow.clone(), creator.clone(), token_program.clone()])?;

    Ok(())
}

pub fn process_cancel_swap(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    // Get accounts
    let account_info_iter = &mut accounts.iter();
    let creator = next_account_info(account_info_iter)?;
    let ata_creator_offered = next_account_info(account_info_iter)?;
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

    // Check ata_creator_offered
    if ata_creator_offered.owner != token_program.key {
        return Err(ProgramError::IncorrectProgramId);
    }
    if !ata_creator_offered.is_writable {
        return Err(SwapError::AccountNotWritable.into());
    }
    let ata_creator_offered_data = TokenAccount::unpack(&ata_creator_offered.data.borrow())?;
    if ata_creator_offered_data.owner != *creator.key {
        return Err(ProgramError::InvalidAccountData);
    }

    // Check swap
    let (swap_pda, swap_bump) = Pubkey::find_program_address(&[b"swap", ata_creator_offered.key.as_ref()], program_id);
    let (escrow_pda, escrow_bump) = Pubkey::find_program_address(&[b"escrow", ata_creator_offered.key.as_ref()], program_id);
    let swap_data = Swap::unpack(&swap.data.borrow())?;
    if !swap.is_writable {
        return Err(SwapError::AccountNotWritable.into());
    }
    if swap.owner != program_id {
        return Err(ProgramError::InvalidArgument);
    }
    if swap.key != &swap_pda {
        return Err(ProgramError::InvalidArgument);
    }

    if !swap_data.is_initialized() {
        return Err(ProgramError::InvalidAccountData);
    }
    if swap_data.creator != *creator.key {
        return Err(ProgramError::InvalidAccountData);
    }
    if swap_data.ata_creator_offered != *ata_creator_offered.key {
        return Err(ProgramError::InvalidAccountData);
    }
    if swap_data.escrow != *ata_escrow.key {
        return Err(ProgramError::InvalidAccountData);
    }
    if swap_data.swap_bump != swap_bump {
        return Err(ProgramError::InvalidArgument);
    }
    if swap_data.escrow_bump != escrow_bump {
        return Err(ProgramError::InvalidAccountData);
    }

    // Check ata_escrow
    if ata_escrow.key != &escrow_pda {
        return Err(ProgramError::InvalidArgument);
    }
    if ata_escrow.owner != token_program.key {
        return Err(ProgramError::IncorrectProgramId);
    }
    if !ata_escrow.is_writable {
        return Err(SwapError::AccountNotWritable.into());
    }

    let escrow_data = TokenAccount::unpack(&ata_escrow.data.borrow())?;
    if escrow_data.owner != *swap.key {
        return Err(ProgramError::InvalidAccountData);
    }
    if escrow_data.mint != swap_data.offered_mint {
        return Err(ProgramError::InvalidAccountData);
    }

    // Check token program
    if token_program.key != &spl_token::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Check system program
    if system_program.key != &system_program::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Transfer offered tokens back to creator
    let transfer_ixn = transfer(token_program.key, ata_escrow.key, ata_creator_offered.key, swap.key, &[swap.key], swap_data.offered_amount)?;
    invoke_signed(
        &transfer_ixn,
        &[ata_escrow.clone(), ata_creator_offered.clone(), swap.clone(), token_program.clone()],
        &[&[&b"swap"[..], ata_creator_offered.key.as_ref(), &[swap_data.swap_bump]]]
    )?;

    // // Close swap
    **creator.try_borrow_mut_lamports()? = creator
        .lamports()
        .checked_add(swap.lamports())
        .ok_or(ProgramError::ArithmeticOverflow)?;
    **swap.try_borrow_mut_lamports()? = 0;
    *swap.try_borrow_mut_data()? = &mut [];

    // // Close escrow
    let close_ata_escrow_ixn = close_account(token_program.key, ata_escrow.key, creator.key, swap.key, &[swap.key])?;
    invoke_signed(
        &close_ata_escrow_ixn,
        &[ata_escrow.clone(), creator.clone(), swap.clone(), token_program.clone()],
        &[&[&b"swap"[..], ata_creator_offered.key.as_ref(), &[swap_data.swap_bump]]]
    )?;

    Ok(())
}