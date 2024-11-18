use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use token_interface::{error::TokenError, state::multisig::Multisig};

pub fn process_initialize_multisig(
    accounts: &[AccountInfo],
    m: u8,
    rent_sysvar_account: bool,
) -> ProgramResult {
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
        let rent = unsafe { Rent::from_bytes(rent_sysvar_info.borrow_data_unchecked()) };
        rent.is_exempt(multisig_info.lamports(), multisig_info_data_len)
    } else {
        Rent::get()?.is_exempt(multisig_info.lamports(), multisig_info_data_len)
    };

    if !is_exempt {
        return Err(TokenError::NotRentExempt.into());
    }

    let multisig = bytemuck::try_from_bytes_mut::<Multisig>(unsafe {
        multisig_info.borrow_mut_data_unchecked()
    })
    .map_err(|_error| ProgramError::InvalidAccountData)?;

    if multisig.is_initialized.into() {
        return Err(TokenError::AlreadyInUse.into());
    }

    multisig.m = m;
    multisig.n = remaining.len() as u8;

    if !Multisig::is_valid_signer_index(multisig.n as usize) {
        return Err(TokenError::InvalidNumberOfProvidedSigners.into());
    }
    if !Multisig::is_valid_signer_index(multisig.m as usize) {
        return Err(TokenError::InvalidNumberOfRequiredSigners.into());
    }
    for (i, signer_info) in remaining.iter().enumerate() {
        multisig.signers[i] = *signer_info.key();
    }
    multisig.is_initialized = true.into();

    Ok(())
}
