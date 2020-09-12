pub type __off_t = ::std::os::raw::c_long;
pub type size_t = ::std::os::raw::c_ulong;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_userspace_memory_region {
    pub slot: u32,
    pub flags: u32,
    pub guest_phys_addr: u64,
    pub memory_size: u64,
    pub userspace_addr: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_regs {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rsp: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    pub rflags: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_segment {
    pub base: u64,
    pub limit: u32,
    pub selector: u16,
    pub type_: u8,
    pub present: u8,
    pub dpl: u8,
    pub db: u8,
    pub s: u8,
    pub l: u8,
    pub g: u8,
    pub avl: u8,
    pub unusable: u8,
    pub padding: u8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_dtable {
    pub base: u64,
    pub limit: u16,
    pub padding: [u16; 3usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_sregs {
    pub cs: kvm_segment,
    pub ds: kvm_segment,
    pub es: kvm_segment,
    pub fs: kvm_segment,
    pub gs: kvm_segment,
    pub ss: kvm_segment,
    pub tr: kvm_segment,
    pub ldt: kvm_segment,
    pub gdt: kvm_dtable,
    pub idt: kvm_dtable,
    pub cr0: u64,
    pub cr2: u64,
    pub cr3: u64,
    pub cr4: u64,
    pub cr8: u64,
    pub efer: u64,
    pub apic_base: u64,
    pub interrupt_bitmap: [u64; 4usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_debug_exit_arch {
    pub exception: u32,
    pub pad: u32,
    pub pc: u64,
    pub dr6: u64,
    pub dr7: u64,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_vcpu_events {
    pub exception: kvm_vcpu_events__bindgen_ty_1,
    pub interrupt: kvm_vcpu_events__bindgen_ty_2,
    pub nmi: kvm_vcpu_events__bindgen_ty_3,
    pub sipi_vector: u32,
    pub flags: u32,
    pub smi: kvm_vcpu_events__bindgen_ty_4,
    pub reserved: [u8; 27usize],
    pub exception_has_payload: u8,
    pub exception_payload: u64,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_vcpu_events__bindgen_ty_1 {
    pub injected: u8,
    pub nr: u8,
    pub has_error_code: u8,
    pub pending: u8,
    pub error_code: u32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_vcpu_events__bindgen_ty_2 {
    pub injected: u8,
    pub nr: u8,
    pub soft: u8,
    pub shadow: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_vcpu_events__bindgen_ty_3 {
    pub injected: u8,
    pub pending: u8,
    pub masked: u8,
    pub pad: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_vcpu_events__bindgen_ty_4 {
    pub smm: u8,
    pub pending: u8,
    pub smm_inside_nmi: u8,
    pub latched_init: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_sync_regs {
    pub regs: kvm_regs,
    pub sregs: kvm_sregs,
    pub events: kvm_vcpu_events,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct kvm_hyperv_exit {
    pub type_: u32,
    pub u: kvm_hyperv_exit__bindgen_ty_1,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union kvm_hyperv_exit__bindgen_ty_1 {
    pub synic: kvm_hyperv_exit__bindgen_ty_1__bindgen_ty_1,
    pub hcall: kvm_hyperv_exit__bindgen_ty_1__bindgen_ty_2,
    _bindgen_union_align: [u64; 4usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_hyperv_exit__bindgen_ty_1__bindgen_ty_1 {
    pub msr: u32,
    pub control: u64,
    pub evt_page: u64,
    pub msg_page: u64,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_hyperv_exit__bindgen_ty_1__bindgen_ty_2 {
    pub input: u64,
    pub result: u64,
    pub params: [u64; 2usize],
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct kvm_run {
    pub request_interrupt_window: u8,
    pub immediate_exit: u8,
    pub padding1: [u8; 6usize],
    pub exit_reason: u32,
    pub ready_for_interrupt_injection: u8,
    pub if_flag: u8,
    pub flags: u16,
    pub cr8: u64,
    pub apic_base: u64,
    pub __bindgen_anon_1: kvm_run__bindgen_ty_1,
    pub kvm_valid_regs: u64,
    pub kvm_dirty_regs: u64,
    pub s: kvm_run__bindgen_ty_2,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union kvm_run__bindgen_ty_1 {
    pub hw: kvm_run__bindgen_ty_1__bindgen_ty_1,
    pub fail_entry: kvm_run__bindgen_ty_1__bindgen_ty_2,
    pub ex: kvm_run__bindgen_ty_1__bindgen_ty_3,
    pub io: kvm_run__bindgen_ty_1__bindgen_ty_4,
    pub debug: kvm_run__bindgen_ty_1__bindgen_ty_5,
    pub mmio: kvm_run__bindgen_ty_1__bindgen_ty_6,
    pub hypercall: kvm_run__bindgen_ty_1__bindgen_ty_7,
    pub tpr_access: kvm_run__bindgen_ty_1__bindgen_ty_8,
    pub s390_sieic: kvm_run__bindgen_ty_1__bindgen_ty_9,
    pub s390_reset_flags: u64,
    pub s390_ucontrol: kvm_run__bindgen_ty_1__bindgen_ty_10,
    pub dcr: kvm_run__bindgen_ty_1__bindgen_ty_11,
    pub internal: kvm_run__bindgen_ty_1__bindgen_ty_12,
    pub osi: kvm_run__bindgen_ty_1__bindgen_ty_13,
    pub papr_hcall: kvm_run__bindgen_ty_1__bindgen_ty_14,
    pub s390_tsch: kvm_run__bindgen_ty_1__bindgen_ty_15,
    pub epr: kvm_run__bindgen_ty_1__bindgen_ty_16,
    pub system_event: kvm_run__bindgen_ty_1__bindgen_ty_17,
    pub s390_stsi: kvm_run__bindgen_ty_1__bindgen_ty_18,
    pub eoi: kvm_run__bindgen_ty_1__bindgen_ty_19,
    pub hyperv: kvm_hyperv_exit,
    pub padding: [::std::os::raw::c_char; 256usize],
    _bindgen_union_align: [u64; 32usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_run__bindgen_ty_1__bindgen_ty_1 {
    pub hardware_exit_reason: u64,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_run__bindgen_ty_1__bindgen_ty_2 {
    pub hardware_entry_failure_reason: u64,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_run__bindgen_ty_1__bindgen_ty_3 {
    pub exception: u32,
    pub error_code: u32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_run__bindgen_ty_1__bindgen_ty_4 {
    pub direction: u8,
    pub size: u8,
    pub port: u16,
    pub count: u32,
    pub data_offset: u64,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_run__bindgen_ty_1__bindgen_ty_5 {
    pub arch: kvm_debug_exit_arch,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_run__bindgen_ty_1__bindgen_ty_6 {
    pub phys_addr: u64,
    pub data: [u8; 8usize],
    pub len: u32,
    pub is_write: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_run__bindgen_ty_1__bindgen_ty_7 {
    pub nr: u64,
    pub args: [u64; 6usize],
    pub ret: u64,
    pub longmode: u32,
    pub pad: u32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_run__bindgen_ty_1__bindgen_ty_8 {
    pub rip: u64,
    pub is_write: u32,
    pub pad: u32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_run__bindgen_ty_1__bindgen_ty_9 {
    pub icptcode: u8,
    pub ipa: u16,
    pub ipb: u32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_run__bindgen_ty_1__bindgen_ty_10 {
    pub trans_exc_code: u64,
    pub pgm_code: u32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_run__bindgen_ty_1__bindgen_ty_11 {
    pub dcrn: u32,
    pub data: u32,
    pub is_write: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_run__bindgen_ty_1__bindgen_ty_12 {
    pub suberror: u32,
    pub ndata: u32,
    pub data: [u64; 16usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_run__bindgen_ty_1__bindgen_ty_13 {
    pub gprs: [u64; 32usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_run__bindgen_ty_1__bindgen_ty_14 {
    pub nr: u64,
    pub ret: u64,
    pub args: [u64; 9usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_run__bindgen_ty_1__bindgen_ty_15 {
    pub subchannel_id: u16,
    pub subchannel_nr: u16,
    pub io_int_parm: u32,
    pub io_int_word: u32,
    pub ipb: u32,
    pub dequeued: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_run__bindgen_ty_1__bindgen_ty_16 {
    pub epr: u32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_run__bindgen_ty_1__bindgen_ty_17 {
    pub type_: u32,
    pub flags: u64,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_run__bindgen_ty_1__bindgen_ty_18 {
    pub addr: u64,
    pub ar: u8,
    pub reserved: u8,
    pub fc: u8,
    pub sel1: u8,
    pub sel2: u16,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_run__bindgen_ty_1__bindgen_ty_19 {
    pub vector: u8,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union kvm_run__bindgen_ty_2 {
    pub regs: kvm_sync_regs,
    pub padding: [::std::os::raw::c_char; 2048usize],
    _bindgen_union_align: [u64; 256usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kvm_cpuid_entry2 {
    pub function: u32,
    pub index: u32,
    pub flags: u32,
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
    pub padding: [u32; 3usize],
}

#[repr(C)]
pub struct kvm_cpuid2 {
    pub nent: u32,
    pub padding: u32,
    pub entries: [kvm_cpuid_entry2; 100],
}

extern "C" {
    #[allow(improper_ctypes)]
    pub fn mmap(
        addr: *mut ::std::os::raw::c_void,
        len: size_t,
        prot: ::std::os::raw::c_int,
        flags: ::std::os::raw::c_int,
        fd: ::std::os::raw::c_int,
        offset: __off_t,
    ) -> *mut ();
}
