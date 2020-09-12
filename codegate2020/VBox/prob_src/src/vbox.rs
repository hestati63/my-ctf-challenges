use super::elf::ELF;
use super::kvm::page_table::{get_index, PageTableFlags};
use super::kvm::{raw, MemoryRegion, KVM};
use super::syscall::SyscallHandler;
use super::Error;

use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{Read, Write};

pub struct FileObject {
    contents: Vec<u8>,
}

impl FileObject {
    pub fn new() -> Self {
        Self { contents: vec![] }
    }
}

pub enum FileKind {
    Regular(FileObject),
    InDev,
    OutDev,
}

#[derive(Clone)]
pub struct DescInner {
    path: String,
    pos: usize,
    mode: u64,
}

pub struct Desc<'a> {
    fd: i32,
    inner: DescInner,
    manager: &'a mut FileManager,
}

impl<'a> Desc<'a> {
    pub fn write(&mut self, slice: &[u8]) -> Result<usize, ()> {
        let file = self.manager.files.get_mut(&self.inner.path).ok_or(())?;
        match file {
            FileKind::OutDev => {
                std::io::stdout().write_all(slice).expect("Fail to write");
                Ok(slice.len())
            }
            FileKind::Regular(obj) if self.inner.mode & 3 != 0 => {
                let file_len = obj.contents.len();
                obj.contents.resize(self.inner.pos, 0);
                obj.contents.extend(slice);
                self.inner.pos += slice.len();
                Ok(slice.len())
            }
            _ => Err(()),
        }
    }

    pub fn read(&mut self, slice: &mut [u8]) -> Result<usize, ()> {
        let file = self.manager.files.get_mut(&self.inner.path).ok_or(())?;
        match file {
            FileKind::InDev => {
                std::io::stdin().read_exact(slice).expect("Fail to read");
                Ok(slice.len())
            }
            FileKind::Regular(obj) if self.inner.mode & 3 != 1 => {
                let file_len = obj.contents.len();
                let last = core::cmp::min(slice.len() + self.inner.pos, file_len);
                let change = last - self.inner.pos;
                slice[..change].clone_from_slice(&obj.contents[self.inner.pos..last]);
                self.inner.pos += change;
                Ok(change)
            }
            _ => Err(()),
        }
    }
}

impl<'a> Drop for Desc<'a> {
    fn drop(&mut self) {
        self.manager.desc.insert(self.fd, self.inner.clone());
    }
}

pub struct FileManager {
    files: BTreeMap<String, FileKind>,
    desc: BTreeMap<i32, DescInner>,
    fds: Vec<i32>,
}

impl FileManager {
    pub fn new() -> Self {
        Self {
            files: vec![
                ("/dev/stdin".to_string(), FileKind::InDev),
                ("/dev/stdout".to_string(), FileKind::OutDev),
                ("/dev/stderr".to_string(), FileKind::OutDev),
            ]
            .into_iter()
            .collect(),
            desc: vec![
                (
                    0,
                    DescInner {
                        path: "/dev/stdin".to_string(),
                        pos: 0,
                        mode: 0,
                    },
                ),
                (
                    1,
                    DescInner {
                        path: "/dev/stdout".to_string(),
                        pos: 0,
                        mode: 0,
                    },
                ),
                (
                    2,
                    DescInner {
                        path: "/dev/stderr".to_string(),
                        pos: 0,
                        mode: 0,
                    },
                ),
            ]
            .into_iter()
            .collect(),
            fds: (3..0x1000).rev().collect(),
        }
    }

    pub fn open_file(&mut self, fd: i32, path: String, mode: u64) -> i32 {
        self.desc.insert(fd, DescInner { path, pos: 0, mode });
        fd
    }

