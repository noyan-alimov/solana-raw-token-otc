use solana_program::program_error::ProgramError;


pub struct CreateSwap {
    pub offered_amount: u64,
    pub desired_amount: u64,
}

pub struct CancelSwap {}

pub enum Instruction {
    /// Creates a new swap.
    /// Transfers offered tokens to escrow.
    ///
    ///   0. `[writable, signer]`  Creator of the swap.
    ///   1. `[]` Mint of creator's offered tokens.
    ///   2. `[writable]` The SPL Account (TokenAccount / ATA) of the creator.
    ///   3. `[]` Mint of desired tokens.
    ///   4. `[writable]` Swap state account, gets created and initialized in this instruction. Seeds = [b"swap", ata_creator_offered].
    ///   5. `[writable]` Escrow, the SPL Account (TokenAccount / ATA) that holds the offered tokens.
    /// Gets created and initialized in this instruction.
    /// Authority (Owner) is the Swap account, mint is the offered token.
    /// Seeds = [b"escrow", ata_creator_offered].
    ///   6. `[]` The SPL token program.
    ///   7. `[]` System program.
    CreateSwap(CreateSwap),

    /// Cancels a swap.
    /// Transfers offered tokens back to creator.
    /// 
    ///  0. `[writable, signer]`  Creator of the swap.
    ///  1. `[writable]` The SPL Account (TokenAccount / ATA) of the creator.
    ///  2. `[writable]` Swap state account, gets closed in this instruction. Seeds = [b"swap", ata_creator_offered].
    ///  3. `[writable]` Escrow, the SPL Account (TokenAccount / ATA) that holds the offered tokens. Gets closed in this instruction. Seeds = [b"escrow", ata_creator_offered].
    ///  4. `[]` The SPL token program.
    ///  5. `[]` System program.
    CancelSwap(CancelSwap),
}

impl Instruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(ProgramError::InvalidInstructionData)?;

        Ok(match tag {
            0 => {
                let (offered_amount, rest) = Self::unpack_u64(rest)?;
                let (desired_amount, _rest) = Self::unpack_u64(rest)?;

                Instruction::CreateSwap(CreateSwap {
                    offered_amount,
                    desired_amount,
                })
            }
            1 => {
                Instruction::CancelSwap(CancelSwap {})
            }
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }

    fn unpack_u64(input: &[u8]) -> Result<(u64, &[u8]), ProgramError> {
        if input.len() >= 8 {
            let (amount, rest) = input.split_at(8);
            let amount = amount
                .get(..8)
                .and_then(|slice| slice.try_into().ok())
                .map(u64::from_le_bytes)
                .ok_or(ProgramError::InvalidInstructionData)?;
            Ok((amount, rest))
        } else {
            Err(ProgramError::InvalidInstructionData.into())
        }
    }
}