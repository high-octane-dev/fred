use std::ffi::{c_void, CString};
use std::fs;

use windows_sys::Win32::UI::WindowsAndMessaging::*;
use windows_sys::Win32::{
    Foundation::*,
    System::{Diagnostics::Debug::*, LibraryLoader::*, Memory::*, Threading::*},
};

const DLL_PATH: &[u8] = b"fred.dll\0";

fn main() -> Result<(), ()> {
    unsafe {
        let dll_path_cstring = CString::new(&DLL_PATH[0..DLL_PATH.len() - 1]).unwrap();

        if !fs::metadata(dll_path_cstring.to_str().unwrap()).is_ok() {
            MessageBoxA(
                0,
                b"Failed to load fred.dll.\0".as_ptr() as *const u8,
                b"Error\0".as_ptr() as *const u8,
                MB_OK,
            );
        } else {
            let exe_path = CString::new("Cars_Steam_Unpacked.exe").unwrap();
            let mut s_info_a: STARTUPINFOA = std::mem::zeroed();
            let mut p_info = std::mem::zeroed();
            s_info_a.cb = std::mem::size_of::<STARTUPINFOA>() as u32;

            CreateProcessA(
                exe_path.as_ptr() as *const u8,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                1,
                CREATE_SUSPENDED,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut s_info_a,
                &mut p_info,
            );

            let allocated_str = VirtualAllocEx(
                p_info.hProcess,
                std::ptr::null_mut(),
                DLL_PATH.len(),
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE,
            );

            WriteProcessMemory(
                p_info.hProcess,
                allocated_str,
                DLL_PATH.as_ptr() as *const c_void,
                DLL_PATH.len(),
                std::ptr::null_mut(),
            );

            let dll_loader_thread: HANDLE = CreateRemoteThread(
                p_info.hProcess,
                std::ptr::null_mut(),
                0,
                std::mem::transmute::<*const (), LPTHREAD_START_ROUTINE>(LoadLibraryA as *const ()),
                allocated_str,
                0,
                std::ptr::null_mut(),
            );

            WaitForSingleObject(dll_loader_thread, INFINITE);
            VirtualFreeEx(p_info.hProcess, allocated_str, 0, MEM_RELEASE);
            CloseHandle(dll_loader_thread);

            ResumeThread(p_info.hThread);
            CloseHandle(p_info.hThread);
            CloseHandle(p_info.hProcess);

            return Ok(());
        }

        return Err(());
    };
}
