#![feature(generic_arg_infer)]
#![feature(allocator_api)]

pub mod allocator;

#[cfg(feature = "C")]
pub mod ffi;

#[cfg(test)]
pub mod tests;
