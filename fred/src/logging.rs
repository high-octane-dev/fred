use libc_stdhandle::stdout;
use once_cell::sync::Lazy;
use winapi::um::{consoleapi::AllocConsole, wincon::SetConsoleTitleA};

static mut GLOBAL_CONSOLE: Lazy<Console> = Lazy::new(|| Console::new());

struct Console {
    console_file: *mut libc::FILE,
}

impl Console {
    pub fn new() -> Self {
        Self {
            console_file: unsafe {
                AllocConsole();
                let file_name = std::ffi::CString::new("CONOUT$").unwrap();
                let mode = std::ffi::CString::new("w").unwrap();
                libc::freopen(file_name.as_ptr(), mode.as_ptr(), stdout())
            },
        }
    }
}

impl Drop for Console {
    fn drop(&mut self) {
        unsafe {
            if !self.console_file.is_null() {
                libc::fclose(self.console_file);
            }
        };
    }
}

pub fn init() {
    Lazy::<Console>::force(unsafe { &GLOBAL_CONSOLE });
    unsafe {
        let console_title = std::ffi::CString::new("Fred Console").unwrap();
        SetConsoleTitleA(console_title.as_ptr());
    }
}
