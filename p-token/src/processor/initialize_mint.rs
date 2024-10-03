use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use pinocchio::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::{Pubkey, PUBKEY_BYTES},
};

use crate::{
    error::TokenError,
    state::{mint::Mint, PodCOption},
};

pub fn process_initialize_mint(
    accounts: &[AccountInfo],
    args: &InitializeMint,
    _rent_sysvar_account: bool,
) -> ProgramResult {
    let [mint_info, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let mint_data = &mut mint_info.try_borrow_mut_data()?;
    let mint = bytemuck::from_bytes_mut::<Mint>(mint_data);

    if mint.is_initialized.into() {
        return Err(TokenError::AlreadyInUse.into());
    }

    // FEBO: ~408 CU can be saved by removing the rent check (is_exempt seems to
    // be very expensive).
    //
    // The transaction will naturally fail if the account is not rent exempt with
    // a TransactionError::InsufficientFundsForRent error.
    /*
    let rent = Rent::get()?;

    if !rent.is_exempt(
        unsafe { *mint_info.unchecked_borrow_lamports() },
        size_of::<Mint>(),
    ) {
        return Err(TokenError::NotRentExempt);
    }
    */

    mint.mint_authority = PodCOption::from(Some(args.data.mint_authority));
    mint.decimals = args.data.decimals;
    mint.is_initialized = true.into();

    if let Some(freeze_authority) = args.freeze_authority {
        mint.freeze_authority = PodCOption::from(Some(*freeze_authority));
    }

    Ok(())
}

/// Instruction data for the `InitializeMint` instruction.
pub struct InitializeMint<'a> {
    pub data: &'a MintData,

    /// The freeze authority/multisignature of the mint.
    pub freeze_authority: Option<&'a Pubkey>,
}

impl<'a> InitializeMint<'a> {
    pub fn try_from_bytes(data: &'a [u8]) -> Result<Self, ProgramError> {
        // We expect the data to be at least the size of the MintInput struct
        // plus one byte for the freeze_authority option.
        if data.len() <= size_of::<MintData>() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let (data, remaining) = data.split_at(size_of::<MintData>());
        let data = bytemuck::from_bytes::<MintData>(data);

        let freeze_authority = match remaining.split_first() {
            Some((&0, _)) => None,
            Some((&1, pubkey)) if pubkey.len() == PUBKEY_BYTES => {
                Some(bytemuck::from_bytes::<Pubkey>(pubkey))
            }
            _ => return Err(ProgramError::InvalidInstructionData),
        };

        Ok(Self {
            data,
            freeze_authority,
        })
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
pub struct MintData {
    /// Number of base 10 digits to the right of the decimal place.
    pub decimals: u8,

    /// The authority/multisignature to mint tokens.
    pub mint_authority: Pubkey,
}
