use std::marker::PhantomData;

use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use super::shared;

#[inline(never)]
pub fn process_mint_to_checked(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    decimals: u8,
) -> ProgramResult {
    shared::mint_to::process_mint_to(program_id, accounts, amount, Some(decimals))
}

pub struct MintToChecked<'a> {
    raw: *const u8,

    _data: PhantomData<&'a [u8]>,
}

impl MintToChecked<'_> {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<MintToChecked, ProgramError> {
        if bytes.len() != 9 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(MintToChecked {
            raw: bytes.as_ptr(),
            _data: PhantomData,
        })
    }

    pub fn amount(&self) -> u64 {
        unsafe {
            let amount = self.raw as *const u64;
            amount.read_unaligned()
        }
    }

    pub fn decimals(&self) -> u8 {
        unsafe { *self.raw.add(8) }
    }
}
