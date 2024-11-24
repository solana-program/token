pub mod account;
pub mod account_state;
pub mod mint;
pub mod multisig;

/// Type alias for fields represented as `COption`.
pub type COption<T> = ([u8; 4], T);
