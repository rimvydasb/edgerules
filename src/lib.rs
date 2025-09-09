extern crate core;
extern crate log;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

mod ast;
mod link;
pub mod runtime;
mod tokenizer;
mod typesystem;
pub mod utils;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
