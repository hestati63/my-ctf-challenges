pub mod fmt;

use core::slice;

pub use fmt::*;

#[derive(Debug)]
pub struct ELF {
    hdr: ELFHeader64,
    pub inp: Vec<u8>,
}

#[derive(Debug)]
pub struct PhdrIter<'a> {
    pub phdrs: &'a [ProgHeader64],
    pub cursor: u16,
    pub size: u16,
}

impl<'a> Iterator for PhdrIter<'a> {
    type Item = ProgHeader64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.size > self.cursor {
            let result = self.phdrs[self.cursor as usize];
            self.cursor += 1;
            return Some(result);
        }
        None
    }
}

impl<'a> ELF {
    pub fn new(inp: Vec<u8>) -> Result<Self, ()> {
        unsafe {
            match (inp.as_ptr() as *const ELFHeader64).as_ref() {
                Some(hdr) if hdr.ei_magic == 0x464C457F && hdr.ei_class == 2 => {
                    Ok(ELF { hdr: *hdr, inp })
                }
                _ => Err(()),
            }
        }
    }

    pub const fn entry(&self) -> u64 {
        self.hdr.e_entry
    }

    pub const fn phdr_off(&self) -> u64 {
        self.hdr.e_phoff
    }

    pub fn phdrs(&self) -> Result<PhdrIter<'a>, ()> {
        /* XXX: Leak cand 2. */
        if self.phdr_off() as usize
            + self.hdr.e_phnum as usize * std::mem::size_of::<ProgHeader64>()
            < self.inp.len()
        {
            Ok(PhdrIter {
                size: self.hdr.e_phnum,
                phdrs: unsafe {
                    slice::from_raw_parts(
                        (self.inp.as_ptr() as u64 + self.phdr_off()) as *mut ProgHeader64,
                        self.hdr.e_phnum as usize,
                    )
                },
                cursor: 0,
            })
        } else {
            Err(())
        }
    }
}
