//! Another ERC20-like Token program for the Solana blockchain.

#![no_std]

#[cfg(not(feature = "runtime-verification"))]
#[path = "entrypoint.rs"]
mod entrypoint;

#[cfg(feature = "runtime-verification")]
#[path = "entrypoint-runtime-verification.rs"]
mod entrypoint;
mod processor;

// Include stubs for non-Solana platforms
#[cfg(not(target_os = "solana"))]
mod stubs;
