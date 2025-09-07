use {
    crate::processor::*,
    pinocchio::{
        account_info::AccountInfo, no_allocator, nostd_panic_handler, program_entrypoint, program_error::{ProgramError, ToStr}, pubkey::Pubkey, sysvars::Sysvar, ProgramResult
    },
    pinocchio_token_interface::{error::TokenError, state::{Initializable, Transmutable}},
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
        // 0 - Test InitializeMint
        0 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: InitializeMint");

            match instruction_data.len() {
                x if 66 <= x => test_process_initialize_mint_freeze(accounts.first_chunk().unwrap(), instruction_data.first_chunk().unwrap()),
                x if 34 <= x => test_process_initialize_mint_no_freeze(accounts.first_chunk().unwrap(), instruction_data.first_chunk().unwrap()),
                _ => panic!("Invalid instruction data length"),
            }
        }
        // 1 - Test InitializeAccount
        1 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: InitializeAccount");

            test_process_initialize_account(accounts.first_chunk().unwrap())
        }
        // 3 - Test Transfer
        3 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: Transfer");

            test_process_transfer(&accounts.first_chunk().unwrap(), &instruction_data.first_chunk().unwrap())
        }
        // 7 - MintTo
        7 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: MintTo");

            test_process_mint_to(&accounts.first_chunk().unwrap(), &instruction_data.first_chunk().unwrap())
        }
        // 8 - Test Burn
        8 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: Burn");

            test_process_burn(&accounts.first_chunk().unwrap(), &instruction_data.first_chunk().unwrap())
        }
        // 9 - Test CloseAccount
        9 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: CloseAccount");

            test_process_close_account(&accounts.first_chunk().unwrap())
        }
        // 12 - Test TransferChecked
        12 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: TransferChecked");

            test_process_transfer_checked(&accounts.first_chunk().unwrap(), &instruction_data.first_chunk().unwrap())
        }
        // 15 - Test BurnChecked
        15 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: BurnChecked");

            test_process_burn_checked(&accounts.first_chunk().unwrap(), &instruction_data.first_chunk().unwrap())
        }
        // 16 - Test InitializeAccount2
        16 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: InitializeAccount2");

            test_process_initialize_account2(&accounts.first_chunk().unwrap(), &instruction_data.first_chunk().unwrap())
        }
        // 18 - Test InitializeAccount3
        18 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Testing Instruction: InitializeAccount3");

            test_process_initialize_account3(&accounts.first_chunk().unwrap(), &instruction_data.first_chunk().unwrap())
        }
        // 20 - Test InitializeMint2
        20 => {
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

            test_process_initialize_multisig(accounts.first_chunk().unwrap(), instruction_data.first_chunk().unwrap())
        }
        // 4 - Approve
        4 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: Approve");

            test_process_approve(accounts.first_chunk().unwrap(), instruction_data.first_chunk().unwrap())
        }
        // 5 - Revoke
        5 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: Revoke");

            test_process_revoke(accounts.first_chunk().unwrap())
        }
        // 6 - SetAuthority
        6 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: SetAuthority");

            test_process_set_authority(accounts.first_chunk().unwrap(), instruction_data)
        }
        // 10 - FreezeAccount
        10 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: FreezeAccount");

            test_process_freeze_account(accounts.first_chunk().unwrap())
        }
        // 11 - ThawAccount
        11 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: ThawAccount");

            test_process_thaw_account(accounts.first_chunk().unwrap())
        }
        // 13 - ApproveChecked
        13 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: ApproveChecked");

            test_process_approve_checked(accounts.first_chunk().unwrap(), instruction_data.first_chunk().unwrap())
        }
        // 14 - MintToChecked
        14 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: MintToChecked");

            test_process_mint_to_checked(accounts.first_chunk().unwrap(), instruction_data.first_chunk().unwrap())
        }
        // 17 - SyncNative
        17 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: SyncNative");

            test_process_sync_native(accounts.first_chunk().unwrap())
        }
        // 19 - InitializeMultisig2
        19 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeMultisig2");

            test_process_initialize_multisig2(accounts.first_chunk().unwrap(), instruction_data.first_chunk().unwrap())
        }
        // 21 - GetAccountDataSize
        21 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: GetAccountDataSize");

            test_process_get_account_data_size(accounts.first_chunk().unwrap())
        }
        // 22 - InitializeImmutableOwner
        22 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: InitializeImmutableOwner");

            test_process_initialize_immutable_owner(accounts.first_chunk().unwrap())
        }
        // 23 - AmountToUiAmount
        23 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: AmountToUiAmount");

            test_process_amount_to_ui_amount(accounts.first_chunk().unwrap(), instruction_data.first_chunk().unwrap())
        }
        // 24 - UiAmountToAmount
        24 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: UiAmountToAmount");

            test_process_ui_amount_to_amount(
                accounts.first_chunk().unwrap(),
                // instruction_data.first_chunk().unwrap(),
                instruction_data, // Sized won't work
            )
        }
        // 38 - WithdrawExcessLamports
        38 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: WithdrawExcessLamports");

            test_process_withdraw_excess_lamports(accounts)
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
    use pinocchio_token_interface::state::mint::Mint;
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
    use pinocchio_token_interface::state::mint::Mint;
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
    use pinocchio_token_interface::state::{account, account_state};

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

    let is_native_mint = accounts[1].key() == &pinocchio_token_interface::native_mint::ID;

    let mint_is_initialised = unsafe {
        (accounts[1].borrow_data_unchecked().as_ptr() as *const pinocchio_token_interface::state::mint::Mint)
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
    } else if !is_native_mint && accounts[1].owner() != &pinocchio_token_interface::program::ID {
        assert_eq!(result, Err(ProgramError::IncorrectProgramId))
    } else if !is_native_mint
            && accounts[1].owner() == &pinocchio_token_interface::program::ID
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
    use pinocchio_token_interface::state::{account, account_state};

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

        if (accounts[0] == accounts[1] || amount == 0) && accounts[0].owner() != &pinocchio_token_interface::program::ID {
            assert_eq!(result, Err(ProgramError::IncorrectProgramId)) // UNTESTED
        } else if (accounts[0] == accounts[1] || amount == 0) && accounts[1].owner() != &pinocchio_token_interface::program::ID {
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
    use pinocchio_token_interface::state::{mint, account, account_state};

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

        if amount == 0 && accounts[0].owner() != &pinocchio_token_interface::program::ID {
            assert_eq!(result, Err(ProgramError::IncorrectProgramId)) // UNTESTED
        } else if amount == 0 && accounts[1].owner() != &pinocchio_token_interface::program::ID {
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

/// accounts[0] // Source Info
/// accounts[1] // Mint Info
/// accounts[2] // Authority Info
/// instruction_data[0..8] // Little Endian Bytes of u64 amount
#[inline(never)]
pub fn test_process_burn(accounts: &[AccountInfo; 3], instruction_data: &[u8; 8]) -> ProgramResult {
    use pinocchio_token_interface::state::{account, account_state, mint};

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
    let amount = || unsafe { u64::from_le_bytes(*(instruction_data.as_ptr() as *const [u8; 8])) };
    let src_initialised = get_account(&accounts[0]).is_initialized();
    let src_init_amount = get_account(&accounts[0]).amount();
    let src_init_state = get_account(&accounts[0]).account_state();
    let src_is_native = get_account(&accounts[0]).is_native();
    let src_mint = get_account(&accounts[0]).mint;
    let src_owned_sys_inc = get_account(&accounts[0]).is_owned_by_system_program_or_incinerator();
    let src_owner = get_account(&accounts[0]).owner;
    let mint_initialised = get_mint(&accounts[1]).is_initialized();
    let mint_init_supply = get_mint(&accounts[1]).supply();
    let mint_owner = get_account(&accounts[1]).owner;

    //-Process Instruction-----------------------------------------------------
    let result = process_burn(accounts, instruction_data);

    //-Assert Postconditions---------------------------------------------------
    if instruction_data.len() < 8 {
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if accounts.len() < 3 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys))
    } else if accounts[0].data_len() != account::Account::LEN {
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if !src_initialised.unwrap() { // UNTESTED
        assert_eq!(result, Err(ProgramError::UninitializedAccount))
    } else if accounts[1].data_len() != mint::Mint::LEN { // UNTESTED
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if !mint_initialised.unwrap() { // UNTESTED
        assert_eq!(result, Err(ProgramError::UninitializedAccount))
    } else if src_init_state.unwrap() == account_state::AccountState::Frozen {
        assert_eq!(result, Err(ProgramError::Custom(17)))
    } else if src_is_native {
        assert_eq!(result, Err(ProgramError::Custom(10)))
    } else if src_init_amount < amount() {
        assert_eq!(result, Err(ProgramError::Custom(1)))
    } else if accounts[1].key() != &src_mint {
        assert_eq!(result, Err(ProgramError::Custom(3)))
    } else {
        if !src_owned_sys_inc {
            // TODO validate_owner and delgated_amount
        }

        if amount() == 0 && src_owner != pinocchio_token_interface::program::ID { // UNTESTED
            assert_eq!(result, Err(ProgramError::IncorrectProgramId))
        } else if amount() == 0 && mint_owner != pinocchio_token_interface::program::ID { // UNTESTED
            assert_eq!(result, Err(ProgramError::IncorrectProgramId))
        } else {
            assert!(get_account(&accounts[0]).amount() == src_init_amount - amount());
            assert!(get_mint(&accounts[1]).supply() == mint_init_supply - amount());
            assert!(result.is_ok());
        }
    }

    result
}

/// accounts[0] // Source Info
/// accounts[1] // Destination Info
/// accounts[2] // Authority Info
#[inline(never)]
pub fn test_process_close_account(accounts: &[AccountInfo; 3]) -> ProgramResult {
    // use pinocchio_token_interface::state::{account, account_state, mint};
    use pinocchio_token_interface::state::account;

    // TODO: requires accounts[..] are all valid ptrs

    //-Helpers-----------------------------------------------------------------
    let get_account = |account_info: &AccountInfo| unsafe {
        (account_info.borrow_data_unchecked().as_ptr() as *const account::Account)
            .read()
    };

    //-Initial State-----------------------------------------------------------
    let src_initialised = get_account(&accounts[0]).is_initialized();
    let src_data_len = accounts[0].data_len();
    let src_init_amount = get_account(&accounts[0]).amount();
    let dst_init_lamports = accounts[0].lamports();
    let src_init_lamports = accounts[1].lamports();
    // let src_init_state = get_account(&accounts[0]).account_state();
    let src_is_native = get_account(&accounts[0]).is_native();
    // let src_mint = get_account(&accounts[0]).mint;
    let src_owned_sys_inc = get_account(&accounts[0]).is_owned_by_system_program_or_incinerator();
    // let src_owner = get_account(&accounts[0]).owner;

    //-Process Instruction-----------------------------------------------------
    let result = process_close_account(accounts);

    //-Assert Postconditions---------------------------------------------------
    if accounts.len() < 3 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys))
    } else if accounts[0] == accounts[2] {
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if src_data_len != account::Account::LEN {
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if !src_initialised.unwrap() { // UNTESTED
        assert_eq!(result, Err(ProgramError::UninitializedAccount))
    } else if !src_is_native && src_init_amount != 0 { // UNTESTED
        assert_eq!(result, Err(ProgramError::Custom(11)))
    } else {
        if !src_owned_sys_inc {
            // TODO: Validate owner
        } else if accounts[1].key() != &account::INCINERATOR_ID { // UNTESTED
            assert_eq!(result, Err(ProgramError::InvalidAccountData))
        } else if u64::MAX - src_init_lamports < dst_init_lamports { // UNTESTED
            assert_eq!(result, Err(ProgramError::Custom(14)))
        } else {
            assert_eq!(accounts[1].lamports(), dst_init_lamports + src_init_lamports);
            assert_eq!(accounts[0].data_len(), 0); // TODO: More sol_memset stuff?
            assert!(result.is_ok());
        }
    }
    result
}

/// accounts[0] // Source Info
/// accounts[1] // Mint Info
/// accounts[2] // Destination Info
/// accounts[3] // Authority Info
/// instruction_data[0..9] // Little Endian Bytes of u64 amount, and decimals
#[inline(never)]
pub fn test_process_transfer_checked(accounts: &[AccountInfo; 4], instruction_data: &[u8; 9]) -> ProgramResult {
    use pinocchio_token_interface::state::{account, account_state, mint::Mint};

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

        if (accounts[0] == accounts[2] || amount == 0) && accounts[0].owner() != &pinocchio_token_interface::program::ID {
            assert_eq!(result, Err(ProgramError::IncorrectProgramId)) // UNTESTED
        } else if (accounts[0] == accounts[2] || amount == 0) && accounts[2].owner() != &pinocchio_token_interface::program::ID {
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

/// accounts[0] // Source Info
/// accounts[1] // Mint Info
/// accounts[2] // Authority Info
/// instruction_data[0..9] // Little Endian Bytes of u64 amount, and decimals
#[inline(never)]
pub fn test_process_burn_checked(accounts: &[AccountInfo; 3], instruction_data: &[u8; 9]) -> ProgramResult {
    use pinocchio_token_interface::state::{account, account_state, mint};

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
    let amount = || unsafe { u64::from_le_bytes(*(instruction_data.as_ptr() as *const [u8; 8])) };
    let src_initialised = get_account(&accounts[0]).is_initialized();
    let src_init_amount = get_account(&accounts[0]).amount();
    let src_init_state = get_account(&accounts[0]).account_state();
    let src_is_native = get_account(&accounts[0]).is_native();
    let src_mint = get_account(&accounts[0]).mint;
    let src_owned_sys_inc = get_account(&accounts[0]).is_owned_by_system_program_or_incinerator();
    let src_owner = get_account(&accounts[0]).owner;
    let mint_initialised = get_mint(&accounts[1]).is_initialized();
    let mint_init_supply = get_mint(&accounts[1]).supply();
    let mint_decimals = get_mint(&accounts[1]).decimals;
    let mint_owner = get_account(&accounts[1]).owner;

    //-Process Instruction-----------------------------------------------------
    let result = process_burn_checked(accounts, instruction_data);

    //-Assert Postconditions---------------------------------------------------
    if instruction_data.len() < 9 {
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if accounts.len() < 3 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys))
    } else if accounts[0].data_len() != account::Account::LEN {
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if !src_initialised.unwrap() { // UNTESTED
        assert_eq!(result, Err(ProgramError::UninitializedAccount))
    } else if accounts[1].data_len() != mint::Mint::LEN { // UNTESTED
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if !mint_initialised.unwrap() { // UNTESTED
        assert_eq!(result, Err(ProgramError::UninitializedAccount))
    } else if src_init_state.unwrap() == account_state::AccountState::Frozen {
        assert_eq!(result, Err(ProgramError::Custom(17)))
    } else if src_is_native {
        assert_eq!(result, Err(ProgramError::Custom(10)))
    } else if src_init_amount < amount() {
        assert_eq!(result, Err(ProgramError::Custom(1)))
    } else if accounts[1].key() != &src_mint {
        assert_eq!(result, Err(ProgramError::Custom(3)))
    } else if instruction_data[8] != mint_decimals {
        assert_eq!(result, Err(ProgramError::Custom(18)))
    } else {
        if !src_owned_sys_inc {
            // TODO validate_owner and delgated_amount
        }

        if amount() == 0 && src_owner != pinocchio_token_interface::program::ID { // UNTESTED
            assert_eq!(result, Err(ProgramError::IncorrectProgramId))
        } else if amount() == 0 && mint_owner != pinocchio_token_interface::program::ID { // UNTESTED
            assert_eq!(result, Err(ProgramError::IncorrectProgramId))
        } else {
            assert!(get_account(&accounts[0]).amount() == src_init_amount - amount());
            assert!(get_mint(&accounts[1]).supply() == mint_init_supply - amount());
            assert!(result.is_ok());
        }
    }

    result
}

/// accounts[0] // New Account Info
/// accounts[1] // Mint Info
/// accounts[2] // Rent Sysvar Info
/// instruction_data[..] // Owner
#[inline(never)]
pub fn test_process_initialize_account2(accounts: &[AccountInfo; 3], instruction_data: &[u8; 32]) -> ProgramResult {
    use pinocchio_token_interface::state::{account, account_state};

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

    let is_native_mint = accounts[1].key() == &pinocchio_token_interface::native_mint::ID;

    let mint_is_initialised = unsafe {
        (accounts[1].borrow_data_unchecked().as_ptr() as *const pinocchio_token_interface::state::mint::Mint)
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
    } else if !is_native_mint && accounts[1].owner() != &pinocchio_token_interface::program::ID {
        assert_eq!(result, Err(ProgramError::IncorrectProgramId))
    } else if !is_native_mint
            && accounts[1].owner() == &pinocchio_token_interface::program::ID
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
    use pinocchio_token_interface::state::{account, account_state};

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

    let is_native_mint = accounts[1].key() == &pinocchio_token_interface::native_mint::ID;

    let mint_is_initialised = unsafe {
        (accounts[1].borrow_data_unchecked().as_ptr() as *const pinocchio_token_interface::state::mint::Mint)
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
    } else if !is_native_mint && accounts[1].owner() != &pinocchio_token_interface::program::ID {
        assert_eq!(result, Err(ProgramError::IncorrectProgramId))
    } else if !is_native_mint
            && accounts[1].owner() == &pinocchio_token_interface::program::ID
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
    use pinocchio_token_interface::state::mint::Mint;
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
    use pinocchio_token_interface::state::mint::Mint;
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

#[inline(never)]
fn test_process_initialize_multisig(accounts: &[AccountInfo; 5], instruction_data: &[u8; 1]) -> ProgramResult {
    process_initialize_multisig(accounts, instruction_data)
}

/// accounts[0] // Source Account Info
/// accounts[1] // Delegate Info
/// accounts[2] // Owner Info
/// instruction_data[0..8] // Little Endian Bytes of u64 amount
#[inline(never)]
fn test_process_approve(accounts: &[AccountInfo; 3], instruction_data: &[u8; 8]) -> ProgramResult {
    use pinocchio_token_interface::state::{account, account_state};

    // TODO: requires accounts[..] are all valid ptrs

    //-Helpers-----------------------------------------------------------------
    let get_account = |account_info: &AccountInfo| unsafe {
        (account_info.borrow_data_unchecked().as_ptr() as *const account::Account)
            .read()
    };

    //-Initial State-----------------------------------------------------------
    let amount = unsafe { u64::from_le_bytes(*(instruction_data.as_ptr() as *const [u8; 8])) };

    //-Process Instruction-----------------------------------------------------
    let result = process_approve(accounts, instruction_data);

    //-Assert Postconditions---------------------------------------------------
    if instruction_data.len() < 8 {
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if accounts.len() < 3 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys))
    } else if accounts[0].data_len() != account::Account::LEN {
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if !get_account(&accounts[0]).is_initialized().unwrap() { // UNTESTED
        assert_eq!(result, Err(ProgramError::UninitializedAccount))
    } else if get_account(&accounts[0]).account_state().unwrap() == account_state::AccountState::Frozen  {
        assert_eq!(result, Err(ProgramError::Custom(17)))
    } else {
        // TODO validate owner

        assert_eq!(get_account(&accounts[0]).delegate().unwrap(), accounts[1].key());
        assert_eq!(get_account(&accounts[0]).delegated_amount(), amount);
        assert!(result.is_ok())
    }

    result
}

/// accounts[0] // Source Account Info
/// accounts[1] // Owner Info
#[inline(never)]
fn test_process_revoke(accounts: &[AccountInfo; 2]) -> ProgramResult {
    use pinocchio_token_interface::state::{account, account_state};

    // TODO: requires accounts[..] are all valid ptrs

    //-Helpers-----------------------------------------------------------------
    let get_account = |account_info: &AccountInfo| unsafe {
        (account_info.borrow_data_unchecked().as_ptr() as *const account::Account)
            .read()
    };

    //-Initial State-----------------------------------------------------------
    let src_initialised = get_account(&accounts[0]).is_initialized();
    let src_init_state = get_account(&accounts[0]).account_state();

    //-Process Instruction-----------------------------------------------------
    let result = process_revoke(accounts);

    //-Assert Postconditions---------------------------------------------------
    if accounts.len() < 1 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys))
    } else if accounts[0].data_len() != account::Account::LEN {
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if !src_initialised.unwrap() { // UNTESTED
        assert_eq!(result, Err(ProgramError::UninitializedAccount))
    } else if accounts.len() < 2 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys))
    } else if src_init_state.is_err() { // UNTESTED
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if src_init_state.unwrap() == account_state::AccountState::Frozen {
        assert_eq!(result, Err(ProgramError::Custom(17)))
    } else {
        // TODO: validate owner / signers

        assert!(get_account(&accounts[0]).delegate().is_none());
        assert_eq!(get_account(&accounts[0]).delegated_amount(), 0);
        assert!(result.is_ok())
    }

    result
}

