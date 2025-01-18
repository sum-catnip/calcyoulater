use std::cell::{OnceCell, RefCell};
use std::ffi::OsString;
use std::mem::zeroed;
use std::os::raw::c_void;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::ptr::{null, null_mut};
use std::str::FromStr;
use std::sync::{Arc, LazyLock, Mutex, OnceLock};
use std::{fs, panic};

use ilhook::x64::{CallbackOption, HookFlags, HookPoint, HookType, Hooker, Registers};
use windows::Win32::System::LibraryLoader::LoadLibraryA;
use windows::Win32::System::Memory::{IsBadReadPtr, IsBadStringPtrW};
use windows::Win32::System::ProcessStatus::{GetModuleInformation, MODULEINFO};
use windows::Win32::System::Threading::GetCurrentProcess;
use windows::{Win32::Foundation::*, Win32::System::SystemServices::*};
use windows::{Win32::UI::WindowsAndMessaging::MessageBoxA, core::*};

const WELL_PLAYED: &str = "WelPlyd!";
const COORD1: &str = "48°25'03.5";
const COORD2: &str = "11°43'24.7\0";

#[unsafe(no_mangle)]
#[allow(non_snake_case, unused_variables)]
pub unsafe extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: u32, _: *mut ()) -> bool {
    match call_reason {
        DLL_PROCESS_ATTACH => attach(),
        DLL_PROCESS_DETACH => (),
        _ => (),
    }

    true
}

fn checkpat(p: *const u8, pat: &[u8]) -> bool {
    for (i, x) in pat.iter().enumerate() {
        if unsafe { *p.add(i) != *x } {
            return false;
        }
    }
    return true;
}

fn sigscan(info: &MODULEINFO, pat: &[u8]) -> Option<*const u8> {
    let base = info.lpBaseOfDll as *const u8;
    for i in 0..info.SizeOfImage {
        let base = unsafe { base.add(i as usize) };
        if i == 0x62a20 {
            println!("memes");
        }
        if checkpat(base, pat) {
            return Some(base);
        }
    }

    None
}

fn leak_wide_string(s: OsString) -> *const u16 {
    // Convert to UTF-16 and add null terminator
    let mut wide: Vec<u16> = s.encode_wide().collect();
    wide.push(0);

    // Leak the memory and get raw pointer
    let boxed = wide.into_boxed_slice();
    Box::leak(boxed).as_ptr()
}

unsafe fn u16_ptr_to_string(ptr: *const u16) -> OsString {
    println!("start of string: {:?}", ptr);
    let len = (0..).take_while(|&i| *ptr.offset(i) != 0).count();
    let slice = std::slice::from_raw_parts(ptr, len);

    OsString::from_wide(slice)
}

unsafe extern "win64" fn on_check_sn(reg: *mut Registers, _: usize) {
    // rcx == current u16str
    // rdx = new u16str
    let readptr = *((*reg).rdx as *const *const u16);
    if IsBadReadPtr(Some(readptr as *const c_void), 20).into()
        && IsBadStringPtrW(PWSTR::from_raw(readptr as *mut u16), 20).into()
    {
        return;
    }
    let new = u16_ptr_to_string(readptr);
    let new_str = new.to_str().unwrap();
    println!("new: {}", new_str);
    if new == "DEADE11D" {
        let writeptr = *((*reg).rdx as *mut *mut u16);
        for (i, x) in WELL_PLAYED.chars().enumerate() {
            *writeptr.offset(i as isize) = x as u16;
        }
    } else if new == "3735937309" {
        let writeptr = *((*reg).rdx as *mut *mut u16);
        for (i, x) in COORD1.chars().enumerate() {
            *writeptr.offset(i as isize) = x as u16;
        }
    } else if new == "33653360435" {
        let writeptr = *((*reg).rdx as *mut *mut u16);
        for (i, x) in COORD2.chars().enumerate() {
            *writeptr.offset(i as isize) = x as u16;
        }
    } else if new == "11011110101011011110000100011101" {
        //let writeptr = *((*reg).rdx as *mut *mut u16);
        //for (i, x) in replaceme.chars().enumerate() {
        //    *writeptr.offset(i as isize) = x as u16;
        //}
    }
}

fn attach() {
    let module = unsafe { LoadLibraryA(s!("calcviewmodel.dll")) };
    let Ok(module) = module else {
        panic!("error getting calcviewmodel.dll handle")
    };

    let proc = unsafe { GetCurrentProcess() };
    let mut modinfo: MODULEINFO = unsafe { zeroed() };

    unsafe { GetModuleInformation(proc, module, &mut modinfo, size_of::<MODULEINFO>() as u32) }
        .expect("error getting calcviewmodel.dll module info");

    let pat: [u8; 19] = [
        0x48, 0x89, 0x5C, 0x24, 0x18, 0x57, 0x48, 0x83, 0xEC, 0x20, 0x48, 0x8B, 0xFA, 0x48, 0x8B,
        0xD9, 0x48, 0x3B, 0xCA,
    ];
    //let pat: [u8; 4] = [0x40, 0x53, 0x48, 0x83];

    let addr = sigscan(&modinfo, &pat).expect("pattern not found");
    let hooker = Hooker::new(
        addr as usize,
        HookType::JmpBack(on_check_sn),
        CallbackOption::None,
        0,
        HookFlags::empty(),
    );
    Box::leak(Box::new(unsafe { hooker.hook().unwrap() }));
}
