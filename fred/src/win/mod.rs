use winapi::{
    shared::minwindef::DWORD,
    um::{
        memoryapi::{VirtualAlloc, VirtualFree, VirtualProtect},
        winnt::*,
    },
};

#[repr(u32)]
pub enum Perm {
    None = PAGE_NOACCESS,
    Read = PAGE_READONLY,
    ReadWrite = PAGE_READWRITE,
    WriteCopy = PAGE_WRITECOPY,
    Execute = PAGE_EXECUTE,
    ExecuteRead = PAGE_EXECUTE_READ,
    ExecuteReadWrite = PAGE_EXECUTE_READWRITE,
    ExecuteWriteCopy = PAGE_EXECUTE_WRITECOPY,
    Guard = PAGE_GUARD,
    NoCache = PAGE_NOCACHE,
    WriteCombine = PAGE_WRITECOMBINE,
}

// Sets the desired permission on the memory block.
pub unsafe fn set_permission(ptr: *mut u8, size: usize, perm: Perm) -> Option<Perm> {
    let mut old_perm: Perm = Perm::None;
    let success = unsafe {
        VirtualProtect(
            std::mem::transmute(ptr),
            size,
            perm as DWORD,
            std::mem::transmute(&mut old_perm),
        )
    };
    if success != 0 {
        Some(old_perm)
    } else {
        None
    }
}

pub unsafe fn allocate_code<T: Copy + Sized>(size: usize) -> &'static mut [T] {
    unsafe {
        let ptr = VirtualAlloc(
            std::ptr::null_mut(),
            std::mem::size_of::<T>() * size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        ) as *mut T;
        std::slice::from_raw_parts_mut(ptr, size)
    }
}

pub unsafe fn deallocate_code<T: Copy + Sized>(ptr: *mut T) -> Option<()> {
    unsafe {
        if VirtualFree(ptr as *mut winapi::ctypes::c_void, 0, MEM_RELEASE) != 0 {
            Some(())
        } else {
            None
        }
    }
}