/// accounts[0] // Account Info
/// accounts[1] // Authority Info
/// instruction_data[0] // Authority Type (instruction)
/// instruction_data[1] // New Authority Follows (0 -> No, 1 -> Yes)
/// instruction_data[2..34] // New Authority Pubkey
#[inline(never)]
fn test_process_set_authority(accounts: &[AccountInfo; 2], instruction_data: &[u8]) -> ProgramResult {
   use pinocchio_token_interface::state::{account, account_state, mint};

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
    let src_initialised = get_account(&accounts[0]).is_initialized();
    let src_init_state = get_account(&accounts[0]).account_state();
    // let authority_type = AuthorityType::from(instruction_data[0]);
    // let authority_type = unsafe { AuthorityType::try_from(*instruction_data.get_unchecked(0)) }; // FIXME
    let account_data_len = accounts[0].data_len();
    let old_mint_authority_is_none = get_mint(&accounts[0]).mint_authority().is_none(); // FIXME
    let old_freeze_authority_is_none = get_mint(&accounts[0]).freeze_authority().is_none(); // FIXME

    //-Process Instruction-----------------------------------------------------
    let result = process_set_authority(accounts, instruction_data);

    //-Assert Postconditions---------------------------------------------------
    if instruction_data.len() < 2 {
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if !(0..=3).contains(&instruction_data[0]) { // UNTESTED
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if instruction_data[1] != 0 && instruction_data[1] != 1 { // UNTESTED
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if instruction_data[1] == 1 && instruction_data.len() < 34 { // UNTESTED
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if accounts.len() < 2 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys))
    } else if account_data_len != account::Account::LEN && account_data_len != mint::Mint::LEN {
         assert_eq!(result, Err(ProgramError::InvalidArgument))
    } else {
        if account_data_len == account::Account::LEN {
            if !src_initialised.unwrap() { // UNTESTED
                assert_eq!(result, Err(ProgramError::UninitializedAccount))
            } else if src_init_state.unwrap() == account_state::AccountState::Frozen { // UNTESTED
                assert_eq!(result, Err(ProgramError::Custom(17)))
            } else if instruction_data[0] != 2 && instruction_data[0] != 3 { // UNTESTED: AuthorityType neither AccountOwner nor CloseAccount
                assert_eq!(result, Err(ProgramError::Custom(15)))
            } else {
                if instruction_data[0] == 2 { // AccountOwner
                    // TODO: Validate Owner

                    if instruction_data[1] != 1 || instruction_data.len() < 34 { // UNTESTED
                        assert_eq!(result, Err(ProgramError::Custom(12)))
                    } else {
                        assert_eq!(get_account(&accounts[0]).owner, instruction_data[2..34]); // UNTESTED
                        assert_eq!(get_account(&accounts[0]).delegate(), None); // UNTESTED
                        assert_eq!(get_account(&accounts[0]).delegated_amount(), 0); // UNTESTED
                        if get_account(&accounts[0]).is_native() {
                            assert_eq!(get_account(&accounts[0]).close_authority(), None); // UNTESTED
                        }
                        assert!(result.is_ok()) //  UNTESTED
                    }
                } else { // Close Account
                    // TODO Validate Owner

                    if instruction_data[1] == 1 { // UNTESTED: 1 ==> 34 <= instruction_data.len()
                        assert_eq!(get_account(&accounts[0]).close_authority().unwrap(), &instruction_data[2..34]); // UNTESTED
                    } else {
                        assert_eq!(get_account(&accounts[0]).close_authority(), None); // UNTESTED
                    }
                    assert!(result.is_ok()) //  UNTESTED
                }
            }
        } else { // account_data_len == mint::Mint::LEN
            if !get_mint(&accounts[0]).is_initialized().unwrap() { // UNTESTED: FIXME not accessing old
                assert_eq!(result, Err(ProgramError::UninitializedAccount))
            } else if instruction_data[0] != 0 && instruction_data[0] != 1 { // UNTESTED: AuthorityType neither MintTokens nor FreezeAccount
                assert_eq!(result, Err(ProgramError::Custom(15)))
            } else {
                if instruction_data[0] == 0 { // MintTokens
                    if old_mint_authority_is_none { // UNTESTED
                        assert_eq!(result, Err(ProgramError::Custom(5)))
                    } /* else if TODO Validate owner {
                        // TODO Validate owner
                    } */ else {
                        if instruction_data[1] == 1 { // UNTESTED: 1 ==> 34 <= instruction_data.len()
                            assert_eq!(get_mint(&accounts[0]).mint_authority().unwrap(), &instruction_data[2..34]); // UNTESTED
                        } else {
                            assert_eq!(get_mint(&accounts[0]).mint_authority(), None); // UNTESTED
                        }
                        assert!(result.is_ok()) //  UNTESTED
                    }
                } else { // FreezeAccount
                    if old_freeze_authority_is_none { // UNTESTED
                        assert_eq!(result, Err(ProgramError::Custom(16)))
                    } /* else if TODO Validate owner {
                        // TODO Validate owner
                    } */ else {
                        if instruction_data[1] == 1 { // UNTESTED: 1 ==> 34 <= instruction_data.len()
                            assert_eq!(get_mint(&accounts[0]).freeze_authority().unwrap(), &instruction_data[2..34]); // UNTESTED
                        } else {
                            assert_eq!(get_mint(&accounts[0]).freeze_authority(), None); // UNTESTED
                        }
                        assert!(result.is_ok()) //  UNTESTED
                    }
                }
            }
        }
    }

    result
}

/// accounts[0] // Source Account Info
/// accounts[1] // Mint Info
/// accounts[2] // Authority Info
#[inline(never)]
fn test_process_freeze_account(accounts: &[AccountInfo; 3]) -> ProgramResult {
    use pinocchio_token_interface::state::{account, account_state, mint};

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
    let src_initialised = get_account(&accounts[0]).is_initialized();
    let src_init_state = get_account(&accounts[0]).account_state();
    let src_is_native = get_account(&accounts[0]).is_native();
    let src_mint = get_account(&accounts[0]).mint;
    let mint_initialised = get_mint(&accounts[1]).is_initialized();
    let mint_freeze_auth = get_mint(&accounts[1]).freeze_authority().cloned();

    //-Process Instruction-----------------------------------------------------
    let result = process_freeze_account(accounts);

    //-Assert Postconditions---------------------------------------------------
    if accounts.len() < 3 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys))
    } else if accounts[0].data_len() != account::Account::LEN {
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if !src_initialised.unwrap() { // UNTESTED
        assert_eq!(result, Err(ProgramError::UninitializedAccount))
    } else if src_init_state.is_err() { // UNTESTED
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if src_init_state.unwrap() == account_state::AccountState::Frozen { // UNTESTED
        // TODO: Why is the double test not throwing an error?
        assert_eq!(result, Err(ProgramError::Custom(13)))
    } else if src_is_native {
        assert_eq!(result, Err(ProgramError::Custom(10)))
    } else if accounts[1].key() != &src_mint {
        assert_eq!(result, Err(ProgramError::Custom(3)))
    } else if accounts[1].data_len() != mint::Mint::LEN { // UNTESTED
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if !mint_initialised.unwrap() { // UNTESTED
        assert_eq!(result, Err(ProgramError::UninitializedAccount))
    } else if mint_freeze_auth.is_none() {
        assert_eq!(result, Err(ProgramError::Custom(16)))
    } else {
        // TODO: Validate owner is authority

        assert_eq!(get_account(&accounts[0]).account_state().unwrap(), account_state::AccountState::Frozen);
        assert!(result.is_ok())
    }
    result
}

/// accounts[0] // Source Account Info
/// accounts[1] // Mint Info
/// accounts[2] // Authority Info
#[inline(never)]
fn test_process_thaw_account(accounts: &[AccountInfo; 3]) -> ProgramResult {
    use pinocchio_token_interface::state::{account, account_state, mint};

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
    let src_initialised = get_account(&accounts[0]).is_initialized();
    let src_init_state = get_account(&accounts[0]).account_state();
    let src_is_native = get_account(&accounts[0]).is_native();
    let src_mint = get_account(&accounts[0]).mint;
    let mint_initialised = get_mint(&accounts[1]).is_initialized();
    let mint_freeze_auth = get_mint(&accounts[1]).freeze_authority().cloned();

    //-Process Instruction-----------------------------------------------------
    let result = process_thaw_account(accounts);

    //-Assert Postconditions---------------------------------------------------
    if accounts.len() < 3 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys))
    } else if accounts[0].data_len() != account::Account::LEN {
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if !src_initialised.unwrap() { // UNTESTED
        assert_eq!(result, Err(ProgramError::UninitializedAccount))
    } else if src_init_state.is_err() { // UNTESTED
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if src_init_state.unwrap() != account_state::AccountState::Frozen {
        assert_eq!(result, Err(ProgramError::Custom(13)))
    } else if src_is_native { // UNTESTED
        // TODO: Unsure if it is even possible to freeze a native mint
        assert_eq!(result, Err(ProgramError::Custom(10)))
    } else if accounts[1].key() != &src_mint {
        assert_eq!(result, Err(ProgramError::Custom(3)))
    } else if accounts[1].data_len() != mint::Mint::LEN { // UNTESTED
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if !mint_initialised.unwrap() { // UNTESTED
        assert_eq!(result, Err(ProgramError::UninitializedAccount))
    } else if mint_freeze_auth.is_none() { // UNTESTED
        // TODO: Not sure how to freeze to then thaw
        assert_eq!(result, Err(ProgramError::Custom(16)))
    } else {
        // TODO: Validate owner is authority

        assert_eq!(get_account(&accounts[0]).account_state().unwrap(), account_state::AccountState::Initialized);
        assert!(result.is_ok())
    }
    result
}

/// accounts[0] // Source Account Info
/// accounts[1] // Expected Mint Info
/// accounts[2] // Delegate Info
/// accounts[3] // Owner Info
/// instruction_data[0..9] // Little Endian Bytes of u64 amount, and decimals
#[inline(never)]
fn test_process_approve_checked(accounts: &[AccountInfo; 4], instruction_data: &[u8; 9]) -> ProgramResult {
    use pinocchio_token_interface::state::{account, account_state, mint};

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
    let amount = unsafe { u64::from_le_bytes(*(instruction_data.as_ptr() as *const [u8; 8])) };

    //-Process Instruction-----------------------------------------------------
    let result = process_approve_checked(accounts, instruction_data);

    //-Assert Postconditions---------------------------------------------------
    if instruction_data.len() < 9 {
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if accounts.len() < 4 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys))
    } else if accounts[0].data_len() != account::Account::LEN {
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if !get_account(&accounts[0]).is_initialized().unwrap() { // UNTESTED
        assert_eq!(result, Err(ProgramError::UninitializedAccount))
    } else if get_account(&accounts[0]).account_state().unwrap() == account_state::AccountState::Frozen  {
        assert_eq!(result, Err(ProgramError::Custom(17)))
    } else if accounts[1].key() != &get_account(&accounts[0]).mint {
        assert_eq!(result, Err(ProgramError::Custom(3)))
    } else if accounts[1].data_len() != mint::Mint::LEN { // UNTESTED
        // Not sure if this is even possible if we get past the case above
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if !get_mint(&accounts[1]).is_initialized().unwrap() { // UNTESTED
        assert_eq!(result, Err(ProgramError::UninitializedAccount))
    } else if instruction_data[8] != get_mint(&accounts[1]).decimals { // UNTESTED
        assert_eq!(result, Err(ProgramError::Custom(18)))
    } else {
        // TODO validate owner

        assert_eq!(get_account(&accounts[0]).delegate().unwrap(), accounts[2].key());
        assert_eq!(get_account(&accounts[0]).delegated_amount(), amount);
        assert!(result.is_ok())
    }

    result
}

/// accounts[0] // Mint Info
/// accounts[1] // Destination Info
/// accounts[2] // Owner Info
/// instruction_data[0..9] // Little Endian Bytes of u64 amount, and decimals
#[inline(never)]
fn test_process_mint_to_checked(accounts: &[AccountInfo; 3], instruction_data: &[u8; 9]) -> ProgramResult {
    use pinocchio_token_interface::state::{mint, account, account_state};

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

        if amount == 0 && accounts[0].owner() != &pinocchio_token_interface::program::ID {
            assert_eq!(result, Err(ProgramError::IncorrectProgramId)) // UNTESTED
        } else if amount == 0 && accounts[1].owner() != &pinocchio_token_interface::program::ID {
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
fn test_process_sync_native(accounts: &[AccountInfo; 1]) -> ProgramResult {
    use pinocchio_token_interface::{program, state::account};

    // TODO: requires accounts[..] are all valid ptrs

    //-Helpers-----------------------------------------------------------------
    let get_account = |account_info: &AccountInfo| unsafe {
        (account_info.borrow_data_unchecked().as_ptr() as *const account::Account)
            .read()
    };

    //-Initial State-----------------------------------------------------------
    let src_owner = accounts[0].owner();
    let src_initialised = get_account(&accounts[0]).is_initialized();
    let src_native_amount = get_account(&accounts[0]).native_amount();
    let src_init_lamports = accounts[0].lamports();
    let src_init_amount = get_account(&accounts[0]).amount();

    //-Process Instruction-----------------------------------------------------
    let result = process_sync_native(accounts);

    //-Assert Postconditions---------------------------------------------------
    if accounts.len() != 1 { // Untested
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys))
    } else if src_owner != &program::ID { // UNTESTED
        assert_eq!(result, Err(ProgramError::IncorrectProgramId))
    } else if accounts[0].data_len() != account::Account::LEN { // UNTESTED
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if !src_initialised.unwrap() { // UNTESTED
        assert_eq!(result, Err(ProgramError::UninitializedAccount))
    } else if src_native_amount.is_none() { // UNTESTED
        assert_eq!(result, Err(ProgramError::Custom(19)))
    } else if src_init_lamports < src_native_amount.unwrap() { // UNTESTED
        assert_eq!(result, Err(ProgramError::Custom(14)))
    } else if src_init_lamports - src_native_amount.unwrap() < src_init_amount { // UNTESTED
        assert_eq!(result, Err(ProgramError::Custom(13)))
    } else { // UNTESTED
        assert_eq!(get_account(&accounts[0]).amount(), src_init_lamports - src_native_amount.unwrap());
        assert!(result.is_ok())
    }
    result
}

#[inline(never)]
fn test_process_initialize_multisig2(accounts: &[AccountInfo; 4], instruction_data: &[u8; 1]) -> ProgramResult {
    process_initialize_multisig2(accounts, instruction_data)
}

/// accounts[0] // Mint Info
#[inline(never)]
fn test_process_get_account_data_size(accounts: &[AccountInfo; 1]) -> ProgramResult {
    use pinocchio_token_interface::state::mint;

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
    } else if accounts[0].owner() != &pinocchio_token_interface::program::ID { // UNTESTED
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

#[inline(never)]
fn test_process_initialize_immutable_owner(accounts: &[AccountInfo; 1]) -> ProgramResult {
    use pinocchio_token_interface::state::account;

    // TODO: requires accounts[..] are all valid ptrs

    //-Helpers-----------------------------------------------------------------
    let get_account = |account_info: &AccountInfo| unsafe {
        (account_info.borrow_data_unchecked().as_ptr() as *const account::Account)
            .read()
    };

    //-Initial State-----------------------------------------------------------
    let src_initialised = get_account(&accounts[0]).is_initialized();

    //-Process Instruction-----------------------------------------------------
    let result = process_initialize_immutable_owner(accounts);

    //-Assert Postconditions---------------------------------------------------
    if accounts.len() != 1 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys)) // UNTESTED
    } else if accounts[0].data_len() != account::Account::LEN {
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if src_initialised.unwrap() { // UNTESTED
        assert_eq!(result, Err(ProgramError::Custom(6)))
    } else { // UNTESTED
        assert!(result.is_ok())
    }
    result
}

#[inline(never)]
fn test_process_amount_to_ui_amount(accounts: &[AccountInfo; 1], instruction_data: &[u8; 8]) -> ProgramResult {
    use pinocchio_token_interface::state::mint;

    // TODO: requires accounts[..] are all valid ptrs

    //-Helpers-----------------------------------------------------------------
    let get_mint = |account_info: &AccountInfo| unsafe {
        (account_info.borrow_data_unchecked().as_ptr() as *const mint::Mint)
            .read()
    };

    //-Initial State-----------------------------------------------------------

    //-Process Instruction-----------------------------------------------------
    let result = process_amount_to_ui_amount(accounts, instruction_data);

    //-Assert Postconditions---------------------------------------------------
    if instruction_data.len() < 8 {
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if accounts.len() < 1 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys))
    } else if accounts[0].owner() != &pinocchio_token_interface::program::ID { // UNTESTED
        assert_eq!(result, Err(ProgramError::IncorrectProgramId))
    } else if accounts[0].data_len() != mint::Mint::LEN {
        assert_eq!(result, Err(ProgramError::Custom(2)))
    } else if !get_mint(&accounts[0]).is_initialized().unwrap() {
        assert_eq!(result, Err(ProgramError::Custom(2)))
    } else {
        // TODO: Checking the return data is correct
        assert!(result.is_ok())
    }
    result
}

