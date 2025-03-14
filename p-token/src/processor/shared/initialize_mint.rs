use {
    core::mem::size_of,
    pinocchio::{
        account_info::AccountInfo,
        program_error::ProgramError,
        pubkey::Pubkey,
        sysvars::{rent::Rent, Sysvar},
        ProgramResult,
    },
    spl_token_interface::{
        error::TokenError,
        state::{load_mut_unchecked, mint::Mint, Initializable},
    },
};

#[inline(always)]
pub fn process_initialize_mint(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
    rent_sysvar_account_provided: bool,
) -> ProgramResult {
    // Validates the instruction data.

    // SAFETY: The minimum size of the instruction data is either 34 or 66 bytes:
    //   - decimals (1 byte)
    //   - mint_authority (32 bytes)
    //   - option + freeze_authority (1 byte + 32 bytes)
    let (decimals, mint_authority, freeze_authority) = unsafe {
        match instruction_data.len() {
            34 if *instruction_data.get_unchecked(33) == 0 => (
                *instruction_data.get_unchecked(0),
                &*(instruction_data.as_ptr().add(1) as *const Pubkey),
                None,
            ),
            66 if *instruction_data.get_unchecked(33) == 1 => (
                *instruction_data.get_unchecked(0),
                &*(instruction_data.as_ptr().add(1) as *const Pubkey),
                Some(&*(instruction_data.as_ptr().add(34) as *const Pubkey)),
            ),
            _ => return Err(ProgramError::InvalidInstructionData),
        }
    };

    // Validates the accounts.

    let (mint_info, rent_sysvar_info) = if rent_sysvar_account_provided {
        let [mint_info, rent_sysvar_info, _remaining @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        (mint_info, Some(rent_sysvar_info))
    } else {
        let [mint_info, _remaining @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        (mint_info, None)
    };

    // SAFETY: single mutable borrow to `mint_info` account data.
    let mint = unsafe { load_mut_unchecked::<Mint>(mint_info.borrow_mut_data_unchecked())? };

    if mint.is_initialized() {
        return Err(TokenError::AlreadyInUse.into());
    }

    // Check rent-exempt status of the mint account.

    let is_exempt = if let Some(rent_sysvar_info) = rent_sysvar_info {
        // SAFETY: single immutable borrow to `rent_sysvar_info`; account ID and length
        // are checked by `from_account_info_unchecked`.
        let rent = unsafe { Rent::from_account_info_unchecked(rent_sysvar_info)? };
        rent.is_exempt(mint_info.lamports(), size_of::<Mint>())
    } else {
        Rent::get()?.is_exempt(mint_info.lamports(), size_of::<Mint>())
    };

    if !is_exempt {
        return Err(TokenError::NotRentExempt.into());
    }

    // Initialize the mint.

    mint.set_initialized();
    mint.set_mint_authority(mint_authority);
    mint.decimals = decimals;

    if let Some(freeze_authority) = freeze_authority {
        mint.set_freeze_authority(freeze_authority);
    }

    Ok(())
}
