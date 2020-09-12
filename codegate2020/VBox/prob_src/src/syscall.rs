use crate::kvm::page_table::{Entry, PageTableFlags, PageTableWalker};
use crate::kvm::{raw, KVM};
use crate::vbox::{ExitKind, FileManager};
use crate::Error;
use std::collections::BTreeMap;

pub struct SyscallHandler<'a> {
    kvm: &'a mut KVM,
    files: &'a mut FileManager,
    id: usize,
}

impl<'a> SyscallHandler<'a> {
    pub fn new(kvm: &'a mut KVM, id: usize, files: &'a mut FileManager) -> Self {
        Self { kvm, id, files }
    }

    pub fn gather_buffer_slices(
        kvm: &mut KVM,
        id: usize,
        base: u64,
        size: u64,
        perm: PageTableFlags,
    ) -> Result<BTreeMap<u64, &'static Entry<()>>, ()> {
        let cpu = kvm.get_vcpu_mut(id).expect("Fatal error.");
        let cr3 = cpu.sregs.cr3;
        let walker = PageTableWalker::new(cr3, kvm);

        let st = base & !4095;
        let ed = (base + size + 4095) & !4095;
        let mut entries = BTreeMap::new();
        for va in (st..ed).step_by(0x1000) {
            let entry = walker.walk(va)?;
            if !PageTableFlags::from_bits_truncate(entry.get())
                .contains(PageTableFlags::PRESENT | perm)
                || entry.get_pa() >= kvm.mem.size()
            {
                return Err(());
            }
            entries.insert(va, entry);
        }
        Ok(entries)
    }

    pub fn sys_write(&mut self, fd: u64, buf: u64, size: u64) -> u64 {
        let Self { files, kvm, id } = self;
        if let Some(mut desc) = files.get(fd as i32) {
            if let Ok(buffer_pas) =
                Self::gather_buffer_slices(kvm, *id, buf, size, PageTableFlags::empty())
            {
                let mut rem = size;
                let mut write_bytes = 0;
                while rem > 0 {
                    let ofs = (buf + write_bytes) & 4095;
                    let pa = buffer_pas[&(buf + write_bytes - ofs)].get_pa();
                    unsafe {
                        let slices = kvm.get_pa_unchecked(pa);
                        let this_write = std::cmp::min(4096 - ofs, rem);
                        if let Ok(this_write) =
                            desc.write(&slices[ofs as usize..(ofs + this_write) as usize])
                        {
                            if this_write == 0 { break; }
                            rem -= this_write as u64;
                            write_bytes += this_write as u64;
                        } else {
                            break;
                        }
                    }
                }
                return write_bytes;
            }
        }
        -1_i64 as u64
    }

    pub fn sys_read(&mut self, fd: u64, buf: u64, size: u64) -> u64 {
        let Self { files, kvm, id } = self;
        if let Some(mut desc) = files.get(fd as i32) {
            if let Ok(buffer_pas) =
                Self::gather_buffer_slices(kvm, *id, buf, size, PageTableFlags::WRITABLE)
            {
                let mut rem = size;
                let mut read_bytes = 0;
                while rem > 0 {
                    let ofs = (buf + read_bytes) & 4095;
                    let pa = buffer_pas[&(buf + read_bytes - ofs)].get_pa();
                    unsafe {
                        let slices = kvm.get_pa_mut_unchecked(pa);
                        let this_read = std::cmp::min(4096 - ofs, rem);
                        if let Ok(this_read) =
                            desc.read(&mut slices[ofs as usize..(ofs + this_read) as usize])
                        {
                            if this_read == 0 { break; }
                            rem -= this_read as u64;
                            read_bytes += this_read as u64;
                        } else {
                            break;
                        }
                    }
                }
                return read_bytes;
            }
        }
        -1_i64 as u64
    }

    pub fn read_string(&mut self, buf: u64) -> Result<String, ()> {
        let Self { files, kvm, id } = self;
        let mut pbuf = String::new();
        let mut len = 0;
        loop {
            let ofs = (buf + len) % 0x1000;

            if let Ok(buffer_pas) =
                Self::gather_buffer_slices(kvm, *id, buf, 0x1000 - ofs, PageTableFlags::empty())
            {
                unsafe {
                    let slices =
                        kvm.get_pa_mut_unchecked(buffer_pas.iter().next().unwrap().1.get_pa());
                    for i in ofs..0x1000 {
                        let ch = slices[i as usize % 0x1000];
                        if ch == 0 {
                            return Ok(pbuf);
                        }
                        pbuf.push(ch.into());

                        len += 1;
                        if len > 0x1000 {
                            return Err(());
                        }
                    }
                }
            } else {
                return Err(());
            }
        }
    }

    pub fn sys_open(&mut self, path: u64, flags: u64, mode: u64) -> u64 {
        if let Ok(file) = self.read_string(path) {
            let Self { files, kvm, id } = self;
            files
                .open(file, flags, mode)
                .map(|i| i as u64)
                .unwrap_or(-1_i64 as u64)
        } else {
            -1_i64 as u64
        }
    }

    pub fn serve(mut self) -> Result<(), ExitKind> {
        let cpu = self.kvm.get_vcpu_mut(self.id).expect("Fatal error.");
        cpu.get_regs().map_err(ExitKind::Error)?;
        cpu.get_sregs().map_err(ExitKind::Error)?;

        //println!("{:#?} syscall!", cpu.regs);
        let o = match cpu.regs.rax {
            0 => {
                // sys read
                let raw::kvm_regs {
                    rdi: fd,
                    rsi: buf,
                    rdx: size,
                    ..
                } = cpu.regs;
                Ok(self.sys_read(fd, buf, size))
            }
            1 => {
                // sys write
                let raw::kvm_regs {
                    rdi: fd,
                    rsi: buf,
                    rdx: size,
                    ..
                } = cpu.regs;
                Ok(self.sys_write(fd, buf, size))
            }
            2 => {
                let raw::kvm_regs {
                    rdi: path,
                    rsi: flags,
                    rdx: mode,
                    ..
                } = cpu.regs;
                Ok(self.sys_open(path, flags, mode))
            }
            3 => {
                let raw::kvm_regs { rdi: fd, .. } = cpu.regs;
                Ok(self
                    .files
                    .close(fd as i32)
                    .map(|_| 0)
                    .unwrap_or(-1_i64 as u64))
            }
            60 => Err(ExitKind::SysExit(cpu.regs.rdi as i32)),
            sysnum => Err(ExitKind::Error(Error::UnsupportedSyscall(sysnum))),
        }?;

        let cpu = self.kvm.get_vcpu_mut(self.id).expect("Fatal error.");
        cpu.regs.rax = o;
        cpu.fill_regs().map_err(ExitKind::Error)?;
        Ok(())
    }
}
