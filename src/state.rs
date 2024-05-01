use solana_program::{program_error::ProgramError, program_pack::{IsInitialized, Pack, Sealed}, pubkey::Pubkey};
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

pub struct Swap {
    pub is_initialized: bool,
    pub creator: Pubkey,
    pub offered_mint: Pubkey,
    pub desired_mint: Pubkey,
    pub ata_creator_offered: Pubkey,
    pub escrow: Pubkey,
    pub offered_amount: u64,
    pub desired_amount: u64,
    pub swap_bump: u8,
    pub escrow_bump: u8,
}

impl Sealed for Swap {}

impl IsInitialized for Swap {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Swap {
    const LEN: usize = 1 + (5 * 32) + (2 * 8) + (2 * 1);

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, Swap::LEN];
        let (
            is_initialized,
            creator,
            offered_mint,
            desired_mint,
            ata_creator_offered,
            escrow,
            offered_amount,
            desired_amount,
            swap_bump,
            escrow_bump,
        ) = array_refs![src, 1, 32, 32, 32, 32, 32, 8, 8, 1, 1];

        Ok(Swap {
            is_initialized: match is_initialized {
                [0] => false,
                [1] => true,
                _ => return Err(ProgramError::InvalidAccountData),
            },
            creator: Pubkey::new_from_array(*creator),
            offered_mint: Pubkey::new_from_array(*offered_mint),
            desired_mint: Pubkey::new_from_array(*desired_mint),
            ata_creator_offered: Pubkey::new_from_array(*ata_creator_offered),
            escrow: Pubkey::new_from_array(*escrow),
            offered_amount: u64::from_le_bytes(*offered_amount),
            desired_amount: u64::from_le_bytes(*desired_amount),
            swap_bump: swap_bump[0],
            escrow_bump: escrow_bump[0],
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, Swap::LEN];
        let (
            is_initialized,
            creator,
            offered_mint,
            desired_mint,
            ata_creator_offered,
            escrow,
            offered_amount,
            desired_amount,
            swap_bump,
            escrow_bump,
        ) = mut_array_refs![dst, 1, 32, 32, 32, 32, 32, 8, 8, 1, 1];
        match self.is_initialized {
            true => is_initialized[0] = 1,
            false => is_initialized[0] = 0,
        };
        creator.copy_from_slice(self.creator.as_ref());
        offered_mint.copy_from_slice(self.offered_mint.as_ref());
        desired_mint.copy_from_slice(self.desired_mint.as_ref());
        ata_creator_offered.copy_from_slice(self.ata_creator_offered.as_ref());
        escrow.copy_from_slice(self.escrow.as_ref());
        *offered_amount = self.offered_amount.to_le_bytes();
        *desired_amount = self.desired_amount.to_le_bytes();
        swap_bump[0] = self.swap_bump;
        escrow_bump[0] = self.escrow_bump;
    }
}