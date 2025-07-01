use {
    super::{account_state::AccountState, COption, Initializable, Transmutable},
    pinocchio::{program_error::ProgramError, pubkey::Pubkey},
};

/// Incinerator address.
pub const INCINERATOR_ID: Pubkey =
    pinocchio_pubkey::pubkey!("1nc1nerator11111111111111111111111111111111");

/// System program id.
const SYSTEM_PROGRAM_ID: Pubkey = pinocchio_pubkey::pubkey!("11111111111111111111111111111111");

/// Internal representation of a token account data.
#[repr(C)]
pub struct Account {
    /// The mint associated with this account
    pub mint: Pubkey,

    /// The owner of this account.
    pub owner: Pubkey,

    /// The amount of tokens this account holds.
    amount: [u8; 8],

    /// If `delegate` is `Some` then `delegated_amount` represents
    /// the amount authorized by the delegate.
    delegate: COption<Pubkey>,

    /// The account's state.
    state: u8,

    /// Indicates whether this account represents a native token or not.
    is_native: [u8; 4],

    /// If `is_native.is_some`, this is a native token, and the value logs the
    /// rent-exempt reserve. An Account is required to be rent-exempt, so
    /// the value is used by the Processor to ensure that wrapped SOL
    /// accounts do not drop below this threshold.
    native_amount: [u8; 8],

    /// The amount delegated.
    delegated_amount: [u8; 8],

    /// Optional authority to close the account.
    close_authority: COption<Pubkey>,
}

impl Account {
    #[inline(always)]
    pub fn set_account_state(&mut self, state: AccountState) {
        self.state = state as u8;
    }

    #[inline(always)]
    pub fn account_state(&self) -> Result<AccountState, ProgramError> {
        AccountState::try_from(self.state)
    }

    #[inline(always)]
    pub fn set_amount(&mut self, amount: u64) {
        self.amount = amount.to_le_bytes();
    }

    #[inline(always)]
    pub fn amount(&self) -> u64 {
        u64::from_le_bytes(self.amount)
    }

    #[inline(always)]
    pub fn clear_delegate(&mut self) {
        self.delegate.0[0] = 0;
    }

    #[inline(always)]
    pub fn set_delegate(&mut self, delegate: &Pubkey) {
        self.delegate.0[0] = 1;
        self.delegate.1 = *delegate;
    }

    #[inline(always)]
    pub fn delegate(&self) -> Option<&Pubkey> {
        if self.delegate.0[0] == 1 {
            Some(&self.delegate.1)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn set_native(&mut self, value: bool) {
        self.is_native[0] = value as u8;
    }

    #[inline(always)]
    pub fn is_native(&self) -> bool {
        self.is_native[0] == 1
    }

    #[inline(always)]
    pub fn set_native_amount(&mut self, amount: u64) {
        self.native_amount = amount.to_le_bytes();
    }

    #[inline(always)]
    pub fn native_amount(&self) -> Option<u64> {
        if self.is_native() {
            Some(u64::from_le_bytes(self.native_amount))
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn set_delegated_amount(&mut self, amount: u64) {
        self.delegated_amount = amount.to_le_bytes();
    }

    #[inline(always)]
    pub fn delegated_amount(&self) -> u64 {
        u64::from_le_bytes(self.delegated_amount)
    }

    #[inline(always)]
    pub fn clear_close_authority(&mut self) {
        self.close_authority.0[0] = 0;
    }

    #[inline(always)]
    pub fn set_close_authority(&mut self, value: &Pubkey) {
        self.close_authority.0[0] = 1;
        self.close_authority.1 = *value;
    }

    #[inline(always)]
    pub fn close_authority(&self) -> Option<&Pubkey> {
        if self.close_authority.0[0] == 1 {
            Some(&self.close_authority.1)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn is_frozen(&self) -> Result<bool, ProgramError> {
        AccountState::try_from(self.state).map(|state| state == AccountState::Frozen)
    }

    #[inline(always)]
    pub fn is_owned_by_system_program_or_incinerator(&self) -> bool {
        SYSTEM_PROGRAM_ID == self.owner || INCINERATOR_ID == self.owner
    }
}

unsafe impl Transmutable for Account {
    const LEN: usize = core::mem::size_of::<Account>();
}

impl Initializable for Account {
    #[inline(always)]
    fn is_initialized(&self) -> Result<bool, ProgramError> {
        AccountState::try_from(self.state).map(|state| state != AccountState::Uninitialized)
    }
}
