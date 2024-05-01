use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey
};

pub mod state;
pub mod instruction;
pub mod processor;
pub mod error;

solana_program::entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let ixn = instruction::Instruction::unpack(instruction_data)?;
    match ixn {
        instruction::Instruction::CreateSwap(instruction) => {
            processor::process_create_swap(program_id, accounts, instruction.offered_amount, instruction.desired_amount)
        }
        instruction::Instruction::CancelSwap(_) => {
            processor::process_cancel_swap(accounts)
        }
    }
}