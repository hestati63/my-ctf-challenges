mod ioctl;
pub mod page_table;
#[allow(dead_code, non_camel_case_types)]
pub mod raw;

use super::Error;
use ioctl::{Into, IoctlFd};
use page_table::PageTableFlags;

#[repr(u64)]
enum KvmOps {
    GetApiVersion = 44544,
    CreateVm = 44545,
    GetVCpuMmapSize = 44548,
    GetSupportedCpuId = 3221794309,
}

impl Into<u64> for KvmOps {
    fn into(self) -> u64 {
        self as u64
    }
}

#[repr(u64)]
enum KvmVmOps {
    SetUserMemoryRegion = 1075883590,
    CreateVCpu = 44609,
}

impl Into<u64> for KvmVmOps {
    fn into(self) -> u64 {
        self as u64
    }
}

#[repr(u64)]
pub enum KvmVCpuOps {
    GetSRegs = 2167975555,
    SetSRegs = 1094233732,
    GetRegs = 2156965505,
    SetRegs = 1083223682,
    Run = 44672,
    SetMsr = 1074310793,
    SetCpuId = 1074310800,
}

impl Into<u64> for KvmVCpuOps {
    fn into(self) -> u64 {
        self as u64
    }
}

pub struct KVMVCpu {
    fd: IoctlFd<KvmVCpuOps>,
    pub run: &'static mut raw::kvm_run,
    pub sregs: raw::kvm_sregs,
    pub regs: raw::kvm_regs,
}

impl KVMVCpu {
    fn new(kvm_fd: &IoctlFd<KvmOps>, vm_fd: &IoctlFd<KvmVmOps>) -> Result<Self, Error> {
        let fd = vm_fd.request(KvmVmOps::CreateVCpu, 0)?;
        let mmap_size = kvm_fd.request(KvmOps::GetVCpuMmapSize, 0)? as u64;
        unsafe {
            // MAP_SHARED
            let run = (raw::mmap(std::ptr::null_mut(), mmap_size, 3, 1, fd, 0)
                as *mut raw::kvm_run)
                .as_mut()
                .ok_or(Error::MemoryRequestFailed)?;

            let fd = IoctlFd::from_raw(fd);
            let mut cpu_vec: raw::kvm_cpuid2 = std::mem::MaybeUninit::zeroed().assume_init();
            cpu_vec.nent = 100;
            kvm_fd.request(KvmOps::GetSupportedCpuId, &mut cpu_vec)?;
            fd.request(KvmVCpuOps::SetCpuId, &cpu_vec)?;

            Ok(Self {
                run,
                fd,
                sregs: std::mem::MaybeUninit::zeroed().assume_init(),
                regs: std::mem::MaybeUninit::zeroed().assume_init(),
            })
        }
    }

    pub fn get_sregs(&mut self) -> Result<(), Error> {
        self.fd.request(KvmVCpuOps::GetSRegs, &mut self.sregs)?;
        Ok(())
    }

    pub fn fill_sregs(&self) -> Result<(), Error> {
        self.fd.request(KvmVCpuOps::SetSRegs, &self.sregs)?;
        Ok(())
    }

    pub fn get_regs(&mut self) -> Result<(), Error> {
        self.fd.request(KvmVCpuOps::GetRegs, &mut self.regs)?;
        Ok(())
    }

    pub fn fill_regs(&self) -> Result<(), Error> {
        self.fd.request(KvmVCpuOps::SetRegs, &self.regs)?;
        Ok(())
    }

    pub fn write_msr(&mut self, index: u32, data: u64) -> Result<i32, Error> {
        let writer = MsrWriter {
            nmsrs: 1,
            _pad: 0,
            entry: [MsrEntry {
                index,
                __reserved: 0,
                data,
            }],
        };
        self.fd.request(KvmVCpuOps::SetMsr, &writer)
    }
}

pub struct KVM {
    fd: IoctlFd<KvmOps>,
    vm_fd: IoctlFd<KvmVmOps>,
    pub mem: MemoryRegion,
    cpus: Vec<KVMVCpu>,
}

#[derive(Debug)]
pub struct MemoryRegion {
    inner: *mut u8,
    size: u64,
}

