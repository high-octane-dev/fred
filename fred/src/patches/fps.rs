use crate::hooking::write_push;

pub fn init() {
    unsafe {
        write_push(0x0070c420 as *mut u8, 60.0_f32.to_bits());
        write_push(0x0070c425 as *mut u8, 60.0_f32.to_bits());
    };
}