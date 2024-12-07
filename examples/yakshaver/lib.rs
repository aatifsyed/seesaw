#![allow(non_camel_case_types)]
include!("generated/bindgen.rs");
include!("generated/seesaw.rs");

use std::ffi::{c_int, c_uint};

/// Implement `yakshaver.h` by just storing an integer in the pointer.
#[seesaw::no_mangle]
impl Yakshaver for () {
    unsafe extern "C" fn create() -> *mut yakshaver {
        0usize as _
    }
    unsafe extern "C" fn destroy(_: *mut yakshaver) {}
    unsafe extern "C" fn yaks_shaved(ptr: *const yakshaver) -> c_uint {
        *(ptr as *const usize) as _
    }
    unsafe extern "C" fn shave(ptr: *mut yakshaver) -> c_int {
        *(ptr as *mut usize) += 1;
        0
    }
}
