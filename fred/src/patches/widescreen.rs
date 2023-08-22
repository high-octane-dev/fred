use sunset::utils::*;

pub fn init() {
    unsafe {
        set_permission(0x007406e0 as *mut u8, 4, Perm::ReadWrite);
        set_permission(0x00743a18 as *mut u8, 4, Perm::ReadWrite);

        *(0x007406e0 as *mut f32) = 1.0 / 1280.0;
        *(0x00743a18 as *mut f32) = 1.0 / 720.0;
    }
}
