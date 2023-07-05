#![feature(abi_thiscall)]
#![feature(const_transmute_copy)]
#![feature(once_cell)]
#![allow(dead_code)]

use std::ffi::*;

mod hooking;
mod logging;
mod offsets;
mod patches;
mod win;

use winapi::{
    shared::minwindef::{DWORD, HINSTANCE},
    um::{winnt::DLL_PROCESS_ATTACH, libloaderapi::LoadLibraryA},
};

#[no_mangle]
extern "stdcall" fn DllMain(_h_inst_dll: HINSTANCE, fdw_reason: DWORD, _lpv_reserved: *mut c_void) {
    if fdw_reason == DLL_PROCESS_ATTACH {
        logging::init();
        patches::fs::init();
        // patches::widescreen::init();
        if let Ok(entries) = std::fs::read_dir("./plugins/") {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if !path.is_dir() {
                        if let Some(extension) = path.extension() {
                            if extension.to_str().unwrap() == "dll" {
                                let c_str = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
                                println!("[fred::DllMain] loading plugin: {}...", path.file_name().unwrap().to_str().unwrap());
                                unsafe { LoadLibraryA(c_str.as_ptr()) };
                            }
                        }
                    }
                }
            }
        }
    }
}
