use bytemuck::{Pod, Zeroable};
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::{Pubkey, PUBKEY_BYTES},
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use std::mem::size_of;
use token_interface::{
    error::TokenError,
    state::{mint::Mint, PodCOption},
};

pub fn process_initialize_mint(
    accounts: &[AccountInfo],
    args: &InitializeMint,
    _rent_sysvar_account: bool,
) -> ProgramResult {
    // Validate the mint account.

    let [mint_info, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let mint_data = unsafe { mint_info.borrow_mut_data_unchecked() };
    let mint = bytemuck::try_from_bytes_mut::<Mint>(mint_data)
        .map_err(|_error| ProgramError::InvalidAccountData)?;

    if mint.is_initialized.into() {
        return Err(TokenError::AlreadyInUse.into());
    }

    // Check rent-exempt status of the mint account.
    //
    // WIP: This is an expensive check (~400 CU) since it involves floating-point
    // operations. Currently we are using a 'scaled' version, which is faster but
    // not as precise as the `f64` version when there are decimal places.

    let rent = Rent::get()?;

    if !rent.is_exempt_scaled(mint_info.lamports(), size_of::<Mint>()) {
        return Err(TokenError::NotRentExempt.into());
    }

    // Initialize the mint.

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

/// Base information for the mint.
#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
pub struct MintData {
    /// Number of base 10 digits to the right of the decimal place.
    pub decimals: u8,

    /// The authority/multisignature to mint tokens.
    pub mint_authority: Pubkey,
}
