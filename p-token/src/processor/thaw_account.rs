use pinocchio::{account_info::AccountInfo, pubkey::Pubkey, ProgramResult};

use super::toggle_account_state::process_toggle_account_state;

#[inline(always)]
pub fn process_thaw_account(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    process_toggle_account_state(program_id, accounts, false)
}
