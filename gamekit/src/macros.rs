//!
//! macros
//!

/// Include binary data with defined memory alignment.
#[macro_export]
macro_rules! include_bytes_aligned {
    ($align_to:expr, $path:expr) => {{
        #[repr(C, align($align_to))]
        struct __Aligned<T: ?Sized>(T);
        static __DATA: &'static __Aligned<[u8]> = &__Aligned(*include_bytes!($path));
        &__DATA.0
    }};
}

/// Include binary resource data with a 4-byte memory alignment.
#[macro_export]
macro_rules! include_resource {
    ($file:expr) => {
        gamekit::include_bytes_aligned!(4, $file)
    };
}