    pub fn open(&mut self, path: String, flags: u64, mode: u64) -> Result<i32, ()> {
        if mode & 3 == 0o1 {
            self.files.remove(&path);
            self.open(path, flags, mode)
        } else if let Some(_) = self.files.get(&path) {
            if let Some(fd) = self.fds.pop() {
                Ok(self.open_file(fd, path, mode))
            } else {
                Err(())
            }
        } else if mode & 0o100 == 0o100 {
            self.files
                .insert(path.clone(), FileKind::Regular(FileObject::new()));
            self.open(path, flags, mode - 0o100)
        } else {
            Err(())
        }
    }

    pub fn get(&mut self, fd: i32) -> Option<Desc<'_>> {
        if let Some(inner) = self.desc.get(&fd) {
            Some(Desc {
                fd,
                inner: inner.clone(),
                manager: self,
            })
        } else {
            None
        }
    }

    pub fn close(&mut self, fd: i32) -> Result<(), ()> {
        if let Some(_) = self.desc.remove(&fd) {
            self.fds.push(fd);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn create(&mut self, name: &str) -> Result<(), ()> {
        if self.files.get(name).is_none() {
            self.files
                .insert(name.to_string(), FileKind::Regular(FileObject::new()));
            Ok(())
        } else {
            Err(())
        }
    }
}

pub struct VBox {
    kvm: KVM,
    files: FileManager,
}

const CS_SEGMENT: raw::kvm_segment = raw::kvm_segment {
    selector: 8,
    base: 0,
    limit: 0xffffffff,
    type_: 0xa,
    s: 1,
    dpl: 0,
    present: 1,
    avl: 0,
    l: 1,
    db: 0,
    g: 1,
    unusable: 0,
    padding: 0,
};

const DS_SEGMENT: raw::kvm_segment = raw::kvm_segment {
    selector: 0x10,
    base: 0,
    limit: 0xffffffff,
    type_: 0x3,
    s: 1,
    dpl: 0,
    present: 1,
    avl: 0,
    l: 1,
    db: 0,
    g: 1,
    unusable: 0,
    padding: 0,
};

const STACK_BOT: u64 = 254 << 39;
const STACK_SIZE: u64 = 0x100000;
const SERVER_LOC: u64 = STACK_BOT + 0x20000000;

type MappingRequests = BTreeMap<u64, (PageTableFlags, Option<(u64, u64, u64)>)>;
pub struct AllocationTracker {
    mapping_requests: MappingRequests,
    p1_set: BTreeSet<u64>,
    p2_set: BTreeSet<u64>,
    p3_set: BTreeSet<u64>,
    p4_set: BTreeSet<u64>,
}

impl AllocationTracker {
    pub fn new() -> Self {
        Self {
            mapping_requests: BTreeMap::new(),
            p1_set: BTreeSet::new(),
            p2_set: BTreeSet::new(),
            p3_set: BTreeSet::new(),
            p4_set: BTreeSet::new(),
        }
    }

    pub fn push_allocation(
        &mut self,
        va: u64,
        load_info: Option<(u64, u64, u64)>,
        perm: PageTableFlags,
    ) -> bool {
        //println!("This: {:x} {:?} {:?}", va, load_info, perm);
        // Track pt allcations.
        let (p1, p2, p3, p4) = get_index(va);
        self.p1_set.insert(p1);
        self.p2_set.insert(p2);
        self.p3_set.insert(p3);
        self.p4_set.insert(p4);

        self.mapping_requests
            .insert(va, (perm, load_info))
            .is_none()
    }

    pub fn allocations(self) -> MappingRequests {
        self.mapping_requests
    }

    pub fn get_mapping_count(&self) -> u64 {
        self.mapping_requests.len() as u64
    }

    pub fn get_pt_count(&self) -> u64 {
        (self.p1_set.len() + self.p2_set.len() + self.p3_set.len() + self.p4_set.len() + 1) as u64
    }
}

impl VBox {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            kvm: KVM::new()?,
            files: FileManager::new(),
        })
    }

    pub fn read_binary(bin: &std::path::Path) -> std::io::Result<Vec<u8>> {
        let mut file = File::open(bin)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;
        Ok(contents)
    }

    pub fn setup_cpu_state(
        &mut self,
        id: usize,
        cr3: u64,
        gdt_base: u64,
        syscall: u64,
        pc: u64,
    ) -> Result<(), Error> {
        let cpu = self.kvm.get_vcpu_mut(id).expect("Fatal error");
        cpu.get_sregs()?;

        let sregs = cpu.sregs;
        cpu.sregs = raw::kvm_sregs {
            cr0: 0x80000013,
            cr2: 0,
            cr3,
            cr4: 0x620,
            // Long mode + syscall.
            efer: 0xd01,
            gdt: raw::kvm_dtable {
                base: gdt_base,
                limit: 0x17,
                padding: [0; 3],
            },
            cs: CS_SEGMENT,
            ds: DS_SEGMENT,
            es: DS_SEGMENT,
            fs: DS_SEGMENT,
            gs: DS_SEGMENT,
            ss: DS_SEGMENT,
            cr8: sregs.cr8,
            apic_base: sregs.apic_base,
            idt: sregs.idt,
            ldt: sregs.ldt,
            tr: sregs.tr,
            interrupt_bitmap: sregs.interrupt_bitmap,
        };
        cpu.fill_sregs()?;

        // Fill regs
        cpu.get_regs()?;
        cpu.regs.rip = pc;
        cpu.regs.rsp = STACK_BOT + STACK_SIZE - 8;
        cpu.regs.rflags = 2;
        cpu.fill_regs()?;

        // syscall
        cpu.write_msr(0xc0000081, (8 << 48) | (8 << 32))?;
        cpu.write_msr(0xc0000082, syscall)?;
        cpu.write_msr(0xc0000084, 0xffffffffffffffff).map(|_| ())
    }

    pub fn fill_allocation_tracker(&mut self, elf: &ELF) -> Result<AllocationTracker, Error> {
        let mut builder = AllocationTracker::new();

        for phdr in elf
            .phdrs()
            .map_err(|_| Error::InvalidELF)?
            .filter(|t| t.p_type == crate::elf::PType::LOAD as u32)
        {
            if phdr.p_vaddr >= STACK_BOT {
                return Err(Error::InvalidELF);
            }
            let mut flags = PageTableFlags::empty();
            if phdr.p_flags & 2 == 2 {
                flags |= PageTableFlags::WRITABLE;
            }
            if phdr.p_flags & 1 != 1 {
                flags |= PageTableFlags::NO_EXECUTE;
            }

            let st = phdr.p_vaddr & !4095;
            let ed = (phdr.p_vaddr + phdr.p_memsz + 4095) & !4095;
            let mut rem = phdr.p_filesz;
            let mut ofs = phdr.p_offset;

            for vaddr in (st..ed).step_by(4096) {
                let mem_ofs = if vaddr < st { st - vaddr } else { 0 };
                let this_write = if rem > 4096 { 4096 } else { rem } - mem_ofs;

                if !builder.push_allocation(vaddr, Some((mem_ofs, this_write, ofs)), flags) {
                    return Err(Error::InvalidELF);
                }

                rem -= this_write;
                ofs += this_write;
            }
        }
        Ok(builder)
    }

    pub fn load_binary(&mut self, bin: &std::path::Path, id: usize) -> Result<(), Error> {
        let elf = ELF::new(Self::read_binary(bin).map_err(Error::OsError)?)
            .map_err(|_| Error::InvalidELF)?;
        let mut builder = self.fill_allocation_tracker(&elf)?;
        let server_base = unsafe { &service_routine as *const _ as u64 };

        assert!(builder.push_allocation(SERVER_LOC, None, PageTableFlags::empty()));

        (STACK_BOT..STACK_BOT + STACK_SIZE)
            .step_by(0x1000)
            .for_each(|i| {
                if !builder.push_allocation(i, None, PageTableFlags::WRITABLE) {
                    unreachable!();
                }
            });

        // binary + stack.
        let size = builder.get_mapping_count() as u64 * 0x1000;
        let pg_tbl_pbase = size + 0x1000 + STACK_SIZE;
        let tsize = pg_tbl_pbase + builder.get_pt_count() * 0x1000;

        // Binary memory.
        self.kvm.register_region(MemoryRegion::new(tsize)?)?;

        // Set cr3 first.
        self.setup_cpu_state(
            id,
            pg_tbl_pbase,
            // offset of gdt table
            unsafe { &gdt64 as *const _ as usize as u64 } - server_base + size - 0x1000,
            unsafe { &do_syscall as *const _ as usize as u64 } - server_base + SERVER_LOC,
            elf.entry(),
        )?;

        let mut pa = 0;
        let mut frees = (pg_tbl_pbase + 0x1000..tsize)
            .step_by(0x1000)
            .collect::<Vec<_>>();
        for (va, (flags, loading)) in builder.allocations().into_iter() {
            self.kvm.map_pv(id, pa, va, flags, &mut frees)?;
            if let Some((mem_ofs, this_write, ofs)) = loading {
                unsafe {
                    let ptr = self.kvm.get_pa_mut_unchecked(pa);
                    core::ptr::copy_nonoverlapping(
                        elf.inp[ofs as usize..].as_ptr(),
                        (ptr.as_mut_ptr() as usize + mem_ofs as usize) as *mut u8,
                        this_write as usize,
                    )
                }
            }
            pa += 0x1000;
        }

        // Copy service_routine
        unsafe {
            let ptr = self.kvm.get_pa_mut_unchecked(size - 0x1000);
            core::ptr::copy_nonoverlapping(
                &service_routine as *const _ as *const u8,
                (ptr.as_mut_ptr() as usize) as *mut u8,
                &service_routine_end as *const _ as usize - &service_routine as *const _ as usize,
            );
        }

        Ok(())
    }

    fn handle_io(kvm: &mut KVM, id: usize, files: &mut FileManager) -> Result<(), ExitKind> {
        let cpu = kvm.get_vcpu_mut(id).expect("Fatal error.");

        if unsafe { cpu.run.__bindgen_anon_1.io.port } == 0 {
            SyscallHandler::new(kvm, id, files).serve()
        } else {
            Err(ExitKind::Error(Error::UnsupportedExitReason(2)))
        }
    }

    fn handle_vm_exit(kvm: &mut KVM, id: usize, files: &mut FileManager) -> Result<(), ExitKind> {
        let cpu = kvm.get_vcpu_mut(id).expect("Fatal error.");
        match cpu.run.exit_reason {
            2 => Self::handle_io(kvm, id, files),
            r => {
                // cpu.get_regs();
                // cpu.get_sregs();
                // println!("{:#?}", cpu.regs);
                // println!("{:#?}", cpu.sregs);
                Err(ExitKind::Error(Error::UnsupportedExitReason(r)))
            }
        }
    }

    pub fn serve(&mut self, bin: &std::path::Path) -> Result<(), Error> {
        let vcpu_idx = self.kvm.new_vcpu()?;
        self.load_binary(bin, vcpu_idx)?;
        match self
            .kvm
            .run(vcpu_idx, Self::handle_vm_exit, &mut self.files)
            .map_err(|_| Error::VmLaunchFailed)?
        {
            ExitKind::Error(e) => Err(e),
            ExitKind::SysExit(exit_code) => {
                println!("VBox exited: {}", exit_code);
                std::process::exit(exit_code)
            }
        }
    }
}

pub enum ExitKind {
    Error(Error),
    SysExit(i32),
}

extern "C" {
    static service_routine: u64;
    static gdt64: u64;
    static do_syscall: u64;
    static service_routine_end: u64;
}

global_asm!(
    "
    service_routine:
    do_syscall:
        out %al, $0
        push %r11
        popf
        jmpq *%rcx
    .p2align 2
    gdt64:
        .quad 0                   # NULL SEGMENT
        .quad 0x00af9a000000ffff  # CODE SEGMENT64
        .quad 0x00cf92000000ffff  # DATA SEGMENT64
    service_routine_end:
"
);
