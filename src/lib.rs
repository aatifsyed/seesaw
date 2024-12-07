//! Generate traits from C header files.
//!
//! When rewriting a C libary in Rust,
//! you often want to preserve the original C header files.
//!
//! This is possible using this crate in conjuction with [`bindgen`](https://docs.rs/bindgen).
//!
//! Take the following `C` header.
//!
//! ```c
#![doc = include_str!("../examples/yakshaver/yakshaver.h")]
//! ```
//!
//! In your `build.rs` script:
//! 1. Use [`bindgen`](https://docs.rs/bindgen) to generate equivalent Rust blocks.
//! 2. Use [`seesaw`] to generate a trait from those bindings.
//!
//! ```
//! // build.rs
#![doc = include_str!("../examples/yakshaver/build.rs")]
//! ```
//!
//! The generated file will look like this:
//!
//! ```
#![doc = include_str!("../examples/yakshaver/generated/seesaw.rs")]
//! ```
//!
//! And you can export the same ABI as the C library using [`no_mangle`],
//! which simply adds `#[no_mangle]` to each of the functions.
//!
//! ```
//! # const _: () = stringify! {
//! #[seesaw::no_mangle]
//! impl YakShaver for () { .. }
//! # }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "macros")]
#[doc(inline)]
pub use seesaw_macros::no_mangle;

#[cfg(feature = "build")]
pub use imp::{seesaw, Destination, Trait, TraitSet};

#[cfg(feature = "build")]
mod imp;
