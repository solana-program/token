use {
    super::shared,
    pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult},
};

pub fn process_initialize_multisig2(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let m = instruction_data
        .first()
        .ok_or(ProgramError::InvalidInstructionData)?;
    shared::initialize_multisig::process_initialize_multisig(accounts, *m, false)
}
