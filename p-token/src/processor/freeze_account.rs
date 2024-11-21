use pinocchio::{account_info::AccountInfo, pubkey::Pubkey, ProgramResult};

use super::shared::toggle_account_state::process_toggle_account_state;

pub fn process_freeze_account(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    process_toggle_account_state(program_id, accounts, true)
}
