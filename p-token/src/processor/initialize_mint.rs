use core::{marker::PhantomData, mem::size_of};
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use token_interface::{
    error::TokenError,
    state::{load_mut_unchecked, mint::Mint, Initializable},
};

#[inline(always)]
pub fn process_initialize_mint(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
    rent_sysvar_account: bool,
) -> ProgramResult {
    // Validates the instruction data.

    let args = InitializeMint::try_from_bytes(instruction_data)?;

    // Validates the accounts.

    let (mint_info, rent_sysvar_info) = if rent_sysvar_account {
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
        // SAFETY: single immutable borrow to `rent_sysvar_info`; account ID and length are
        // checked by `from_account_info_unchecked`.
        let rent = unsafe { Rent::from_account_info_unchecked(rent_sysvar_info)? };
        rent.is_exempt(mint_info.lamports(), size_of::<Mint>())
    } else {
        Rent::get()?.is_exempt(mint_info.lamports(), size_of::<Mint>())
    };

    if !is_exempt {
        return Err(TokenError::NotRentExempt.into());
    }

    // Initialize the mint.

    mint.set_initialized(true);
    mint.set_mint_authority(args.mint_authority());
    mint.decimals = args.decimals();

    if let Some(freeze_authority) = args.freeze_authority() {
        mint.set_freeze_authority(freeze_authority);
    }

    Ok(())
}

/// Instruction data for the `InitializeMint` instruction.
pub struct InitializeMint<'a> {
    raw: *const u8,

    _data: PhantomData<&'a [u8]>,
}

impl InitializeMint<'_> {
    #[inline]
    pub fn try_from_bytes(bytes: &[u8]) -> Result<InitializeMint, ProgramError> {
        // The minimum expected size of the instruction data.
        // - decimals (1 byte)
        // - mint_authority (32 bytes)
        // - option + freeze_authority (1 byte + 32 bytes)
        if bytes.len() < 34 || (bytes[33] == 1 && bytes.len() < 66) {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(InitializeMint {
            raw: bytes.as_ptr(),
            _data: PhantomData,
        })
    }

    #[inline]
    pub fn decimals(&self) -> u8 {
        // SAFETY: the `bytes` length was validated in `try_from_bytes`.
        unsafe { *self.raw }
    }

    #[inline]
    pub fn mint_authority(&self) -> &Pubkey {
        // SAFETY: the `bytes` length was validated in `try_from_bytes`.
        unsafe { &*(self.raw.add(1) as *const Pubkey) }
    }

    #[inline]
    pub fn freeze_authority(&self) -> Option<&Pubkey> {
        // SAFETY: the `bytes` length was validated in `try_from_bytes`.
        unsafe {
            if *self.raw.add(33) == 0 {
                Option::None
            } else {
                Option::Some(&*(self.raw.add(34) as *const Pubkey))
            }
        }
    }
}
