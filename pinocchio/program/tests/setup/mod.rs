use solana_pubkey::Pubkey;

#[allow(dead_code)]
pub mod account;
#[allow(dead_code)]
pub mod mint;
#[allow(dead_code)]
pub mod mollusk;

pub const TOKEN_PROGRAM_ID: Pubkey = Pubkey::new_from_array(pinocchio_token_interface::program::ID);
