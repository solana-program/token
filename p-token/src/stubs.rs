// Stubs for Solana functions when building on non-Solana platforms
// These are only used for generating SMIR, not for actual execution

#[cfg(not(target_os = "solana"))]
#[no_mangle]
pub unsafe extern "C" fn sol_memcpy_(dst: *mut u8, src: *const u8, n: usize) {
    // Simple memcpy implementation for non-Solana builds
    // This is only used for SMIR generation, not actual execution
    if !dst.is_null() && !src.is_null() && n > 0 {
        core::ptr::copy_nonoverlapping(src, dst, n);
    }
}