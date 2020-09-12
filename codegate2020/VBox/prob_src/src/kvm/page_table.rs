use super::KVM;
use bitflags::bitflags;
use core::ops::{Index, IndexMut};

pub trait Level {}
pub trait HasNextLevel {}
pub trait HasUpperLevel {}

pub enum DescriptorType<T, V> {
    Invalid(V),
    Table(T),
}

impl<T, V> DescriptorType<T, V> {
    #[inline]
    pub fn unwrap_or_else<F: FnOnce(V) -> T>(self, op: F) -> T {
        match self {
            Self::Table(t) => t,
            Self::Invalid(e) => op(e),
        }
    }

    #[inline]
    pub fn get_table(self) -> Result<T, ()> {
        match self {
            Self::Table(t) => Ok(t),
            Self::Invalid(_) => Err(()),
        }
    }
}

#[repr(transparent)]
pub struct Entry<T>(u64, std::marker::PhantomData<T>);

impl<T> Clone for Entry<T> {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}

impl<T> Copy for Entry<T> {}

impl<T> Entry<T> {
    pub const fn new(p: u64) -> Self {
        Self(p, std::marker::PhantomData)
    }

    #[inline]
    pub fn set(&mut self, v: u64) {
        self.0 = v;
    }

    #[inline]
    pub fn get(&self) -> u64 {
        self.0
    }

    #[inline]
    pub fn get_pa(&self) -> u64 {
        self.0 & 0x8ffffffff000
    }
}

pub trait NextLevel<T> {
    fn as_next_table(&mut self, kvm: &KVM) -> Result<DescriptorType<&mut T, &mut Self>, ()>;
}

const L1_SHIFT: u64 = 39;
const L2_SHIFT: u64 = 30;
const L3_SHIFT: u64 = 21;
const L4_SHIFT: u64 = 12;

pub fn get_index(v: u64) -> (u64, u64, u64, u64) {
    (
        (v >> L1_SHIFT) & ((1 << 9) - 1),
        (v >> L2_SHIFT) & ((1 << 9) - 1),
        (v >> L3_SHIFT) & ((1 << 9) - 1),
        (v >> L4_SHIFT) & ((1 << 9) - 1),
    )
}

impl<T> NextLevel<T> for Entry<T>
where
    T: HasUpperLevel + 'static,
{
    fn as_next_table(&mut self, kvm: &KVM) -> Result<DescriptorType<&mut T, &mut Self>, ()> {
        let flags = PageTableFlags::from_bits_truncate(self.0);

        if !flags.contains(PageTableFlags::PRESENT) {
            Ok(DescriptorType::Invalid(self))
        } else if flags.contains(PageTableFlags::HUGE_PAGE) {
            Err(())
        } else {
            Ok(DescriptorType::Table(unsafe {
                (kvm.get_pa(self.0 & !0xfff).ok_or(())?.as_ptr() as usize as *mut T)
                    .as_mut()
                    .expect("Fatal error")
            }))
        }
    }
}

impl NextLevel<()> for Entry<()> {
    fn as_next_table(&mut self, _kvm: &KVM) -> Result<DescriptorType<&mut (), &mut Self>, ()> {
        let flags = PageTableFlags::from_bits_truncate(self.0);

        if !flags.contains(PageTableFlags::PRESENT) {
            Ok(DescriptorType::Invalid(self))
        } else {
            Err(())
        }
    }
}

#[repr(C, align(4096))]
pub struct PageTable<L>
where
    L: Level,
{
    entries: [Entry<L>; 512],
}

impl<L> PageTable<L>
where
    L: Level,
{
    pub fn empty() -> Self {
        Self {
            entries: [Entry::new(0); 512],
        }
    }
}

impl<L> Index<usize> for PageTable<L>
where
    L: Level,
{
    type Output = Entry<L>;

    fn index(&self, index: usize) -> &Entry<L> {
        &self.entries[index]
    }
}

impl<L> IndexMut<usize> for PageTable<L>
where
    L: Level,
{
    fn index_mut(&mut self, index: usize) -> &mut Entry<L> {
        &mut self.entries[index]
    }
}

pub type L1 = PageTable<L2>;
pub type L2 = PageTable<L3>;
pub type L3 = PageTable<L4>;
pub type L4 = PageTable<()>;

impl Level for L1 {}
impl Level for L2 {}
impl Level for L3 {}
impl Level for L4 {}
impl Level for () {}

impl HasNextLevel for L1 {}
impl HasNextLevel for L2 {}
impl HasNextLevel for L3 {}

impl HasUpperLevel for L2 {}
impl HasUpperLevel for L3 {}
impl HasUpperLevel for L4 {}

bitflags! {
    pub struct PageTableFlags: u64 {
        const PRESENT =         1;
        const WRITABLE =        1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const WRITE_THROUGH =   1 << 3;
        const NO_CACHE =        1 << 4;
        const ACCESSED =        1 << 5;
        const DIRTY =           1 << 6;
        const HUGE_PAGE =       1 << 7;
        const GLOBAL =          1 << 8;
        const NO_EXECUTE =      1 << 63;
    }
}

pub struct PageTableWalker<'a> {
    owner: &'a KVM,
    root: u64,
}

impl<'a> PageTableWalker<'a> {
    pub fn new(root: u64, owner: &'a KVM) -> Self {
        Self { root, owner }
    }

    fn alloc_table<L>(&self, entry: &mut Entry<L>, frees: &mut Vec<u64>) -> &mut L {
        let table = frees.pop().expect("oom!");
        entry.set(
            table
                | (PageTableFlags::PRESENT
                    | PageTableFlags::WRITABLE
                    | PageTableFlags::USER_ACCESSIBLE)
                    .bits(),
        );

        let tbl_hs = self.owner.get_pa(table).expect("Invalid PA.");
        unsafe {
            (tbl_hs.as_ptr() as usize as *mut L)
                .as_mut()
                .expect("Fatal Error")
        }
    }

    pub fn insert(
        &self,
        p: u64,
        v: u64,
        perm: PageTableFlags,
        frees: &mut Vec<u64>,
    ) -> Result<(), ()> {
        if p & 0xfff != 0 && v & 0xfff != 0 {
            Err(())
        } else if let Some(tbl) = self.owner.get_pa(self.root) {
            let (l1, l2, l3, l4) = get_index(v);
            let root = unsafe { (tbl.as_ptr() as *mut L1).as_mut().expect("Fatal Error") };
            root[l1 as usize]
                .as_next_table(self.owner)?
                .unwrap_or_else(|d| self.alloc_table(d, frees))[l2 as usize]
                .as_next_table(self.owner)?
                .unwrap_or_else(|d| self.alloc_table(d, frees))[l3 as usize]
                .as_next_table(self.owner)?
                .unwrap_or_else(|d| self.alloc_table(d, frees))[l4 as usize]
                .set(p | (perm | PageTableFlags::PRESENT).bits());
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn walk(&self, v: u64) -> Result<&'static Entry<()>, ()> {
        if v & 0xfff != 0 {
            Err(())
        } else if let Some(tbl) = self.owner.get_pa(self.root) {
            let (l1, l2, l3, l4) = get_index(v);
            let root = unsafe { (tbl.as_ptr() as *mut L1).as_mut().expect("Fatal Error") };
            Ok(
                &root[l1 as usize].as_next_table(self.owner)?.get_table()?[l2 as usize]
                    .as_next_table(self.owner)?
                    .get_table()?[l3 as usize]
                    .as_next_table(self.owner)?
                    .get_table()?[l4 as usize],
            )
        } else {
            Err(())
        }
    }
}
