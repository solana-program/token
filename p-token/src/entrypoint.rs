use {
    crate::processor::*,
    pinocchio::{
        account_info::AccountInfo,
        no_allocator, nostd_panic_handler, program_entrypoint,
        program_error::{ProgramError, ToStr},
        pubkey::Pubkey,
        ProgramResult,
    },
    spl_token_interface::{error::TokenError, state::{Initializable, Transmutable}},
};

program_entrypoint!(process_instruction);
// Do not allocate memory.
no_allocator!();
// Use the no_std panic handler.
nostd_panic_handler!();

/// Log an error.
#[cold]
fn log_error(error: &ProgramError) {
    pinocchio::log::sol_log(error.to_str::<TokenError>());
}

/// Process an instruction.
///
/// In the first stage, the entrypoint checks the discriminator of the
/// instruction data to determine whether the instruction is a "batch"
/// instruction or a "regular" instruction. This avoids nesting of "batch"
/// instructions, since it is not sound to have a "batch" instruction inside
/// another "batch" instruction.
#[inline(always)]
pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [discriminator, remaining @ ..] = instruction_data else {
        return Err(TokenError::InvalidInstruction.into());
    };

    let result = if *discriminator == 255 {
        // 255 - Batch
        #[cfg(feature = "logging")]
        pinocchio::msg!("Instruction: Batch");

        process_batch(accounts, remaining)
    } else {
        inner_process_instruction(accounts, instruction_data)
    };

    result.inspect_err(log_error)
}

