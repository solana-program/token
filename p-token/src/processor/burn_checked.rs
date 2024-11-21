use std::marker::PhantomData;

use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use super::shared;

#[inline(always)]
pub fn process_burn_checked(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    decimals: u8,
) -> ProgramResult {
    shared::burn::process_burn(program_id, accounts, amount, Some(decimals))
}

pub struct BurnChecked<'a> {
    raw: *const u8,

    _data: PhantomData<&'a [u8]>,
}

impl BurnChecked<'_> {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<BurnChecked, ProgramError> {
        if bytes.len() != 9 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(BurnChecked {
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
