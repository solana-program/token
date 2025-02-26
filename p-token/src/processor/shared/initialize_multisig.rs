use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use token_interface::{
    error::TokenError,
    state::{load_mut_unchecked, multisig::Multisig, Initializable},
};

#[inline(always)]
pub fn process_initialize_multisig(
    accounts: &[AccountInfo],
    m: u8,
    // JC nit: same here, whatever name we choose for this variable
    rent_sysvar_account: bool,
) -> ProgramResult {
    // Accounts expected depend on whether we have the `rent_sysvar` account or not.

    let (multisig_info, rent_sysvar_info, remaining) = if rent_sysvar_account {
        let [multisig_info, rent_sysvar_info, remaining @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        (multisig_info, Some(rent_sysvar_info), remaining)
    } else {
        let [multisig_info, remaining @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        (multisig_info, None, remaining)
    };

    let multisig_info_data_len = multisig_info.data_len();

    let is_exempt = if let Some(rent_sysvar_info) = rent_sysvar_info {
        // SAFETY: single immutable borrow to `rent_sysvar_info`; account ID and length are
        // checked by `from_account_info_unchecked`.
        let rent = unsafe { Rent::from_account_info_unchecked(rent_sysvar_info)? };
        rent.is_exempt(multisig_info.lamports(), multisig_info_data_len)
    } else {
        Rent::get()?.is_exempt(multisig_info.lamports(), multisig_info_data_len)
    };

    // SAFETY: single mutable borrow to `multisig_info` account data.
    let multisig =
        unsafe { load_mut_unchecked::<Multisig>(multisig_info.borrow_mut_data_unchecked())? };

    if multisig.is_initialized() {
        return Err(TokenError::AlreadyInUse.into());
    }

    if !is_exempt {
        return Err(TokenError::NotRentExempt.into());
    }

    // Initialize the multisig account.

    multisig.m = m;
    multisig.n = remaining.len() as u8;

    // JC nit: just wondering, does it make any difference in CUs to change
    // `is_valid_signer_index` to take in a `u8` instead of a `usize` and avoid
    // widening?
    if !Multisig::is_valid_signer_index(multisig.n as usize) {
        return Err(TokenError::InvalidNumberOfProvidedSigners.into());
    }
    if !Multisig::is_valid_signer_index(multisig.m as usize) {
        return Err(TokenError::InvalidNumberOfRequiredSigners.into());
    }

    for (i, signer_info) in remaining.iter().enumerate() {
        multisig.signers[i] = *signer_info.key();
    }

    multisig.set_initialized(true);

    Ok(())
}
