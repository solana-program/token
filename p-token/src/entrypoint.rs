use pinocchio::{
    account_info::AccountInfo, default_panic_handler, no_allocator, program_entrypoint,
    program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};
use spl_token_interface::instruction::TokenInstruction;

use crate::processor::*;

program_entrypoint!(process_instruction);
// Do not allocate memory.
no_allocator!();
// Use the default panic handler.
default_panic_handler!();

/// Process an instruction.
///
/// In the first stage, the entrypoint checks the discriminator of the instruction data
/// to determine whether the instruction is a "batch" instruction or a "regular" instruction.
/// This avoids nesting of "batch" instructions, since it is not sound to have a "batch"
/// instruction inside another "batch" instruction.
#[inline(always)]
pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [discriminator, remaining @ ..] = instruction_data else {
        return Err(ProgramError::InvalidInstructionData);
    };

    if *discriminator == 255 {
        // 255 - Batch
        #[cfg(feature = "logging")]
        pinocchio::msg!("Instruction: Batch");

        return process_batch(accounts, remaining);
    }

    inner_process_instruction(accounts, instruction_data)
}

/// Process a "regular" instruction.
///
/// The processor of the token program is divided into two parts to reduce the overhead
/// of having a large `match` statement. The first part of the processor handles the
/// most common instructions, while the second part handles the remaining instructions.
///
/// The rationale is to reduce the overhead of making multiple comparisons for popular
/// instructions.
///
/// Instructions on the first part of the inner processor:
///
/// -  `0`: `InitializeMint`
/// -  `1`: `InitializeAccount`
/// -  `3`: `Transfer`
/// -  `7`: `MintTo`
/// -  `9`: `CloseAccount`
/// - `16`: `InitializeAccount2`
/// - `18`: `InitializeAccount3`
/// - `20`: `InitializeMint2`
#[inline(always)]
pub(crate) fn inner_process_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [discriminator, instruction_data @ ..] = instruction_data else {
        return Err(ProgramError::InvalidInstructionData);
    };

    match instruction {
        // 0 - InitializeMint
        TokenInstruction::InitializeMint => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeMint");

            process_initialize_mint(accounts, instruction_data)
        }
        // 1 - InitializeAccount
        1 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeAccount");

            process_initialize_account(accounts)
        }
        // 3 - Transfer
        TokenInstruction::Transfer => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: Transfer");

            process_transfer(accounts, instruction_data)
        }
        // 7 - MintTo
        TokenInstruction::MintTo => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: MintTo");

            process_mint_to(accounts, instruction_data)
        }
        // 9 - CloseAccount
        TokenInstruction::CloseAccount => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: CloseAccount");

            process_close_account(accounts)
        }
        // 16 - InitializeAccount2
        16 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeAccount2");

            process_initialize_account2(accounts, instruction_data)
        }
        // 18 - InitializeAccount3
        TokenInstruction::InitializeAccount3 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeAccount3");

            process_initialize_account3(accounts, instruction_data)
        }
        // 20 - InitializeMint2
        TokenInstruction::InitializeMint2 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeMint2");

            process_initialize_mint2(accounts, instruction_data)
        }
        d => inner_process_remaining_instruction(accounts, instruction_data, d),
    }
}

/// Process a remaining "regular" instruction.
///
/// This function is called by the [`inner_process_instruction`] function if the discriminator
/// does not match any of the common instructions. This function is used to reduce the
/// overhead of having a large `match` statement in the [`inner_process_instruction`] function.
fn inner_process_remaining_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
    instruction: TokenInstruction,
) -> ProgramResult {
    match discriminator {
        // 2 - InitializeMultisig
        TokenInstruction::InitializeMultisig => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeMultisig");

            process_initialize_multisig(accounts, instruction_data)
        }
        // 4 - Approve
        TokenInstruction::Approve => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: Approve");

            process_approve(accounts, instruction_data)
        }
        // 5 - Revoke
        TokenInstruction::Revoke => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: Revoke");

            process_revoke(accounts, instruction_data)
        }
        // 6 - SetAuthority
        TokenInstruction::SetAuthority => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: SetAuthority");

            process_set_authority(accounts, instruction_data)
        }
        // 8 - Burn
        TokenInstruction::Burn => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: Burn");

            process_burn(accounts, instruction_data)
        }
        // 10 - FreezeAccount
        TokenInstruction::FreezeAccount => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: FreezeAccount");

            process_freeze_account(accounts)
        }
        // 11 - ThawAccount
        TokenInstruction::ThawAccount => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: ThawAccount");

            process_thaw_account(accounts)
        }
        // 12 - TransferChecked
        TokenInstruction::TransferChecked => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: TransferChecked");

            process_transfer_checked(accounts, instruction_data)
        }
        // 13 - ApproveChecked
        TokenInstruction::ApproveChecked => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: ApproveChecked");

            process_approve_checked(accounts, instruction_data)
        }
        // 14 - MintToChecked
        TokenInstruction::MintToChecked => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: MintToChecked");

            process_mint_to_checked(accounts, instruction_data)
        }
        // 15 - BurnChecked
        TokenInstruction::BurnChecked => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: BurnChecked");

            process_burn_checked(accounts, instruction_data)
        }
        // 17 - SyncNative
        TokenInstruction::SyncNative => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: SyncNative");

            process_sync_native(accounts)
        }
        // 19 - InitializeMultisig2
        TokenInstruction::InitializeMultisig2 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeMultisig2");

            process_initialize_multisig2(accounts, instruction_data)
        }
        // 21 - GetAccountDataSize
        TokenInstruction::GetAccountDataSize => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: GetAccountDataSize");

            process_get_account_data_size(accounts)
        }
        // 22 - InitializeImmutableOwner
        TokenInstruction::InitializeImmutableOwner => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeImmutableOwner");

            process_initialize_immutable_owner(accounts)
        }
        // 23 - AmountToUiAmount
        TokenInstruction::AmountToUiAmount => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: AmountToUiAmount");

            process_amount_to_ui_amount(accounts, instruction_data)
        }
        // 24 - UiAmountToAmount
        TokenInstruction::UiAmountToAmount => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: UiAmountToAmount");

            process_ui_amount_to_amount(accounts, instruction_data)
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
