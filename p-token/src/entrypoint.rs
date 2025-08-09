use {
    crate::processor::*,
    pinocchio::{
        account_info::AccountInfo,
        no_allocator, nostd_panic_handler, program_entrypoint,
        program_error::{ProgramError, ToStr},
        pubkey::Pubkey,
        ProgramResult,
        sysvars::Sysvar,
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

            match instruction_data.len() {
                x if 66 <= x => test_process_initialize_mint_freeze(accounts.first_chunk().unwrap(), instruction_data.first_chunk().unwrap()),
                x if 34 <= x => test_process_initialize_mint_no_freeze(accounts.first_chunk().unwrap(), instruction_data.first_chunk().unwrap()),
                _ => panic!("Invalid instruction data length"),
            }
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

            test_process_transfer(&accounts.first_chunk().unwrap(), &instruction_data.first_chunk().unwrap())
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

            test_process_mint_to(&accounts.first_chunk().unwrap(), &instruction_data.first_chunk().unwrap())
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

            test_process_transfer_checked(&accounts.first_chunk().unwrap(), &instruction_data.first_chunk().unwrap())
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

            test_process_initialize_account2(&accounts.first_chunk().unwrap(), &instruction_data.first_chunk().unwrap())
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

            test_process_initialize_account3(&accounts.first_chunk().unwrap(), &instruction_data.first_chunk().unwrap())
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

            match instruction_data.len() {
                x if 66 <= x => test_process_initialize_mint2_freeze(accounts.first_chunk().unwrap(), instruction_data.first_chunk().unwrap()),
                x if 34 <= x => test_process_initialize_mint2_no_freeze(accounts.first_chunk().unwrap(), instruction_data.first_chunk().unwrap()),
                _ => panic!("Invalid instruction data length"),
            }
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
        // 102 - InitializeMultisig
        102 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeMultisig");

            test_process_initialize_multisig(accounts.first_chunk().unwrap(), instruction_data.first_chunk().unwrap())
        }
        // 4 - Approve
        4 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: Approve");

            process_approve(accounts, instruction_data)
        }
        // 104 - Approve
        104 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: Approve");

            test_process_approve(accounts.first_chunk().unwrap(), instruction_data.first_chunk().unwrap())
        }
        // 5 - Revoke
        5 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: Revoke");

            process_revoke(accounts)
        }
        // 105 - Revoke
        105 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: Revoke");

            test_process_revoke(accounts.first_chunk().unwrap())
        }
        // 6 - SetAuthority
        6 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: SetAuthority");

            process_set_authority(accounts, instruction_data)
        }
        // 106 - SetAuthority
        106 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: SetAuthority");

            test_process_set_authority(accounts.first_chunk().unwrap(), instruction_data.first_chunk().unwrap())
        }
        // 10 - FreezeAccount
        10 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: FreezeAccount");

            process_freeze_account(accounts)
        }
        // 110 - FreezeAccount
        110 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: FreezeAccount");

            test_process_freeze_account(accounts.first_chunk().unwrap())
        }
        // 11 - ThawAccount
        11 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: ThawAccount");

            process_thaw_account(accounts)
        }
        // 111 - ThawAccount
        111 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: ThawAccount");

            test_process_thaw_account(accounts.first_chunk().unwrap())
        }
        // 13 - ApproveChecked
        13 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: ApproveChecked");

            process_approve_checked(accounts, instruction_data)
        }
        // 113 - ApproveChecked
        113 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: ApproveChecked");

            test_process_approve_checked(accounts.first_chunk().unwrap(), instruction_data.first_chunk().unwrap())
        }
        // 14 - MintToChecked
        14 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: MintToChecked");

            process_mint_to_checked(accounts, instruction_data)
        }
        // 114 - MintToChecked
        114 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: MintToChecked");

            test_process_mint_to_checked(accounts.first_chunk().unwrap(), instruction_data.first_chunk().unwrap())
        }
        // 17 - SyncNative
        17 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: SyncNative");

            process_sync_native(accounts)
        }
        // 117 - SyncNative
        117 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: SyncNative");

            test_process_sync_native(accounts.first_chunk().unwrap())
        }
        // 19 - InitializeMultisig2
        19 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeMultisig2");

            process_initialize_multisig2(accounts, instruction_data)
        }
        // 119 - InitializeMultisig2
        119 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeMultisig2");

            test_process_initialize_multisig2(accounts.first_chunk().unwrap(), instruction_data.first_chunk().unwrap())
        }
        // 21 - GetAccountDataSize
        21 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: GetAccountDataSize");

            process_get_account_data_size(accounts)
        }
        // 121 - GetAccountDataSize
        121 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: GetAccountDataSize");

            test_process_get_account_data_size(accounts.first_chunk().unwrap())
        }
        // 22 - InitializeImmutableOwner
        22 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeImmutableOwner");

            process_initialize_immutable_owner(accounts)
        }
        // 122 - InitializeImmutableOwner
        122 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeImmutableOwner");

            test_process_initialize_immutable_owner(accounts.first_chunk().unwrap())
        }
        // 23 - AmountToUiAmount
        23 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: AmountToUiAmount");

            process_amount_to_ui_amount(accounts, instruction_data)
        }
        // 123 - AmountToUiAmount
        123 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: AmountToUiAmount");

            test_process_amount_to_ui_amount(accounts.first_chunk().unwrap(), instruction_data.first_chunk().unwrap())
        }
        // 24 - UiAmountToAmount
        24 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: UiAmountToAmount");

            process_ui_amount_to_amount(accounts, instruction_data)
        }
        // 124 - UiAmountToAmount
        124 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: UiAmountToAmount");

            test_process_ui_amount_to_amount(accounts.first_chunk().unwrap(), instruction_data.first_chunk().unwrap())
        }
        // 38 - WithdrawExcessLamports
        38 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: WithdrawExcessLamports");

            process_withdraw_excess_lamports(accounts)
        }
        // 138 - WithdrawExcessLamports
        138 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: WithdrawExcessLamports");

            test_process_withdraw_excess_lamports(accounts.first_chunk().unwrap())
        }
        _ => Err(TokenError::InvalidInstruction.into()),
    }
}