/// Process a "regular" instruction.
///
/// The processor of the token program is divided into two parts to reduce the
/// overhead of having a large `match` statement. The first part of the
/// processor handles the most common instructions, while the second part
/// handles the remaining instructions.
///
/// The rationale is to reduce the overhead of making multiple comparisons for
/// popular instructions.
///
/// Instructions on the first part of the inner processor:
///
/// - `0`: `InitializeMint`
/// - `1`: `InitializeAccount`
/// - `3`: `Transfer`
/// - `7`: `MintTo`
/// - `9`: `CloseAccount`
/// - `16`: `InitializeAccount2`
/// - `18`: `InitializeAccount3`
/// - `20`: `InitializeMint2`
#[inline(always)]
pub(crate) fn inner_process_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [discriminator, instruction_data @ ..] = instruction_data else {
        return Err(TokenError::InvalidInstruction.into());
    };

    match *discriminator {
        // 0 - InitializeMint
        0 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeMint");

            process_initialize_mint(accounts, instruction_data)
        }
        // 100 - Test InitializeMint
        100 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: InitializeMint");

            let accounts = [
                accounts[0].clone(), // Mint Info
                accounts[1].clone(), // Rent Sysvar Info
            ];

            let instruction_data = [
                // Decimals: instruction_data[0]
                1,
                // Mint Authority: instruction_data[1..33]
                2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33,
                // Freeze Authority: instruction_data[33]
                34,
            ];

            test_process_initialize_mint(&accounts, &instruction_data)
        }
        // 1 - InitializeAccount
        1 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeAccount");

            process_initialize_account(accounts)
        }
        // 101 - Test InitializeAccount
        101 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: InitializeAccount");

            test_process_initialize_account(accounts.first_chunk().unwrap())
        }
        // 3 - Transfer
        3 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: Transfer");

            process_transfer(accounts, instruction_data)
        }
        // 103 - Test Transfer
        103 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: Transfer");

            let accounts = [
                accounts[0].clone(), // Source Account Info
                accounts[1].clone(), // Mint Info
                accounts[2].clone(), // Destination Account Info
                accounts[3].clone(), // Authority Info
            ];

            let instruction_data = [
                // LE bytes for amount
                1, 2, 3, 4, 5, 6, 7, 8,
            ];

            test_process_transfer(&accounts, &instruction_data)
        }
        // 7 - MintTo
        7 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: MintTo");

            process_mint_to(accounts, instruction_data)
        }
        // 107 - MintTo
        107 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: MintTo");

            let accounts = [
                accounts[0].clone(), // Mint Info
                accounts[1].clone(), // Destination Account Info
                accounts[2].clone(), // Owner Info
            ];

            let instruction_data = [
                // LE bytes for amount
                1, 2, 3, 4, 5, 6, 7, 8,
            ];

            test_process_mint_to(&accounts, &instruction_data)
        }
        // 8 - Burn
        8 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: Burn");

            process_burn(accounts, instruction_data)
        }
        // 108 - Test Burn
        108 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: Burn");

            let accounts = [
                accounts[0].clone(), // Source Account Info
                accounts[1].clone(), // Mint Info
                accounts[2].clone(), // Authority Info
            ];

            let instruction_data = [
                // LE bytes for amount
                1, 2, 3, 4, 5, 6, 7, 8,
            ];

            test_process_burn(&accounts, &instruction_data)
        }
        // 9 - CloseAccount
        9 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: CloseAccount");

            process_close_account(accounts)
        }
        // 109 - Test CloseAccount
        109 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: CloseAccount");

            let accounts = [
                accounts[0].clone(), // Source Account Info
                accounts[1].clone(), // Destination Account Info
                accounts[2].clone(), // Authority Info
            ];

            test_process_close_account(&accounts)
        }
        // 12 - TransferChecked
        12 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: TransferChecked");

            process_transfer_checked(accounts, instruction_data)
        }
        // 112 - Test TransferChecked
        112 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: TransferChecked");

            let accounts = [
                accounts[0].clone(), // Source Account Info
                accounts[1].clone(), // Mint Info
                accounts[2].clone(), // Destination Account Info
                accounts[3].clone(), // Authority Info
            ];

            let instruction_data = [
                // LE bytes for amount
                1, 2, 3, 4, 5, 6, 7, 8,
                // Decimals
                9,
            ];

            test_process_transfer_checked(&accounts, &instruction_data)
        }
        // 15 - BurnChecked
        15 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: BurnChecked");

            process_burn_checked(accounts, instruction_data)
        }
        // 115 - Test BurnChecked
        115 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: BurnChecked");

            let accounts = [
                accounts[0].clone(), // Source Account Info
                accounts[1].clone(), // Mint Info
                accounts[2].clone(), // Authority Info
            ];

            let instruction_data = [
                // LE bytes for amount
                1, 2, 3, 4, 5, 6, 7, 8,
                // Decimals
                9,
            ];

            test_process_burn_checked(&accounts, &instruction_data)
        }
        // 16 - InitializeAccount2
        16 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeAccount2");

            process_initialize_account2(accounts, instruction_data)
        }
        // 116 - Test InitializeAccount2
        116 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: InitializeAccount2");

            let accounts = [
                accounts[0].clone(), // New Account Info
                accounts[1].clone(), // Mint Info
                // accounts[2].clone // Owner Info
            ];

            let instruction_data = [
                // Owner Pubkey (If not provided in `accounts`)
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
            ];

            test_process_initialize_account2(&accounts, &instruction_data)
        }
        // 18 - InitializeAccount3
        18 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeAccount3");

            process_initialize_account3(accounts, instruction_data)
        }
        // 118 - Test InitializeAccount3
        118 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: InitializeAccount3");

            let accounts = [
                accounts[0].clone(), // New Account Info
                accounts[1].clone(), // Mint Info
                // accounts[2].clone // Owner Info
            ];

            let instruction_data = [
                // Owner Pubkey (If not provided in `accounts`)
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
            ];

            test_process_initialize_account3(&accounts, &instruction_data)
        }
        // 20 - InitializeMint2
        20 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeMint2");

            process_initialize_mint2(accounts, instruction_data)
        }
        // 120 - Test InitializeMint2
        120 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: InitializeMint2");

            let accounts = [
                accounts[0].clone(), // Mint Info
                accounts[1].clone(), // Rent Sysvar Info
            ];

            let instruction_data = [
                // Decimals: instruction_data[0]
                1,
                // Mint Authority: instruction_data[1..33]
                2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33,
                // Freeze Authority: instruction_data[33]
                34,
            ];

            test_process_initialize_mint2(&accounts, &instruction_data)
        }
        d => inner_process_remaining_instruction(accounts, instruction_data, d),
    }
}

