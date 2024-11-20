use pinocchio::{
    account_info::AccountInfo, entrypoint, program_error::ProgramError, pubkey::Pubkey,
    ProgramResult,
};

use crate::processor::{
    approve::process_approve,
    approve_checked::{process_approve_checked, ApproveChecked},
    burn::process_burn,
    burn_checked::{process_burn_checked, BurnChecked},
    close_account::process_close_account,
    freeze_account::process_freeze_account,
    initialize_account::process_initialize_account,
    initialize_account2::process_initialize_account2,
    initialize_account3::process_initialize_account3,
    initialize_mint::{process_initialize_mint, InitializeMint},
    initialize_mint2::process_initialize_mint2,
    initialize_multisig::process_initialize_multisig,
    initialize_multisig2::process_initialize_multisig2,
    mint_to::process_mint_to,
    mint_to_checked::{process_mint_to_checked, MintToChecked},
    revoke::process_revoke,
    set_authority::{process_set_authority, SetAuthority},
    thaw_account::process_thaw_account,
    transfer::process_transfer,
    transfer_checked::{process_transfer_checked, TransferChecked},
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
        // 11 - ThawAccount
        Some((&11, _)) => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: ThawAccount");

            process_thaw_account(program_id, accounts)
        }
        // 12 - TransferChecked
        Some((&12, data)) => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: TransferChecked");

            let args = TransferChecked::try_from_bytes(data)?;

            process_transfer_checked(program_id, accounts, args.amount(), args.decimals())
        }
        // 13 - ApproveChecked
        Some((&13, data)) => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: ApproveChecked");

            let args = ApproveChecked::try_from_bytes(data)?;

            process_approve_checked(program_id, accounts, args.amount(), args.decimals())
        }
        // 14 - MintToChecked
        Some((&14, data)) => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: MintToChecked");

            let args = MintToChecked::try_from_bytes(data)?;

            process_mint_to_checked(program_id, accounts, args.amount(), args.decimals())
        }
        // 15 - BurnChecked
        Some((&15, data)) => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: BurnChecked");

            let args = BurnChecked::try_from_bytes(data)?;

            process_burn_checked(program_id, accounts, args.amount(), args.decimals())
        }
        // 16 - InitializeAccount2
        Some((&16, data)) => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeAccount2");

            let owner = unsafe { &*(data.as_ptr() as *const Pubkey) };

            process_initialize_account2(program_id, accounts, owner)
        }
        // 18 - InitializeAccount3
        Some((&18, data)) => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeAccount3");

            let owner = unsafe { &*(data.as_ptr() as *const Pubkey) };

            process_initialize_account3(program_id, accounts, owner)
        }
        // 19 - InitializeMultisig2
        Some((&19, data)) => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeMultisig2");

            let m = data.first().ok_or(ProgramError::InvalidInstructionData)?;

            process_initialize_multisig2(accounts, *m)
        }
        // 20 - InitializeMint2
        Some((&20, data)) => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeMint2");

            let instruction = InitializeMint::try_from_bytes(data)?;

            process_initialize_mint2(accounts, &instruction)
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