// Hack Tests For Stable MIR JSON ---------------------------------------------
/// accounts[0] // Mint Info
/// accounts[1] // Rent Sysvar Info
/// instruction_data[0]      // Decimals
/// instruction_data[1..33]  // Mint Authority Pubkey
/// instruction_data[33]     // Freeze Authority Exists? 1 for freeze
/// instruction_data[34..66] // instruction_data[33] == 1 ==> Freeze Authority Pubkey
#[inline(never)]
pub fn test_process_initialize_mint_freeze(accounts: &[AccountInfo; 2], instruction_data: &[u8; 66]) -> ProgramResult {
    use spl_token_interface::state::mint::Mint;
    //-Helpers-----------------------------------------------------------------
    let get_mint = |account_info: &AccountInfo| unsafe {
        (account_info.borrow_data_unchecked().as_ptr() as *const Mint)
            .read()
    };

    //-Initial State-----------------------------------------------------------
    let minimum_balance = unsafe {
        pinocchio::sysvars::rent::Rent::from_bytes_unchecked(accounts[1].borrow_data_unchecked())
    }.minimum_balance(accounts[0].data_len());
    let mint_is_initialised_prior = get_mint(&accounts[0]).is_initialized();

    //-Process Instruction-----------------------------------------------------
    let result = process_initialize_mint(accounts, instruction_data);

    //-Assert Postconditions---------------------------------------------------
    if instruction_data.len() < 34 {
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if instruction_data[33] != 0 && instruction_data[33] != 1 {
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if instruction_data[33] == 1 && instruction_data.len() < 66 {
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if accounts.len() < 2 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys))
    } else if accounts[0].data_len() != Mint::LEN {
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if mint_is_initialised_prior.unwrap()  { // Untested
        assert_eq!(result, Err(ProgramError::Custom(6)))
    } else if accounts[0].lamports() < minimum_balance {
        assert_eq!(result, Err(ProgramError::Custom(0)))
    } else {
        assert!(get_mint(&accounts[0]).is_initialized().unwrap());
        assert_eq!(get_mint(&accounts[0]).mint_authority().unwrap(), &instruction_data[1..33]);
        assert_eq!(get_mint(&accounts[0]).decimals, instruction_data[0]);

        if instruction_data[33] == 1 {
            assert_eq!(get_mint(&accounts[0]).freeze_authority().unwrap(), &instruction_data[34..66]);
        }
    }

    result
}

/// accounts[0] // Mint Info
/// accounts[1] // Rent Sysvar Info
/// instruction_data[0]      // Decimals
/// instruction_data[1..33]  // Mint Authority Pubkey
/// instruction_data[33]     // Freeze Authority Exists? 0 for no freeze
#[inline(never)]
pub fn test_process_initialize_mint_no_freeze(accounts: &[AccountInfo; 2], instruction_data: &[u8; 34]) -> ProgramResult {
    use spl_token_interface::state::mint::Mint;
    //-Helpers-----------------------------------------------------------------
    let get_mint = |account_info: &AccountInfo| unsafe {
        (account_info.borrow_data_unchecked().as_ptr() as *const Mint)
            .read()
    };

    //-Initial State-----------------------------------------------------------
    let minimum_balance = unsafe {
        pinocchio::sysvars::rent::Rent::from_bytes_unchecked(accounts[1].borrow_data_unchecked())
    }.minimum_balance(accounts[0].data_len());
    let mint_is_initialised_prior = get_mint(&accounts[0]).is_initialized();

    //-Process Instruction-----------------------------------------------------
    let result = process_initialize_mint(accounts, instruction_data);

    //-Assert Postconditions---------------------------------------------------
    if instruction_data.len() < 34 {
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if instruction_data[33] != 0 && instruction_data[33] != 1 {
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if instruction_data[33] == 1 && instruction_data.len() < 66 {
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if accounts.len() < 2 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys))
    } else if accounts[0].data_len() != Mint::LEN {
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if mint_is_initialised_prior.unwrap()  { // Untested
        assert_eq!(result, Err(ProgramError::Custom(6)))
    } else if accounts[0].lamports() < minimum_balance {
        assert_eq!(result, Err(ProgramError::Custom(0)))
    } else {
        assert!(get_mint(&accounts[0]).is_initialized().unwrap());
        assert_eq!(get_mint(&accounts[0]).mint_authority().unwrap(), &instruction_data[1..33]);
        assert_eq!(get_mint(&accounts[0]).decimals, instruction_data[0]);

        if instruction_data[33] == 1 {
            assert_eq!(get_mint(&accounts[0]).freeze_authority().unwrap(), &instruction_data[34..66]);
        }
    }

    result
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

/// accounts[0] // Source Info
/// accounts[1] // Destination Info
/// accounts[2] // Authority Info
/// instruction_data[0..8] // Little Endian Bytes of u64 amount
#[inline(never)]
pub fn test_process_transfer(accounts: &[AccountInfo; 3], instruction_data: &[u8; 8]) -> ProgramResult {
    use spl_token_interface::state::{account, account_state};

    // TODO: requires accounts[..] are all valid ptrs

    //-Helpers-----------------------------------------------------------------
    let get_account = |account_info: &AccountInfo| unsafe {
        (account_info.borrow_data_unchecked().as_ptr() as *const account::Account)
            .read()
    };

    //-Initial State-----------------------------------------------------------
    let amount = unsafe { u64::from_le_bytes(*(instruction_data.as_ptr() as *const [u8; 8])) };
    let src_initialised = get_account(&accounts[0]).is_initialized();
    let dst_initialised = get_account(&accounts[1]).is_initialized();
    let src_initial_amount = get_account(&accounts[0]).amount();
    let dst_initial_amount = get_account(&accounts[1]).amount();
    let src_initial_lamports = accounts[0].lamports();
    let dst_initial_lamports = accounts[1].lamports();

    //-Process Instruction-----------------------------------------------------
    let result = process_transfer(accounts, instruction_data);

    //-Assert Postconditions---------------------------------------------------
    if instruction_data.len() < 8 {
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if accounts.len() < 3 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys))
    } else if !src_initialised.unwrap() {
        // I don't believe there is a way to construct an Account with AccountState::Uninitialized
        // so it can only be an invalid account
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if accounts[0] != accounts[1] && !dst_initialised.unwrap() {
        // I don't believe there is a way to construct an Account with AccountState::Uninitialized
        // so it can only be an invalid account
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if get_account(&accounts[0]).account_state().unwrap() == account_state::AccountState::Frozen {
        assert_eq!(result, Err(ProgramError::Custom(17)))
    } else if src_initial_amount < amount {
        assert_eq!(result, Err(ProgramError::Custom(1)))
    } else if accounts[0] != accounts[1] && get_account(&accounts[0]).mint != get_account(&accounts[1]).mint {
        assert_eq!(result, Err(ProgramError::Custom(3)))
    } else {
        if get_account(&accounts[0]).delegate() == Some(accounts[2].key()) {
            // TODO validate_owner and delgated_amount
        } else {
            // TODO validate_owner
        }

        if (accounts[0] == accounts[1] || amount == 0) && unsafe { accounts[0].owner() } != &spl_token_interface::program::ID {
            assert_eq!(result, Err(ProgramError::IncorrectProgramId)) // UNTESTED
        } else if (accounts[0] == accounts[1] || amount == 0) && unsafe { accounts[1].owner() } != &spl_token_interface::program::ID {
            assert_eq!(result, Err(ProgramError::IncorrectProgramId)) // UNTESTED
        } else if accounts[0] != accounts[1] && amount != 0 && get_account(&accounts[0]).is_native() && src_initial_lamports < amount {
            // Not sure how to fund native mint
            assert_eq!(result, Err(ProgramError::Custom(14))) // UNTESTED
        } else if accounts[0] != accounts[1] && amount != 0 && get_account(&accounts[0]).is_native() && u64::MAX - amount < dst_initial_lamports {
            // Not sure how to fund native mint
            assert_eq!(result, Err(ProgramError::Custom(14))) // UNTESTED
        }  else {
            assert!(result.is_ok());
            assert_eq!(get_account(&accounts[0]).amount(), src_initial_amount - amount);
            assert_eq!(get_account(&accounts[1]).amount(), dst_initial_amount + amount);

            if get_account(&accounts[0]).is_native() {
                // UNTESTED Not sure how to fund native mint
                assert_eq!(accounts[0].lamports(), src_initial_lamports + amount);
                assert_eq!(accounts[1].lamports(), src_initial_lamports - amount);
            }
        }
    }

    result
}

/// accounts[0] // Mint Info
/// accounts[1] // Destination Info
/// accounts[2] // Owner Info
/// instruction_data[0..8] // Little Endian Bytes of u64 amount
#[inline(never)]
pub fn test_process_mint_to(accounts: &[AccountInfo; 3], instruction_data: &[u8; 8]) -> ProgramResult {
    use spl_token_interface::state::{mint, account, account_state};

    // TODO: requires accounts[..] are all valid ptrs

    //-Helpers-----------------------------------------------------------------
    let get_account = |account_info: &AccountInfo| unsafe {
        (account_info.borrow_data_unchecked().as_ptr() as *const account::Account)
            .read()
    };
    let get_mint = |account_info: &AccountInfo| unsafe {
        (account_info.borrow_data_unchecked().as_ptr() as *const mint::Mint)
            .read()
    };

    //-Initial State-----------------------------------------------------------
    let initial_supply = get_mint(&accounts[0]).supply();
    let initial_amount = get_account(&accounts[1]).amount();

    //-Process Instruction-----------------------------------------------------
    let result = process_mint_to(accounts, instruction_data);

    //-Assert Postconditions---------------------------------------------------
    if instruction_data.len() < 8 {
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if accounts.len() < 3 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys))
    } else if accounts[1].data_len() != account::Account::LEN {
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if !get_account(&accounts[1]).is_initialized().unwrap() { // UNTESTED
        assert_eq!(result, Err(ProgramError::UninitializedAccount))
    } else if get_account(&accounts[1]).account_state().unwrap() == account_state::AccountState::Frozen  {
        assert_eq!(result, Err(ProgramError::Custom(17)))
    } else if get_account(&accounts[1]).is_native() {
        assert_eq!(result, Err(ProgramError::Custom(10)))
    } else if accounts[0].key() != &get_account(&accounts[1]).mint {
        assert_eq!(result, Err(ProgramError::Custom(3)))
    } else if accounts[0].data_len() != mint::Mint::LEN { // UNTESTED
        // Not sure if this is even possible if we get past the case above
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if !get_mint(&accounts[0]).is_initialized().unwrap() { // UNTESTED
        assert_eq!(result, Err(ProgramError::UninitializedAccount))
    } else {
        if get_mint(&accounts[0]).mint_authority().is_some() { // UNTESTED
            // TODO validate_owner
        } else { // UNTESTED
            assert_eq!(result, Err(ProgramError::Custom(5)))
        }

        let amount =  unsafe { u64::from_le_bytes(*(instruction_data.as_ptr() as *const [u8; 8])) };

        if amount == 0 && unsafe { accounts[0].owner() } != &spl_token_interface::program::ID {
            assert_eq!(result, Err(ProgramError::IncorrectProgramId)) // UNTESTED
        } else if amount == 0 && unsafe { accounts[1].owner() } != &spl_token_interface::program::ID {
            assert_eq!(result, Err(ProgramError::IncorrectProgramId)) // UNTESTED
        } else if amount != 0 && u64::MAX - amount < initial_supply {
            assert_eq!(result, Err(ProgramError::Custom(14)))
        } else {
            assert_eq!(get_mint(&accounts[0]).supply(), initial_supply + amount);
            assert_eq!(get_account(&accounts[1]).amount(), initial_amount + amount);
            assert!(result.is_ok());
        }
    }

    result
}

#[inline(never)]
pub fn test_process_burn(accounts: &[AccountInfo; 3], instruction_data: &[u8; 8]) -> ProgramResult {
    process_burn(accounts, instruction_data)
}

#[inline(never)]
pub fn test_process_close_account(accounts: &[AccountInfo; 3]) -> ProgramResult {
    process_close_account(accounts)
}

/// accounts[0] // Source Info
/// accounts[1] // Mint Info
/// accounts[2] // Destination Info
/// accounts[3] // Authority Info
/// instruction_data[0..9] // Little Endian Bytes of u64 amount, and decimals
#[inline(never)]
pub fn test_process_transfer_checked(accounts: &[AccountInfo; 4], instruction_data: &[u8; 9]) -> ProgramResult {
    use spl_token_interface::state::{account, account_state, mint::Mint};

    // TODO: requires accounts[..] are all valid ptrs

    //-Helpers-----------------------------------------------------------------
    let get_account = |account_info: &AccountInfo| unsafe {
        (account_info.borrow_data_unchecked().as_ptr() as *const account::Account)
            .read()
    };
    let get_mint = |account_info: &AccountInfo| unsafe {
        (account_info.borrow_data_unchecked().as_ptr() as *const Mint)
            .read()
    };

    //-Initial State-----------------------------------------------------------
    let amount = unsafe { u64::from_le_bytes(*(instruction_data.as_ptr() as *const [u8; 8])) };
    let src_initialised = get_account(&accounts[0]).is_initialized();
    let dst_initialised = get_account(&accounts[2]).is_initialized();
    let src_initial_amount = get_account(&accounts[0]).amount();
    let dst_initial_amount = get_account(&accounts[2]).amount();
    let src_initial_lamports = accounts[0].lamports();
    let dst_initial_lamports = accounts[2].lamports();

    //-Process Instruction-----------------------------------------------------
    let result = process_transfer_checked(accounts, instruction_data);

    //-Assert Postconditions---------------------------------------------------
    if instruction_data.len() < 9 {
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if accounts.len() < 4 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys))
    } else if !src_initialised.unwrap() {
        // I don't believe there is a way to construct an Account with AccountState::Uninitialized
        // so it can only be an invalid account
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if accounts[0] != accounts[2] && !dst_initialised.unwrap() {
        // I don't believe there is a way to construct an Account with AccountState::Uninitialized
        // so it can only be an invalid account
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if get_account(&accounts[0]).account_state().unwrap() == account_state::AccountState::Frozen {
        assert_eq!(result, Err(ProgramError::Custom(17)))
    } else if src_initial_amount < amount {
        assert_eq!(result, Err(ProgramError::Custom(1)))
    } else if accounts[0] != accounts[2] && get_account(&accounts[0]).mint != get_account(&accounts[2]).mint {
        assert_eq!(result, Err(ProgramError::Custom(3)))
    } else if accounts[1].key() != &get_account(&accounts[0]).mint {
        assert_eq!(result, Err(ProgramError::Custom(3)))
    } else if accounts[1].data_len() != core::mem::size_of::<Mint>() {
        assert_eq!(result, Err(ProgramError::InvalidAccountData)) // UNTESTED
    } else if !get_mint(&accounts[1]).is_initialized().unwrap() {
        assert_eq!(result, Err(ProgramError::UninitializedAccount)) // UNTESTED
    } else if instruction_data[8] != get_mint(&accounts[1]).decimals {
        assert_eq!(result, Err(ProgramError::Custom(18)))
    } else {
        if get_account(&accounts[0]).delegate() == Some(accounts[3].key()) {
            // TODO validate_owner and delgated_amount
        } else {
            // TODO validate_owner
        }

        if (accounts[0] == accounts[2] || amount == 0) && unsafe { accounts[0].owner() } != &spl_token_interface::program::ID {
            assert_eq!(result, Err(ProgramError::IncorrectProgramId)) // UNTESTED
        } else if (accounts[0] == accounts[2] || amount == 0) && unsafe { accounts[2].owner() } != &spl_token_interface::program::ID {
            assert_eq!(result, Err(ProgramError::IncorrectProgramId)) // UNTESTED
        } else if get_account(&accounts[0]).is_native() && src_initial_lamports < amount {
            // Not sure how to fund native mint
            assert_eq!(result, Err(ProgramError::Custom(14))) // UNTESTED
        } else if get_account(&accounts[0]).is_native() && u64::MAX - amount < dst_initial_lamports {
            // Not sure how to fund native mint
            assert_eq!(result, Err(ProgramError::Custom(14))) // UNTESTED
        }  else {
            assert!(result.is_ok());
            assert_eq!(get_account(&accounts[0]).amount(), src_initial_amount - amount);
            assert_eq!(get_account(&accounts[2]).amount(), dst_initial_amount + amount);

            if get_account(&accounts[0]).is_native() {
                // UNTESTED Not sure how to fund native mint
                assert_eq!(accounts[0].lamports(), src_initial_lamports + amount);
                assert_eq!(accounts[2].lamports(), src_initial_lamports - amount);
            }
        }
    }

    result
}

#[inline(never)]
pub fn test_process_burn_checked(accounts: &[AccountInfo; 3], instruction_data: &[u8; 9]) -> ProgramResult {
    process_burn_checked(accounts, instruction_data)
}

/// accounts[0] // New Account Info
/// accounts[1] // Mint Info
/// accounts[2] // Rent Sysvar Info
/// instruction_data[..] // Owner
#[inline(never)]
pub fn test_process_initialize_account2(accounts: &[AccountInfo; 3], instruction_data: &[u8; 32]) -> ProgramResult {
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
        pinocchio::sysvars::rent::Rent::from_bytes_unchecked(accounts[2].borrow_data_unchecked())
    }.minimum_balance(accounts[0].data_len());

    let is_native_mint = accounts[1].key() == &spl_token_interface::native_mint::ID;

    let mint_is_initialised = unsafe {
        (accounts[1].borrow_data_unchecked().as_ptr() as *const spl_token_interface::state::mint::Mint)
            .read()
            .is_initialized()
    };

    //-Process Instruction-----------------------------------------------------
    let result = process_initialize_account2(accounts, instruction_data);

    //-Assert Postconditions---------------------------------------------------
    if instruction_data.len() < pinocchio::pubkey::PUBKEY_BYTES {
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if accounts.len() < 3 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys));
    } else if accounts[2].key() != &pinocchio::sysvars::rent::RENT_ID {
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
        assert_eq!(get_account(&accounts[0]).owner, *instruction_data);

        if is_native_mint {
            assert!(get_account(&accounts[0]).is_native());
            assert_eq!(get_account(&accounts[0]).native_amount().unwrap(), minimum_balance);
            assert_eq!(get_account(&accounts[0]).amount(), accounts[0].lamports() - minimum_balance);
        }
    }

    result
}

