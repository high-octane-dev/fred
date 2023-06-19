use super::*;

unsafe fn inline_replace(src: *mut u8, dst: *mut u8, size: usize) {
    write_nop(src, size);
    write_call(src, dst);
}

unsafe fn inline_replace_jump(src: *mut u8, dst: *mut u8, size: usize) {
    write_nop(src, size);
    write_jmp(src, dst);
}
