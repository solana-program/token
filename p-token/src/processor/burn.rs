use pinocchio::{account_info::AccountInfo, pubkey::Pubkey, ProgramResult};

use super::shared;

#[inline(never)]
pub fn process_burn(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    shared::burn::process_burn(program_id, accounts, amount, None)
}