/// accounts[0] // New Account Info
/// accounts[1] // Mint Info
/// instruction_data[..] // Owner
#[inline(never)]
pub fn test_process_initialize_account3(accounts: &[AccountInfo; 2], instruction_data: &[u8; 32]) -> ProgramResult {
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

    // Note: Rent is a supported sysvar so ProgramError::UnsupportedSysvar should be impossible
    let rent = pinocchio::sysvars::rent::Rent::get().unwrap();
    let minimum_balance = rent.minimum_balance(accounts[0].data_len());

    let is_native_mint = accounts[1].key() == &spl_token_interface::native_mint::ID;

    let mint_is_initialised = unsafe {
        (accounts[1].borrow_data_unchecked().as_ptr() as *const spl_token_interface::state::mint::Mint)
            .read()
            .is_initialized()
    };

    //-Process Instruction-----------------------------------------------------
    let result = process_initialize_account3(accounts, instruction_data);

    //-Assert Postconditions---------------------------------------------------
    if instruction_data.len() < pinocchio::pubkey::PUBKEY_BYTES {
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if accounts.len() < 2 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys));
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
        assert_eq!(get_account(&accounts[0]).owner, *instruction_data);

        if is_native_mint {
            assert!(get_account(&accounts[0]).is_native());
            assert_eq!(get_account(&accounts[0]).native_amount().unwrap(), minimum_balance);
            assert_eq!(get_account(&accounts[0]).amount(), accounts[0].lamports() - minimum_balance);
        }
    }

    result
}

