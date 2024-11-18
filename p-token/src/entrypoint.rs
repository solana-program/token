use pinocchio::{
    account_info::AccountInfo, entrypoint, program_error::ProgramError, pubkey::Pubkey,
    ProgramResult,
};

use crate::processor::{
    approve::process_approve,
    burn::process_burn,
    close_account::process_close_account,
    freeze_account::process_freeze_account,
    initialize_account::process_initialize_account,
    initialize_mint::{process_initialize_mint, InitializeMint},
    initialize_multisig::process_initialize_multisig,
    mint_to::process_mint_to,
    revoke::process_revoke,
    set_authority::{process_set_authority, SetAuthority},
    thaw_account::process_thaw_account,
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
        // 2 - InitializeMultisig
        Some((&2, data)) => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeMultisig");

            let m = data.first().ok_or(ProgramError::InvalidInstructionData)?;

            process_initialize_multisig(accounts, *m, true)
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
        // 4 - Approve
        Some((&4, data)) => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: Approve");

            let amount = u64::from_le_bytes(
                data.try_into()
                    .map_err(|_error| ProgramError::InvalidInstructionData)?,
            );

            process_approve(program_id, accounts, amount, None)
        }
        // 5 - Revoke
        Some((&5, _)) => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: Revoke");

            process_revoke(program_id, accounts)
        }
        // 6 - SetAuthority
        Some((&6, data)) => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: SetAuthority");

            let instruction = SetAuthority::try_from_bytes(data)?;
            process_set_authority(
                program_id,
                accounts,
                instruction.authority_type,
                instruction.new_authority,
            )
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
        // 8 - Burn
        Some((&8, data)) => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: Burn");

            let amount = u64::from_le_bytes(
                data.try_into()
                    .map_err(|_error| ProgramError::InvalidInstructionData)?,
            );

            process_burn(program_id, accounts, amount, None)
        }
        // 9 - CloseAccount
        Some((&9, _)) => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: CloseAccount");

            process_close_account(program_id, accounts)
        }
        // 10 - FreezeAccount
        Some((&10, _)) => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: FreezeAccount");

            process_freeze_account(program_id, accounts)
        }
        // 10 - ThawAccount
        Some((&11, _)) => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: ThawAccount");

            process_thaw_account(program_id, accounts)
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
