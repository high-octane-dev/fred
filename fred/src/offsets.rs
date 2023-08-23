use sunset::*;
use std::ffi::*;

#[from_offset(0x0068b173)]
pub extern "cdecl" fn operator_new(sz: usize) -> *mut c_void;

#[from_offset(0x0068ae53)]
pub extern "cdecl" fn operator_delete(ptr: *mut c_void);

#[from_offset(0x0068ef6a)]
pub extern "cdecl" fn _fopen(file_name: *const i8, mode: *const i8) -> *mut c_void;
