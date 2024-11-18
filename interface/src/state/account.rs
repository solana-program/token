use bytemuck::{Pod, Zeroable};
use pinocchio::pubkey::Pubkey;

use super::{PodCOption, PodU64};

/// Account data.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct Account {
    /// The mint associated with this account
    pub mint: Pubkey,

    /// The owner of this account.
    pub owner: Pubkey,

    /// The amount of tokens this account holds.
    pub amount: PodU64,

    /// If `delegate` is `Some` then `delegated_amount` represents
    /// the amount authorized by the delegate
    pub delegate: PodCOption<Pubkey>,

    /// The account's state
    pub state: u8,

    /// If is_native.is_some, this is a native token, and the value logs the
    /// rent-exempt reserve. An Account is required to be rent-exempt, so
    /// the value is used by the Processor to ensure that wrapped SOL
    /// accounts do not drop below this threshold.
    pub is_native: PodCOption<PodU64>,

    /// The amount delegated
    pub delegated_amount: PodU64,

    /// Optional authority to close the account.
    pub close_authority: PodCOption<Pubkey>,
}

impl Account {
    /// Size of the `Account` account.
    pub const LEN: usize = core::mem::size_of::<Self>();

    #[inline]
    pub fn is_initialized(&self) -> bool {
        self.state != AccountState::Uninitialized as u8
    }

    #[inline]
    pub fn is_frozen(&self) -> bool {
        self.state == AccountState::Frozen as u8
    }

    pub fn amount(&self) -> u64 {
        self.amount.into()
    }
}

/// Account state.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum AccountState {
    /// Account is not yet initialized
    #[default]
    Uninitialized,

    /// Account is initialized; the account owner and/or delegate may perform
    /// permitted operations on this account
    Initialized,

    /// Account has been frozen by the mint freeze authority. Neither the
    /// account owner nor the delegate are able to perform operations on
    /// this account.
    Frozen,
}

impl From<u8> for AccountState {
    fn from(value: u8) -> Self {
        match value {
            0 => AccountState::Uninitialized,
            1 => AccountState::Initialized,
            2 => AccountState::Frozen,
            _ => panic!("invalid account state value: {value}"),
        }
    }
}

impl From<AccountState> for u8 {
    fn from(value: AccountState) -> Self {
        match value {
            AccountState::Uninitialized => 0,
            AccountState::Initialized => 1,
            AccountState::Frozen => 2,
        }
    }
}