/// Process a remaining "regular" instruction.
///
/// This function is called by the [`inner_process_instruction`] function if the
/// discriminator does not match any of the common instructions. This function
/// is used to reduce the overhead of having a large `match` statement in the
/// [`inner_process_instruction`] function.
fn inner_process_remaining_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
    discriminator: u8,
) -> ProgramResult {
    match discriminator {
        // 2 - InitializeMultisig
        2 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeMultisig");

            process_initialize_multisig(accounts, instruction_data)
        }
        // 4 - Approve
        4 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: Approve");

            process_approve(accounts, instruction_data)
        }
        // 5 - Revoke
        5 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: Revoke");

            process_revoke(accounts)
        }
        // 6 - SetAuthority
        6 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: SetAuthority");

            process_set_authority(accounts, instruction_data)
        }
        // 10 - FreezeAccount
        10 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: FreezeAccount");

            process_freeze_account(accounts)
        }
        // 11 - ThawAccount
        11 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: ThawAccount");

            process_thaw_account(accounts)
        }
        // 13 - ApproveChecked
        13 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: ApproveChecked");

            process_approve_checked(accounts, instruction_data)
        }
        // 14 - MintToChecked
        14 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: MintToChecked");

            process_mint_to_checked(accounts, instruction_data)
        }
        // 17 - SyncNative
        17 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: SyncNative");

            process_sync_native(accounts)
        }
        // 19 - InitializeMultisig2
        19 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeMultisig2");

            process_initialize_multisig2(accounts, instruction_data)
        }
        // 21 - GetAccountDataSize
        21 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: GetAccountDataSize");

            process_get_account_data_size(accounts)
        }
        // 22 - InitializeImmutableOwner
        22 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeImmutableOwner");

            process_initialize_immutable_owner(accounts)
        }
        // 23 - AmountToUiAmount
        23 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: AmountToUiAmount");

            process_amount_to_ui_amount(accounts, instruction_data)
        }
        // 24 - UiAmountToAmount
        24 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: UiAmountToAmount");

            process_ui_amount_to_amount(accounts, instruction_data)
        }
        // 38 - WithdrawExcessLamports
        38 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: WithdrawExcessLamports");

            process_withdraw_excess_lamports(accounts)
        }
        _ => Err(TokenError::InvalidInstruction.into()),
    }
}

// Hack Tests For Stable MIR JSON ---------------------------------------------
#[inline(never)]
pub fn test_process_initialize_mint(accounts: &[AccountInfo; 2], instruction_data: &[u8; 34]) -> ProgramResult {
    process_initialize_mint(accounts, instruction_data)
}

