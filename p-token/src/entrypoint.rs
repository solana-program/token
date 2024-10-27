use pinocchio::{
    account_info::AccountInfo, entrypoint, program_error::ProgramError, pubkey::Pubkey,
    ProgramResult,
};

use crate::processor::{
    initialize_account::process_initialize_account,
    initialize_mint::{process_initialize_mint, InitializeMint},
    mint_to::process_mint_to,
    transfer::process_transfer,
};

entrypoint!(process_instruction);

#[inline(always)]
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data.split_first() {
        // 0 - InitializeMint
        Some((&0, data)) => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeMint");

            let instruction = InitializeMint::try_from_bytes(data)?;
            process_initialize_mint(accounts, &instruction, true)
        }
        // 1 - InitializeAccount
        Some((&1, _)) => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeAccount");

            process_initialize_account(program_id, accounts, None, true)
        }
        // 3 - Transfer
        Some((&3, data)) => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: Transfer");

            let amount = u64::from_le_bytes(
                data.try_into()
                    .map_err(|_error| ProgramError::InvalidInstructionData)?,
            );

            process_transfer(program_id, accounts, amount, None)
        }
        // 7 - InitializeMint
        Some((&7, data)) => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: MintTo");

            let amount = u64::from_le_bytes(
                data.try_into()
                    .map_err(|_error| ProgramError::InvalidInstructionData)?,
            );

            process_mint_to(program_id, accounts, amount, None)
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
