use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};
use spl_token_interface::{
    error::TokenError,
    instruction::AuthorityType,
    state::{account::Account, load_mut, mint::Mint, Transmutable},
};

use super::validate_owner;

#[inline(always)]
pub fn process_set_authority(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    // Validates the instruction data.

    let args = SetAuthority::try_from_bytes(instruction_data)?;

    // Validates the accounts.

    let [account_info, authority_info, remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if account_info.data_len() == Account::LEN {
        // SAFETY: single mutable borrow to `account_info` account data and
        // `load_mut` validates that the account is initialized.
        let account = unsafe { load_mut::<Account>(account_info.borrow_mut_data_unchecked())? };

        if account.is_frozen() {
            return Err(TokenError::AccountFrozen.into());
        }

        match args.authority_type()? {
            AuthorityType::AccountOwner => {
                validate_owner(&account.owner, authority_info, remaining)?;

                if let Some(authority) = args.new_authority() {
                    account.owner = *authority;
                } else {
                    return Err(TokenError::InvalidInstruction.into());
                }

                account.clear_delegate();
                account.set_delegated_amount(0);

                if account.is_native() {
                    account.clear_close_authority();
                }
            }
            AuthorityType::CloseAccount => {
                let authority = account.close_authority().unwrap_or(&account.owner);
                validate_owner(authority, authority_info, remaining)?;

                if let Some(authority) = args.new_authority() {
                    account.set_close_authority(authority);
                } else {
                    account.clear_close_authority();
                }
            }
            _ => {
                return Err(TokenError::AuthorityTypeNotSupported.into());
            }
        }
    } else if account_info.data_len() == Mint::LEN {
        // SAFETY: single mutable borrow to `account_info` account data and
        // `load_mut` validates that the mint is initialized.
        let mint = unsafe { load_mut::<Mint>(account_info.borrow_mut_data_unchecked())? };

        match args.authority_type()? {
            AuthorityType::MintTokens => {
                // Once a mint's supply is fixed, it cannot be undone by setting a new
                // mint_authority.
                let mint_authority = mint.mint_authority().ok_or(TokenError::FixedSupply)?;

                validate_owner(mint_authority, authority_info, remaining)?;

                if let Some(authority) = args.new_authority() {
                    mint.set_mint_authority(authority);
                } else {
                    mint.clear_mint_authority();
                }
            }
            AuthorityType::FreezeAccount => {
                // Once a mint's freeze authority is disabled, it cannot be re-enabled by
                // setting a new freeze_authority.
                let freeze_authority = mint
                    .freeze_authority()
                    .ok_or(TokenError::MintCannotFreeze)?;

                validate_owner(freeze_authority, authority_info, remaining)?;

                if let Some(authority) = args.new_authority() {
                    mint.set_freeze_authority(authority);
                } else {
                    mint.clear_freeze_authority();
                }
            }
            _ => {
                return Err(TokenError::AuthorityTypeNotSupported.into());
            }
        }
    } else {
        return Err(ProgramError::InvalidArgument);
    }

    Ok(())
}

#[repr(C)]
struct SetAuthority {
    authority_type: u8,

    new_authority: (u8, Pubkey),
}

impl SetAuthority {
    #[inline]
    pub fn try_from_bytes(bytes: &[u8]) -> Result<&SetAuthority, ProgramError> {
        // The minimum expected size of the instruction data is either 2 or 34 bytes:
        //   - authority_type (1 byte)
        //   - option + new_authority (1 byte + 32 bytes)
        unsafe {
            match bytes.len() {
                2 if *bytes.get_unchecked(1) == 0 => Ok(&*(bytes.as_ptr() as *const SetAuthority)),
                34 => Ok(&*(bytes.as_ptr() as *const SetAuthority)),
                _ => Err(ProgramError::InvalidInstructionData),
            }
        }
    }

    #[inline]
    pub fn authority_type(&self) -> Result<AuthorityType, ProgramError> {
        self.authority_type.try_into()
    }

    #[inline]
    pub fn new_authority(&self) -> Option<&Pubkey> {
        if self.new_authority.0 == 0 {
            Option::None
        } else {
            Option::Some(&self.new_authority.1)
        }
    }
}
