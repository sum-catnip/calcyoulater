#![windows_subsystem = "windows"]

use std::{ffi::OsStr, os::raw::c_void, ptr::null, time::Duration};

use dll_syringe::{process::OwnedProcess, Syringe};
use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System, UpdateKind};

use windows::Win32::{
    Foundation::{GENERIC_EXECUTE, GENERIC_READ},
    Security::{CreateWellKnownSid, WinBuiltinAnyPackageSid, PSID, SECURITY_MAX_SID_SIZE},
};

use windows_acl::acl::{AceType, ACL};

fn update_perms(dll: &str) {
    let mut siddata = [0u8; SECURITY_MAX_SID_SIZE as usize];
    let psid = PSID(siddata.as_mut_ptr() as *mut c_void);
    let mut sidsize = SECURITY_MAX_SID_SIZE;
    let res = unsafe { CreateWellKnownSid(WinBuiltinAnyPackageSid, None, psid, &mut sidsize) };

    if res.is_err() {
        println!("error getting anypackages sid: {:?}", res);
        return;
    }

    let res = ACL::from_file_path(dll, false);
    let Ok(mut acl) = res else {
        println!("error dll acl: {:?}", res);
        return;
    };

    let res = acl.add_entry(
        psid.0,
        AceType::AccessAllow,
        0,
        GENERIC_READ.0 | GENERIC_EXECUTE.0,
    );
    let Ok(res) = res else {
        println!("error adding acl entry: {:?}", res);
        return;
    };

    println!("res: {:?}", res);
}

fn inject(pid: Pid) {
    let args: Vec<String> = std::env::args().collect();

    update_perms(&args[1]);
    let Ok(proc) = OwnedProcess::from_pid(pid.as_u32()) else {
        println!("error getting calc proc");
        return;
    };
    let syringe = Syringe::for_process(proc);
    let res = syringe.inject(&args[1]);
    if res.is_err() {
        println!("shitfuck: {:?}", res);
    }
}

fn main() {
    let mut sys = System::new_with_specifics(
        RefreshKind::nothing()
            .with_processes(ProcessRefreshKind::nothing().with_exe(UpdateKind::Always)),
    );

    let mut injected = Vec::new();

    loop {
        // wait for calculator app to open
        std::thread::sleep(Duration::from_secs(2));
        let calcs = sys.processes_by_exact_name(OsStr::new("CalculatorApp.exe"));
        for c in calcs {
            let pid = c.pid();
            if !injected.contains(&pid) {
                println!("{}", pid);
                std::thread::sleep(Duration::from_secs(3));
                inject(pid);
                injected.push(pid);
            }
        }
        sys.refresh_specifics(
            RefreshKind::nothing()
                .with_processes(ProcessRefreshKind::nothing().with_exe(UpdateKind::Always)),
        );
    }
}
