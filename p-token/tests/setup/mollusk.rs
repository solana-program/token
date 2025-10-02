use mollusk_svm::Mollusk;
use pinocchio_token_interface::state::{load_mut_unchecked, mint::Mint};
use solana_account::Account;
use solana_pubkey::Pubkey;
use solana_rent::Rent;
use solana_sdk_ids::bpf_loader_upgradeable;

use crate::setup::TOKEN_PROGRAM_ID;

pub fn create_mint_account(
    mint_authority: Pubkey,
    freeze_authority: Option<Pubkey>,
    decimals: u8,
    program_owner: &Pubkey,
) -> Account {
    let space = size_of::<Mint>();
    let lamports = Rent::default().minimum_balance(space);

    let mut data: Vec<u8> = vec![0u8; space];
    let mint = unsafe { load_mut_unchecked::<Mint>(data.as_mut_slice()).unwrap() };
    mint.set_mint_authority(mint_authority.as_array());
    if let Some(freeze_authority) = freeze_authority {
        mint.set_freeze_authority(freeze_authority.as_array());
    }
    mint.set_initialized();
    mint.decimals = decimals;

    Account {
        lamports,
        data,
        owner: *program_owner,
        executable: false,
        ..Default::default()
    }
}

/// Creates a Mollusk instance with the default feature set.
pub fn mollusk() -> Mollusk {
    let mut mollusk = Mollusk::default();
    mollusk.add_program(
        &TOKEN_PROGRAM_ID,
        "pinocchio_token_program",
        &bpf_loader_upgradeable::id(),
    );
    mollusk
}