impl MemoryRegion {
    pub fn new(size: u64) -> Result<Self, Error> {
        unsafe {
            let layout = std::alloc::Layout::from_size_align(size as usize, 4096)
                .map_err(|_| Error::MemoryRequestFailed)?;
            let alloc = std::alloc::alloc_zeroed(layout);
            if alloc.is_null() {
                return Err(Error::MemoryRequestFailed);
            }

            Ok(MemoryRegion { inner: alloc, size })
        }
    }

    pub fn size(&self) -> u64 {
        self.size
    }
}

#[repr(C)]
struct MsrEntry {
    index: u32,
    __reserved: u32,
    data: u64,
}

#[repr(C)]
struct MsrWriter {
    nmsrs: u32,
    _pad: u32,
    entry: [MsrEntry; 1],
}

impl KVM {
    const KVM_API_VERSION: i32 = 12;

    pub fn new() -> Result<Self, Error> {
        let fd = IoctlFd::from_path("/dev/kvm")?;

        let version = fd.request(KvmOps::GetApiVersion, 0)?;
        if version != KVM::KVM_API_VERSION {
            return Err(Error::VersionMismatch);
        }

        let vm_fd = IoctlFd::from_raw(fd.request(KvmOps::CreateVm, 0)?);
        Ok(KVM {
            fd,
            vm_fd,
            mem: MemoryRegion {
                inner: std::ptr::null_mut(),
                size: 0,
            },
            cpus: vec![],
        })
    }

    pub fn new_vcpu(&mut self) -> Result<usize, Error> {
        let cpu = KVMVCpu::new(&self.fd, &self.vm_fd)?;
        let index = self.cpus.len();
        self.cpus.push(cpu);
        Ok(index)
    }

    pub fn get_vcpu_mut(&mut self, idx: usize) -> Option<&mut KVMVCpu> {
        self.cpus.get_mut(idx)
    }

    pub fn register_region(&mut self, mem: MemoryRegion) -> Result<i32, Error> {
        let mut region = raw::kvm_userspace_memory_region {
            slot: 0,
            flags: 0,
            guest_phys_addr: 0,
            memory_size: mem.size,
            userspace_addr: mem.inner as u64,
        };
        let r = self
            .vm_fd
            .request(KvmVmOps::SetUserMemoryRegion, &mut region);
        if r.is_ok() {
            self.mem = mem;
        }
        r
    }

    pub unsafe fn get_pa_mut_unchecked(&mut self, pa: u64) -> &mut [u8] {
        let pa = pa & !4095;
        std::slice::from_raw_parts_mut((self.mem.inner as u64 + pa) as *mut u8, 0x1000)
    }

    pub fn get_pa_mut(&mut self, pa: u64) -> Option<&mut [u8]> {
        if pa < self.mem.size {
            Some(unsafe { self.get_pa_mut_unchecked(pa) })
        } else {
            None
        }
    }

    pub unsafe fn get_pa_unchecked(&self, pa: u64) -> &[u8] {
        let pa = pa & !4095;
        std::slice::from_raw_parts_mut((self.mem.inner as u64 + pa) as *mut u8, 0x1000)
    }

    pub fn get_pa(&self, pa: u64) -> Option<&[u8]> {
        if pa < self.mem.size {
            Some(unsafe { self.get_pa_unchecked(pa) })
        } else {
            None
        }
    }

    pub fn run<F, T, E>(&mut self, id: usize, mut handler: F, aux: &mut T) -> Result<E, ()>
    where
        F: FnMut(&mut Self, usize, &mut T) -> Result<(), E>,
    {
        loop {
            if let Some(cpu) = self.cpus.get(id) {
                cpu.fd.request(KvmVCpuOps::Run, 0).unwrap();
            } else {
                break Err(());
            }
            if let Err(e) = handler(self, id, aux) {
                break Ok(e);
            }
        }
    }

    pub fn map_pv(
        &mut self,
        id: usize,
        pa: u64,
        va: u64,
        perm: PageTableFlags,
        frees: &mut Vec<u64>,
    ) -> Result<(), Error> {
        let cpu = &mut self.cpus[id];
        cpu.get_sregs()?;
        let cr3 = cpu.sregs.cr3;
        page_table::PageTableWalker::new(cr3, self)
            .insert(pa, va, perm, frees)
            .map_err(|_| Error::InvalidPageTable)
    }
}