/// accounts[0] // Mint Info
/// instruction_data[0]      // Decimals
/// instruction_data[1..33]  // Mint Authority Pubkey
/// instruction_data[33]     // Freeze Authority Exists? 1 for freeze
/// instruction_data[34..66] // instruction_data[33] == 1 ==> Freeze Authority Pubkey
#[inline(never)]
pub fn test_process_initialize_mint2_freeze(accounts: &[AccountInfo; 1], instruction_data: &[u8; 66]) -> ProgramResult {
    use spl_token_interface::state::mint::Mint;
    //-Helpers-----------------------------------------------------------------
    let get_mint = |account_info: &AccountInfo| unsafe {
        (account_info.borrow_data_unchecked().as_ptr() as *const Mint)
            .read()
    };

    //-Initial State-----------------------------------------------------------
    // Note: Rent is a supported sysvar so ProgramError::UnsupportedSysvar should be impossible
    let rent = pinocchio::sysvars::rent::Rent::get().unwrap();
    let minimum_balance = rent.minimum_balance(accounts[0].data_len());
    let mint_is_initialised_prior = get_mint(&accounts[0]).is_initialized();

    //-Process Instruction-----------------------------------------------------
    let result = process_initialize_mint2(accounts, instruction_data);

    //-Assert Postconditions---------------------------------------------------
    if instruction_data.len() < 34 {
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if instruction_data[33] != 0 && instruction_data[33] != 1 {
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if instruction_data[33] == 1 && instruction_data.len() < 66 {
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if accounts.len() < 1 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys))
    } else if accounts[0].data_len() != Mint::LEN {
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if mint_is_initialised_prior.unwrap()  { // Untested
        assert_eq!(result, Err(ProgramError::Custom(6)))
    } else if accounts[0].lamports() < minimum_balance {
        assert_eq!(result, Err(ProgramError::Custom(0)))
    } else {
        assert!(get_mint(&accounts[0]).is_initialized().unwrap());
        assert_eq!(get_mint(&accounts[0]).mint_authority().unwrap(), &instruction_data[1..33]);
        assert_eq!(get_mint(&accounts[0]).decimals, instruction_data[0]);

        if instruction_data[33] == 1 {
            assert_eq!(get_mint(&accounts[0]).freeze_authority().unwrap(), &instruction_data[34..66]);
        }
    }

    result
}

/// accounts[0] // Mint Info
/// instruction_data[0]      // Decimals
/// instruction_data[1..33]  // Mint Authority Pubkey
/// instruction_data[33]     // Freeze Authority Exists? 0 for no freeze
#[inline(never)]
pub fn test_process_initialize_mint2_no_freeze(accounts: &[AccountInfo; 1], instruction_data: &[u8; 34]) -> ProgramResult {
    use spl_token_interface::state::mint::Mint;
    //-Helpers-----------------------------------------------------------------
    let get_mint = |account_info: &AccountInfo| unsafe {
        (account_info.borrow_data_unchecked().as_ptr() as *const Mint)
            .read()
    };

    //-Initial State-----------------------------------------------------------
    // Note: Rent is a supported sysvar so ProgramError::UnsupportedSysvar should be impossible
    let rent = pinocchio::sysvars::rent::Rent::get().unwrap();
    let minimum_balance = rent.minimum_balance(accounts[0].data_len());
    let mint_is_initialised_prior = get_mint(&accounts[0]).is_initialized();

    //-Process Instruction-----------------------------------------------------
    let result = process_initialize_mint2(accounts, instruction_data);

    //-Assert Postconditions---------------------------------------------------
    if instruction_data.len() < 34 {
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if instruction_data[33] != 0 && instruction_data[33] != 1 {
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if instruction_data[33] == 1 && instruction_data.len() < 66 {
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if accounts.len() < 1 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys))
    } else if accounts[0].data_len() != Mint::LEN {
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if mint_is_initialised_prior.unwrap()  { // Untested
        assert_eq!(result, Err(ProgramError::Custom(6)))
    } else if accounts[0].lamports() < minimum_balance {
        assert_eq!(result, Err(ProgramError::Custom(0)))
    } else {
        assert!(get_mint(&accounts[0]).is_initialized().unwrap());
        assert_eq!(get_mint(&accounts[0]).mint_authority().unwrap(), &instruction_data[1..33]);
        assert_eq!(get_mint(&accounts[0]).decimals, instruction_data[0]);

        if instruction_data[33] == 1 {
            assert_eq!(get_mint(&accounts[0]).freeze_authority().unwrap(), &instruction_data[34..66]);
        }
    }

    result
}

fn test_process_initialize_multisig(accounts: &[AccountInfo; 4], instruction_data: &[u8; 1]) -> ProgramResult {
    process_initialize_multisig(accounts, instruction_data)
}

fn test_process_approve(accounts: &[AccountInfo; 4], instruction_data: &[u8; 1]) -> ProgramResult {
    process_approve(accounts, instruction_data)
}

fn test_process_revoke(accounts: &[AccountInfo; 4]) -> ProgramResult {
    process_revoke(accounts)
}

fn test_process_set_authority(accounts: &[AccountInfo; 4], instruction_data: &[u8; 1]) -> ProgramResult {
    process_set_authority(accounts, instruction_data)
}

fn test_process_freeze_account(accounts: &[AccountInfo; 4]) -> ProgramResult {
    process_freeze_account(accounts)
}

fn test_process_thaw_account(accounts: &[AccountInfo; 4]) -> ProgramResult {
    process_thaw_account(accounts)
}

fn test_process_approve_checked(accounts: &[AccountInfo; 4], instruction_data: &[u8; 1]) -> ProgramResult {
    process_approve_checked(accounts, instruction_data)
}

/// accounts[0] // Mint Info
/// accounts[1] // Destination Info
/// accounts[2] // Owner Info
/// instruction_data[0..9] // Little Endian Bytes of u64 amount, and decimals
fn test_process_mint_to_checked(accounts: &[AccountInfo; 3], instruction_data: &[u8; 9]) -> ProgramResult {
    use spl_token_interface::state::{mint, account, account_state};

    // TODO: requires accounts[..] are all valid ptrs

    //-Helpers-----------------------------------------------------------------
    let get_account = |account_info: &AccountInfo| unsafe {
        (account_info.borrow_data_unchecked().as_ptr() as *const account::Account)
            .read()
    };
    let get_mint = |account_info: &AccountInfo| unsafe {
        (account_info.borrow_data_unchecked().as_ptr() as *const mint::Mint)
            .read()
    };

    //-Initial State-----------------------------------------------------------
    let initial_supply = get_mint(&accounts[0]).supply();
    let initial_amount = get_account(&accounts[1]).amount();

    //-Process Instruction-----------------------------------------------------
    let result = process_mint_to_checked(accounts, instruction_data);

    //-Assert Postconditions---------------------------------------------------
    if instruction_data.len() < 9 {
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if accounts.len() < 3 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys))
    } else if accounts[1].data_len() != account::Account::LEN {
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if !get_account(&accounts[1]).is_initialized().unwrap() { // UNTESTED
        assert_eq!(result, Err(ProgramError::UninitializedAccount))
    } else if get_account(&accounts[1]).account_state().unwrap() == account_state::AccountState::Frozen  {
        assert_eq!(result, Err(ProgramError::Custom(17)))
    } else if get_account(&accounts[1]).is_native() {
        assert_eq!(result, Err(ProgramError::Custom(10)))
    } else if accounts[0].key() != &get_account(&accounts[1]).mint {
        assert_eq!(result, Err(ProgramError::Custom(3)))
    } else if accounts[0].data_len() != mint::Mint::LEN { // UNTESTED
        // Not sure if this is even possible if we get past the case above
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if !get_mint(&accounts[0]).is_initialized().unwrap() { // UNTESTED
        assert_eq!(result, Err(ProgramError::UninitializedAccount))
    } else if instruction_data[8] != get_mint(&accounts[0]).decimals { // UNTESTED
        assert_eq!(result, Err(ProgramError::Custom(18)))
    } else {
        if get_mint(&accounts[0]).mint_authority().is_some() { // UNTESTED
            // TODO validate_owner
        } else { // UNTESTED
            assert_eq!(result, Err(ProgramError::Custom(5)))
        }

        let amount =  unsafe { u64::from_le_bytes(*(instruction_data.as_ptr() as *const [u8; 8])) };

        if amount == 0 && unsafe { accounts[0].owner() } != &spl_token_interface::program::ID {
            assert_eq!(result, Err(ProgramError::IncorrectProgramId)) // UNTESTED
        } else if amount == 0 && unsafe { accounts[1].owner() } != &spl_token_interface::program::ID {
            assert_eq!(result, Err(ProgramError::IncorrectProgramId)) // UNTESTED
        } else if amount != 0 && u64::MAX - amount < initial_supply {
            assert_eq!(result, Err(ProgramError::Custom(14)))
        } else {
            assert_eq!(get_mint(&accounts[0]).supply(), initial_supply + amount);
            assert_eq!(get_account(&accounts[1]).amount(), initial_amount + amount);
            assert!(result.is_ok());
        }
    }

    result
}

fn test_process_sync_native(accounts: &[AccountInfo; 4]) -> ProgramResult {
    process_sync_native(accounts)
}

fn test_process_initialize_multisig2(accounts: &[AccountInfo; 4], instruction_data: &[u8; 1]) -> ProgramResult {
    process_initialize_multisig2(accounts, instruction_data)
}

/// accounts[0] // Mint Info
fn test_process_get_account_data_size(accounts: &[AccountInfo; 1]) -> ProgramResult {
    use spl_token_interface::state::mint;

    // TODO: requires accounts[..] are all valid ptrs

    //-Helpers-----------------------------------------------------------------

    //-Initial State-----------------------------------------------------------
    let get_mint = |account_info: &AccountInfo| unsafe {
        (account_info.borrow_data_unchecked().as_ptr() as *const mint::Mint)
            .read()
    };

    //-Process Instruction-----------------------------------------------------
    let result = process_get_account_data_size(accounts);

    //-Assert Postconditions---------------------------------------------------
    if accounts.len() < 1 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys))
    } else if unsafe { accounts[0].owner() } != &spl_token_interface::program::ID { // UNTESTED
        assert_eq!(result, Err(ProgramError::IncorrectProgramId))
    } else if accounts[0].data_len() != mint::Mint::LEN {
        assert_eq!(result, Err(ProgramError::Custom(2)))
    } else if !get_mint(&accounts[0]).is_initialized().unwrap() {
        assert_eq!(result, Err(ProgramError::Custom(2)))
    } else {

        // TODO: Figure out how to read return data and use it is Account::LEN
        // NOTE: This uses syscalls::sol_set_return_data
        assert!(result.is_ok())
    }
    result
}

fn test_process_initialize_immutable_owner(accounts: &[AccountInfo; 4]) -> ProgramResult {
    process_initialize_immutable_owner(accounts)
}

fn test_process_amount_to_ui_amount(accounts: &[AccountInfo; 4], instruction_data: &[u8; 1]) -> ProgramResult {
    process_amount_to_ui_amount(accounts, instruction_data)
}

fn test_process_ui_amount_to_amount(accounts: &[AccountInfo; 4], instruction_data: &[u8; 1]) -> ProgramResult {
    process_ui_amount_to_amount(accounts, instruction_data)
}

fn test_process_withdraw_excess_lamports(accounts: &[AccountInfo; 4]) -> ProgramResult {
    process_withdraw_excess_lamports(accounts)
}
