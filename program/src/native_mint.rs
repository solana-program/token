//! The Mint that represents the native token

/// There are `10^9` lamports in one SOL
pub const DECIMALS: u8 = 9;

// The Mint for native SOL Token accounts
solana_pubkey::declare_id!("So11111111111111111111111111111111111111112");

#[cfg(test)]
mod tests {
    use {super::*, solana_native_token::sol_str_to_lamports};

    #[test]
    fn test_decimals() {
        assert_eq!(
            sol_str_to_lamports("42.").unwrap(),
            crate::try_ui_amount_into_amount("42.".to_string(), DECIMALS).unwrap()
        );
    }
}
