use std::ffi::c_void;

use winapi::um::processthreadsapi::GetCurrentThread;

use crate::hooking::detail::relocate_code;

use self::detail::find_suitable_backup_size;

mod detail;
mod legacy;
mod tests;

pub unsafe fn write_jmp(src: *mut u8, dst: *mut u8) -> Option<()> {
    let relative_address = (dst as u32) - (src as u32) - 5;

    crate::win::set_permission(src, 5, crate::win::Perm::ExecuteReadWrite)?;
    *src = 0xE9;
    *(src.add(1) as *mut u32) = relative_address;
    Some(())
}

pub unsafe fn write_call(src: *mut u8, dst: *mut u8) -> Option<()> {
    let relative_address = (dst as u32) - (src as u32) - 5;

    crate::win::set_permission(src, 5, crate::win::Perm::ExecuteReadWrite)?;
    *src = 0xE8;
    *(src.add(1) as *mut u32) = relative_address;
    Some(())
}

pub unsafe fn write_push(src: *mut u8, dst: u32) -> Option<()> {
    crate::win::set_permission(src, 5, crate::win::Perm::ExecuteReadWrite)?;
    *src = 0xE8;
    *(src.add(1) as *mut u32) = dst;
    Some(())
}

pub unsafe fn write_nop(addr: *mut u8, code_size: usize) -> Option<()> {
    crate::win::set_permission(addr, code_size, crate::win::Perm::ExecuteReadWrite)?;
    std::ptr::write_bytes(addr, 0x90, code_size);
    Some(())
}

#[repr(C, packed)]
pub union Register {
    pub pointer: *mut (),
    pub unsigned_integer: u32,
    pub signed_integer: i32,
    pub floating_point: f32,
}

#[repr(C, packed)]
pub struct InlineCtx {
    pub eflags: Register,
    pub edi: Register,
    pub esi: Register,
    pub ebp: Register,
    pub esp: Register,
    pub ebx: Register,
    pub edx: Register,
    pub ecx: Register,
    pub eax: Register,
}

type CallbackFuncPtr = extern "cdecl" fn(InlineCtx);

#[derive(Debug)]
pub enum InlineHookErr {
    // The size of the code at the desired address cannot be made large enough to fit a 5-byte JMP.
    InvalidCodeSize,

    // Failed to relocate code from the source to a trampoline.
    FailedToRelocateCode,
}

pub unsafe fn inline_hook(ptr: usize, callback: CallbackFuncPtr) -> Result<(), InlineHookErr> {
    // Calculate the minimum bytes needed to be backed up, and an upper-bound limit of how many bytes the relocated code could take. (Used for below allocation)
    let (original_code_len, padded_code_len) = find_suitable_backup_size(ptr as *const u8);

    if original_code_len < 5 {
        Err(InlineHookErr::InvalidCodeSize)
    } else {
        let jit_area = crate::win::allocate_code::<u8>(9 + padded_code_len + 5);
        // Build inline handler.
        jit_area[0] = 0x60; // pushad
        jit_area[1] = 0x9C; // pushfd
        write_call(jit_area.as_mut_ptr().offset(2), callback as *mut u8).unwrap(); // call callback
        jit_area[7] = 0x9D; // popfd
        jit_area[8] = 0x61; // popad

        // Attempt to build/relocate the code, and if successful, copy into the trampoline.
        match relocate_code(
            ptr as usize,
            original_code_len,
            jit_area.as_ptr().offset(9) as usize,
        ) {
            Ok(relocated) => {
                jit_area[9..9 + relocated.len()].copy_from_slice(&relocated);

                let old_perm = crate::win::set_permission(
                    ptr as *mut u8,
                    original_code_len,
                    crate::win::Perm::ExecuteReadWrite,
                )
                .unwrap();

                // Insert jmp from the inline handler back to the original function.
                write_jmp(
                    jit_area.as_mut_ptr().offset((9 + relocated.len()) as isize),
                    (ptr + 5) as *mut u8,
                )
                .unwrap();

                // Ensure original function has the trampoline area nop'd out.
                write_nop(ptr as *mut u8, original_code_len);

                // Insert jmp from the source to the inline handler.
                write_jmp(ptr as *mut u8, jit_area.as_mut_ptr());

                // Reset the permission at the source.
                crate::win::set_permission(ptr as *mut u8, original_code_len, old_perm).unwrap();

                Ok(())
            }
            Err(err) => {
                dbg!(err);
                Err(InlineHookErr::FailedToRelocateCode)
            }
        }
    }
}

pub unsafe fn replace_hook<T>(ptr: *mut *mut T, callback: *mut T) {
    detours_sys::DetourTransactionBegin();
    detours_sys::DetourUpdateThread(GetCurrentThread() as _);
    detours_sys::DetourAttach(ptr as *mut *mut c_void, callback as *mut c_void);
    detours_sys::DetourTransactionCommit();
}
