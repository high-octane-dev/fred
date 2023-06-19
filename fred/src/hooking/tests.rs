#[cfg(test)]
mod tests {
    use crate::hooking;

    static mut ADD_FUNC_PTR: *mut extern "thiscall" fn(*mut i32, i32) -> i32 = unsafe {
        std::mem::transmute::<_, *mut extern "thiscall" fn(x: *mut i32, y: i32) -> i32>(
            add as *const (),
        )
    };

    unsafe extern "thiscall" fn add(x: *mut i32, y: i32) -> i32 {
        println!("Hello from original function!");
        *x = *x + y;
        *x
    }

    unsafe extern "thiscall" fn add_hook(x: *mut i32, y: i32) -> i32 {
        println!("Hello from function hook!");
        std::mem::transmute::<_, extern "thiscall" fn(*mut i32, i32) -> i32>(ADD_FUNC_PTR)(x, y)
    }

    #[test]
    fn test_replacement() {
        assert_eq!(std::mem::size_of::<hooking::Register>(), 4);
        unsafe {
            hooking::replace_hook(&mut ADD_FUNC_PTR, add_hook as *mut _);

            let mut x = 33;
            let y = 1;
            let result = add(&mut x as *mut i32, y);
            assert_eq!(result, 34);
        }
    }
}
