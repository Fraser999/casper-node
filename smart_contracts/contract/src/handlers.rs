//! Contains definitions for panic and allocation error handlers, along with other `no_std` support
//! code.
#[cfg(feature = "test-support")]
use crate::contract_api::runtime;
#[cfg(feature = "test-support")]
use alloc::format;

/// A panic handler for use in a `no_std` environment.
#[panic_handler]
#[no_mangle]
pub fn panic(_info: &::core::panic::PanicInfo) -> ! {
    #[cfg(feature = "test-support")]
    runtime::print(&format!("Panic: {}", _info));
    loop {}
}
