use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::{Pubkey, PUBKEY_BYTES},
    ProgramResult,
};
use token_interface::{
    error::TokenError,
    instruction::AuthorityType,
    state::{account::Account, mint::Mint, PodCOption},
};

use super::validate_owner;

pub fn process_set_authority(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    authority_type: AuthorityType,
    new_authority: Option<&Pubkey>,
) -> ProgramResult {
    let [account_info, authority_info, remaning @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if account_info.data_len() == Account::LEN {
        let account = bytemuck::try_from_bytes_mut::<Account>(unsafe {
            account_info.borrow_mut_data_unchecked()
        })
        .map_err(|_error| ProgramError::InvalidAccountData)?;

        if account.is_frozen() {
            return Err(TokenError::AccountFrozen.into());
        }

        match authority_type {
            AuthorityType::AccountOwner => {
                validate_owner(program_id, &account.owner, authority_info, remaning)?;

                if let Some(authority) = new_authority {
                    account.owner = *authority;
                } else {
                    return Err(TokenError::InvalidInstruction.into());
                }

                account.delegate.clear();
                account.delegated_amount = 0.into();

                if account.is_native.is_some() {
                    account.close_authority.clear();
                }
            }
            AuthorityType::CloseAccount => {
                let authority = account.close_authority.as_ref().unwrap_or(&account.owner);
                validate_owner(program_id, authority, authority_info, remaning)?;
                account.close_authority = PodCOption::from(new_authority.copied());
            }
            _ => {
                return Err(TokenError::AuthorityTypeNotSupported.into());
            }
        }
    } else if account_info.data_len() == Mint::LEN {
        let mint = bytemuck::try_from_bytes_mut::<Mint>(unsafe {
            account_info.borrow_mut_data_unchecked()
        })
        .map_err(|_error| ProgramError::InvalidAccountData)?;

        match authority_type {
            AuthorityType::MintTokens => {
                // Once a mint's supply is fixed, it cannot be undone by setting a new
                // mint_authority
                let mint_authority = mint
                    .mint_authority
                    .as_ref()
                    .ok_or(TokenError::FixedSupply)?;

                validate_owner(program_id, mint_authority, authority_info, remaning)?;
                mint.mint_authority = PodCOption::from(new_authority.copied());
            }
            AuthorityType::FreezeAccount => {
                // Once a mint's freeze authority is disabled, it cannot be re-enabled by
                // setting a new freeze_authority
                let freeze_authority = mint
                    .freeze_authority
                    .as_ref()
                    .ok_or(TokenError::MintCannotFreeze)?;

                validate_owner(program_id, freeze_authority, authority_info, remaning)?;
                mint.freeze_authority = PodCOption::from(new_authority.copied());
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

/// Instruction data for the `InitializeMint` instruction.
pub struct SetAuthority<'a> {
    pub authority_type: AuthorityType,

    /// New authority.
    pub new_authority: Option<&'a Pubkey>,
}

impl<'a> SetAuthority<'a> {
    pub fn try_from_bytes(data: &'a [u8]) -> Result<Self, ProgramError> {
        // We expect the data to be at least the size of the u8 (authority_type)
        // plus one byte for the authority option.
        if data.len() <= 2 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let (authority_type, remaining) = data.split_at(1);

        let new_authority = match remaining.split_first() {
            Some((&0, _)) => None,
            Some((&1, pubkey)) if pubkey.len() == PUBKEY_BYTES => {
                Some(bytemuck::from_bytes::<Pubkey>(pubkey))
            }
            _ => return Err(ProgramError::InvalidInstructionData),
        };

        Ok(Self {
            authority_type: AuthorityType::from(authority_type[0]),
            new_authority,
        })
    }
}