/// accounts[0] // New Account Info
/// accounts[1] // Mint Info
/// accounts[2] // Owner Info
/// accounts[3] // Rent Sysvar Info
#[inline(never)]
pub fn test_process_initialize_account(accounts: &[AccountInfo; 4]) -> ProgramResult {
    use spl_token_interface::state::{account, account_state};

    // TODO: requires accounts[..] are all valid ptrs

    //-Helpers-----------------------------------------------------------------
    let get_account = |account_info: &AccountInfo| unsafe {
        (account_info.borrow_data_unchecked().as_ptr() as *const account::Account)
            .read()
    };

    //-Initial State-----------------------------------------------------------
    let initial_state_new_account =  get_account(&accounts[0])
        .account_state()
        .unwrap();

    let minimum_balance = unsafe {
        pinocchio::sysvars::rent::Rent::from_bytes_unchecked(accounts[3].borrow_data_unchecked())
    }.minimum_balance(accounts[0].data_len());

    let is_native_mint = accounts[1].key() == &spl_token_interface::native_mint::ID;

    let mint_is_initialised = unsafe {
        (accounts[1].borrow_data_unchecked().as_ptr() as *const spl_token_interface::state::mint::Mint)
            .read()
            .is_initialized()
    };

    //-Process Instruction-----------------------------------------------------
    let result = process_initialize_account(accounts);

    //-Assert Postconditions---------------------------------------------------
    if accounts.len() < 4 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys));
    } else if accounts[3].key() != &pinocchio::sysvars::rent::RENT_ID {
        assert_eq!(result, Err(ProgramError::InvalidArgument))
    } else if accounts[0].data_len() != account::Account::LEN {
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if initial_state_new_account != account_state::AccountState::Uninitialized  { // Untested
        assert_eq!(result, Err(ProgramError::Custom(6)))
    } else if accounts[0].lamports() < minimum_balance {
        assert_eq!(result, Err(ProgramError::Custom(0)))
    } else if !is_native_mint && unsafe {accounts[1].owner()} != &spl_token_interface::program::ID {
        assert_eq!(result, Err(ProgramError::IncorrectProgramId))
    } else if !is_native_mint
            && unsafe {accounts[1].owner()} == &spl_token_interface::program::ID
            && !mint_is_initialised.unwrap() {
        assert_eq!(result, Err(ProgramError::Custom(2)))
    } else {
        assert!(result.is_ok());
        assert_eq!(get_account(&accounts[0]).account_state().unwrap(), account_state::AccountState::Initialized);
        assert_eq!(get_account(&accounts[0]).mint, *accounts[1].key());
        assert_eq!(get_account(&accounts[0]).owner, *accounts[2].key());

        if is_native_mint {
            assert!(get_account(&accounts[0]).is_native());
            assert_eq!(get_account(&accounts[0]).native_amount().unwrap(), minimum_balance);
            assert_eq!(get_account(&accounts[0]).amount(), accounts[0].lamports() - minimum_balance);
        }
    }

    result
}

#[inline(never)]
pub fn test_process_transfer(accounts: &[AccountInfo; 4], instruction_data: &[u8; 8]) -> ProgramResult {
    process_transfer(accounts, instruction_data)
}

#[inline(never)]
pub fn test_process_mint_to(accounts: &[AccountInfo; 3], instruction_data: &[u8; 8]) -> ProgramResult {
    process_mint_to(accounts, instruction_data)
}

#[inline(never)]
pub fn test_process_burn(accounts: &[AccountInfo; 3], instruction_data: &[u8; 8]) -> ProgramResult {
    process_burn(accounts, instruction_data)
}

#[inline(never)]
pub fn test_process_close_account(accounts: &[AccountInfo; 3]) -> ProgramResult {
    process_close_account(accounts)
}

#[inline(never)]
pub fn test_process_transfer_checked(accounts: &[AccountInfo; 4], instruction_data: &[u8; 9]) -> ProgramResult {
    process_transfer_checked(accounts, instruction_data)
}

#[inline(never)]
pub fn test_process_burn_checked(accounts: &[AccountInfo; 3], instruction_data: &[u8; 9]) -> ProgramResult {
    process_burn_checked(accounts, instruction_data)
}

#[inline(never)]
pub fn test_process_initialize_account2(accounts: &[AccountInfo; 2], instruction_data: &[u8; 32]) -> ProgramResult {
    process_initialize_account2(accounts, instruction_data)
}

#[inline(never)]
pub fn test_process_initialize_account3(accounts: &[AccountInfo; 2], instruction_data: &[u8; 32]) -> ProgramResult {
    process_initialize_account3(accounts, instruction_data)
}

#[inline(never)]
pub fn test_process_initialize_mint2(accounts: &[AccountInfo; 2], instruction_data: &[u8; 34]) -> ProgramResult {
    process_initialize_mint2(accounts, instruction_data)
}