extern crate core;
extern crate log;

// Logging traces are helpful while developing and testing, but we do not want
// to pay the cost (or expose internals) in WASM builds. Use this macro instead
// of `log::trace!` directly so the calls compile to nothing on WASM targets.
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        #[cfg(not(target_arch = "wasm32"))]
        {
            log::trace!($($arg)*);
        }
    };
}

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

pub mod ast;
pub mod link;
pub mod runtime;
pub mod tokenizer;
pub mod typesystem;
pub mod utils;

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
pub mod wasm;

pub mod test_support;