#[inline(never)]
fn test_process_ui_amount_to_amount(accounts: &[AccountInfo; 1], instruction_data: &[u8]) -> ProgramResult {
    use pinocchio_token_interface::state::mint;

    // TODO: requires accounts[..] are all valid ptrs

    // //-Helpers-----------------------------------------------------------------
    let get_mint = |account_info: &AccountInfo| unsafe {
        (account_info.borrow_data_unchecked().as_ptr() as *const mint::Mint)
            .read()
    };

    // //-Initial State-----------------------------------------------------------
    let ui_amount = core::str::from_utf8(instruction_data);

    //-Process Instruction-----------------------------------------------------
    let result = process_ui_amount_to_amount(accounts, instruction_data);

    //-Assert Postconditions---------------------------------------------------
    // TODO: validations module is private, so we need a work around
    if ui_amount.is_err() { // UNTESTED
        assert_eq!(result, Err(ProgramError::Custom(12)))
    } else if accounts.len() < 1 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys))
    } else if accounts[0].owner() != &pinocchio_token_interface::program::ID { // UNTESTED
        assert_eq!(result, Err(ProgramError::IncorrectProgramId))
    } else if accounts[0].data_len() != mint::Mint::LEN {
        assert_eq!(result, Err(ProgramError::Custom(2)))
    } else if !get_mint(&accounts[0]).is_initialized().unwrap() {
        assert_eq!(result, Err(ProgramError::Custom(2)))
    } else if ui_amount.unwrap().is_empty() {
        assert_eq!(result, Err(ProgramError::InvalidArgument))
    } else if ui_amount.unwrap() == "." {
        assert_eq!(result, Err(ProgramError::InvalidArgument))
    } else if 1 < ui_amount.unwrap().chars().filter(|&c| c == '.').count() {
        assert_eq!(result, Err(ProgramError::InvalidArgument))
    } else if ui_amount.unwrap().starts_with('.') && ui_amount.unwrap().chars().skip(1).all(|c| c == '0') {
        assert_eq!(result, Err(ProgramError::InvalidArgument))
    } else if ui_amount.unwrap().split_once('.').map_or(false, |(_, frac)| { (get_mint(&accounts[0]).decimals as usize) < frac.trim_end_matches('0').len()}) {
        assert_eq!(result, Err(ProgramError::InvalidArgument))
    } else if ui_amount.unwrap().split_once('.').map_or(
        257_usize < ui_amount.unwrap().len() + (get_mint(&accounts[0]).decimals as usize),
        |(ints, _)| { 257_usize < ints.len() + (get_mint(&accounts[0]).decimals as usize) }) {
            assert_eq!(result, Err(ProgramError::InvalidArgument))
    } /*else if ui_amount.unwrap() == "+." {
        // TODO: Why is this valid?
        assert_eq!(result, Err(ProgramError::InvalidArgument))
    } else if ui_amount.unwrap() == "+" {
        // TODO: Why is this valid?
        assert_eq!(result, Err(ProgramError::InvalidArgument))
    }*/ else if ui_amount.unwrap().chars().nth(0).unwrap() == '-' {
        assert_eq!(result, Err(ProgramError::InvalidArgument))
    } else if ui_amount.unwrap().contains(|c: char| !c.is_digit(10) && c != '+' && c != '.') {
        assert_eq!(result, Err(ProgramError::InvalidArgument))
    } else if ui_amount.unwrap().split_once('.').map_or(
        {
            const MAX_VAL: &str = "1844674407370955"; // TODO: What should this be?
            let ui_amount = ui_amount.unwrap();
            let ui_amount = ui_amount.strip_prefix('+').unwrap_or(ui_amount);
            let ui_amount = ui_amount.trim_start_matches('0');
            match ui_amount.len().cmp(&MAX_VAL.len()) {
                core::cmp::Ordering::Less => false,
                core::cmp::Ordering::Greater => true,
                core::cmp::Ordering::Equal => MAX_VAL < ui_amount,
            }
        },
        |(ints, fracs)| {
            const MAX_VAL: &str = "1844674407370955"; // TODO: What should this be?
            let ints = ints.strip_prefix('+').unwrap_or(ints);
            let hi = ints.trim_start_matches('0');
            let lo = if hi.is_empty() { fracs.trim_start_matches('0') } else { fracs };

            let total_len = hi.len() + lo.len();

            match total_len.cmp(&MAX_VAL.len()) {
                core::cmp::Ordering::Less => false,
                core::cmp::Ordering::Greater => { true },
                core::cmp::Ordering::Equal => {
                    if hi.len() > MAX_VAL.len() {
                        return true;
                    }
                    let (max_hi, max_lo) = MAX_VAL.split_at(hi.len());
                    hi > max_hi || (hi == max_hi && lo > max_lo)
                }
            }
        }
    ) {
        // TODO: What is going on ??? Need to fix
        // assert_eq!(result, Err(ProgramError::InvalidArgument))
    } else {
        assert!(result.is_ok())
    }
    result
}

#[inline(never)]
fn test_process_withdraw_excess_lamports(accounts: &[AccountInfo]) -> ProgramResult {
    process_withdraw_excess_lamports(accounts)
}
