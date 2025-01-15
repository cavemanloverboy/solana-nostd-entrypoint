#![no_std]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

pub mod entrypoint_nostd;
pub use entrypoint_nostd::*;
pub mod __private {
    pub use solana_msg::sol_log;
}

#[macro_export]
macro_rules! noalloc_allocator {
    () => {
        pub mod allocator {
            pub struct NoAlloc;
            extern crate alloc;
            unsafe impl alloc::alloc::GlobalAlloc for NoAlloc {
                #[inline]
                unsafe fn alloc(
                    &self,
                    _: core::alloc::Layout,
                ) -> *mut u8 {
                    panic!("no_alloc :)");
                }
                #[inline]
                unsafe fn dealloc(
                    &self,
                    _: *mut u8,
                    _: core::alloc::Layout,
                ) {
                }
            }

            #[cfg(target_os = "solana")]
            #[global_allocator]
            static A: NoAlloc = NoAlloc;
        }
    };
}

#[macro_export]
macro_rules! basic_panic_impl {
    () => {
        #[cfg(target_os = "solana")]
        #[no_mangle]
        fn custom_panic(_info: &core::panic::PanicInfo<'_>) {
            $crate::__private::sol_log("panicked!");
        }
    };
}
