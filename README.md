<!-- cargo-rdme start -->

Generate traits from C header files.

When rewriting a C libary in Rust,
you often want to preserve the original C header files.

This is possible using this crate in conjuction with [`bindgen`](https://docs.rs/bindgen).

Take the following `C` header.

```c
```

In your `build.rs` script:
1. Use [`bindgen`](https://docs.rs/bindgen) to generate equivalent Rust blocks.
2. Use [`seesaw`] to generate a trait from those bindings.

```rust
// build.rs
```

The generated file will look like this:

```rust
```

And you can export the same ABI as the C library using [`no_mangle`],
which simply adds `#[no_mangle]` to each of the functions.

```rust
#[seesaw::no_mangle]
impl YakShaver for () { .. }
```

<!-- cargo-rdme end -->
