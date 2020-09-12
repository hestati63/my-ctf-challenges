#![feature(new_uninit, global_asm)]

#[allow(dead_code)]
mod elf;
mod kvm;
mod syscall;
mod vbox;

use vbox::VBox;

#[derive(Debug)]
pub enum Error {
    OsError(std::io::Error),
    VersionMismatch,
    MemoryRequestFailed,
    InvalidELF,
    InvalidPageTable,
    UnsupportedExitReason(u32),
    UnsupportedSyscall(u64),
    VmLaunchFailed,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(p) = args.get(1) {
        let binary = std::path::Path::new(p);
        if binary.exists() {
            if let Err(err) = VBox::new().unwrap().serve(std::path::Path::new(p)) {
                eprintln!("{:?}", err);
            }
        } else {
            eprintln!("Error: {}: No such file or directory", args[1]);
        }
    } else {
        eprintln!("Usage: {} <binary>", args[0]);
    }
}
