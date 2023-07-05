use std::os::{raw::c_void, windows::prelude::MetadataExt};

use winapi::um::minwinbase::CRITICAL_SECTION;

use crate::{hooking::*, offsets::*};

#[repr(packed)]
pub struct FileInfo {
    pub path: [u8; 100],
    pub offset: u32,
    pub size: u32,
}

#[repr(packed)]
pub struct LoadedFile {
    pub pak_file_pointer: *mut c_void,
    pub file_info: *mut FileInfo,
    pub loaded_pak: *const LoadedPak,
    pub seek: u32,
}

#[repr(packed)]
pub struct LoadedPak {
    pub path: [u8; 100],
    pub files_in_pak: usize, // *mut FileInfo,
    pub file_count: u32,
    pub file_data_start_offset: u32,
}

#[repr(packed)]
pub struct PakSystem {
    pub loaded_paks: *mut LoadedPak,
    pub pak_count: u32,
    pub unk: u32,
    pub lock: CRITICAL_SECTION,
}

// Used to ensure that the game can access a LoadedPak's file_data_start_offset from a LoadedFile.
// (The game does this at 0x00687f20 in the Steam version of the game)
static DUMMY_LOADED_PAK: LoadedPak = LoadedPak {
    path: [0; 100],
    files_in_pak: 0,
    file_count: 0,
    file_data_start_offset: 0,
};

unsafe extern "thiscall" fn load_file_by_path(
    _this: *mut PakSystem,
    path: *const i8,
    _unk: u32,
) -> *mut LoadedFile {
    let file = operator_new(std::mem::size_of::<LoadedFile>()) as *mut LoadedFile;
    (*file).pak_file_pointer = std::ptr::null_mut();
    (*file).file_info = std::ptr::null_mut();
    (*file).loaded_pak = std::ptr::null_mut();
    (*file).seek = 0;

    let sanitized_name = std::ffi::CString::from_raw(path as *mut i8)
        .into_string()
        .unwrap()
        .replace("/", "\\");
    let sanitized_path = std::path::Path::new(&sanitized_name);
    if sanitized_path.exists() && sanitized_path.is_file() {
        println!(
            "[fred::load_file_by_path] loading file: {}...",
            sanitized_name
        );
        let mode = std::ffi::CString::new("rb").unwrap();
        (*file).pak_file_pointer = _fopen(path, mode.as_ptr());
        // Here, we allocate a new dummy FileInfo to ensure that the game can access its "offset" which we set to zero as, of course, modded files not inside a Pak don't need one.
        // We do, however, give it an actual proper file size. This is why we need to allocate it ourselves, since it must be unique.
        // (The game accesses this at 0x00687f20 in the Steam version of the game)

        let file_info = operator_new(std::mem::size_of::<FileInfo>()) as *mut FileInfo;
        (*file_info).path = [0; 100];
        (*file_info).offset = 0;
        (*file_info).size = std::fs::metadata(sanitized_path).unwrap().file_size() as u32;

        (*file).file_info = file_info;
        (*file).loaded_pak = &DUMMY_LOADED_PAK;

        file
    } else {
        operator_delete(file as *mut c_void);
        std::ptr::null_mut()
    }
}

// Stubs the function responsible for mounting Pak files so they are no longer necessary.
unsafe extern "thiscall" fn mount_pak_file(_this: *mut PakSystem, _path: *const i8) {}

// Hooks the function responsible for freeing LoadedFiles so our FileInfos get properly free'd.
extern "cdecl" fn free_dummy_file_info(ctx: &mut InlineCtx) {
    unsafe {
        let loaded_file = ctx.edi.pointer as *mut LoadedFile;
        operator_delete((*loaded_file).file_info as *mut c_void);
    }
}

static mut LOAD_FILE_BY_PATH_FUNC_PTR: *mut extern "thiscall" fn(
    *mut PakSystem,
    *const i8,
    u32,
) -> *mut LoadedFile = unsafe {
    std::mem::transmute::<
        _,
        *mut extern "thiscall" fn(*mut PakSystem, *const i8, u32) -> *mut LoadedFile,
    >(0x00687d90 as *const ())
};
static mut MOUNT_PAK_FILE_FUNC_PTR: *mut extern "thiscall" fn(*mut PakSystem, *const i8) = unsafe {
    std::mem::transmute::<_, *mut extern "thiscall" fn(*mut PakSystem, *const i8)>(
        0x00687a60 as *const (),
    )
};

pub fn init() {
    unsafe {
        replace_hook(&mut LOAD_FILE_BY_PATH_FUNC_PTR, load_file_by_path as _);
        replace_hook(&mut MOUNT_PAK_FILE_FUNC_PTR, mount_pak_file as _);
        inline_hook(0x00687f09, free_dummy_file_info).unwrap();
    }
}
