use std::io;

use super::{
    mailbox::{Mailbox, MEM_FLAG_DIRECT, MEM_FLAG_ZERO},
    mem::{map_phys_to_virt, unmap_phys_to_virt, MemMap},
    platform::PAGE_SIZE,
};

pub struct GpuMem<'a> {
    mailbox: &'a Mailbox,
    handle: u32,
    size: usize,
    memmap: MemMap,
}

impl<'a> GpuMem<'a> {
    pub fn alloc(mailbox: &Mailbox, size: usize) -> Result<GpuMem, io::Error> {
        let size = size.next_multiple_of(PAGE_SIZE);

        let mut data = [0u32; 9];

        data[0] = 36;
        data[1] = 0x00000000;
        data[2] = 0x3000c;
        data[3] = 12;
        data[4] = 12;
        data[5] = size as u32;
        data[6] = PAGE_SIZE as u32;
        data[7] = MEM_FLAG_DIRECT | MEM_FLAG_ZERO;
        data[8] = 0x00000000;

        unsafe { mailbox.send(data.as_ptr() as *const libc::c_void) }?;

        let handle = data[5];

        data[0] = 28;
        data[1] = 0x00000000;
        data[2] = 0x3000d;
        data[3] = 4;
        data[4] = 4;
        data[5] = handle;
        data[6] = 0x00000000;

        unsafe { mailbox.send(data.as_ptr() as *const libc::c_void) }?;

        let bus = data[5] as *mut u32;
        let phys = (data[5] - 0xC0000000) as *mut u32;
        let virt = unsafe { map_phys_to_virt(phys, size) }?;

        Ok(GpuMem {
            mailbox,
            handle,
            size,
            memmap: MemMap { bus, phys, virt },
        })
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn memmap(&self) -> &MemMap {
        &self.memmap
    }
}

impl<'a> Drop for GpuMem<'a> {
    fn drop(&mut self) {
        let mailbox = &mut self.mailbox;

        unsafe { unmap_phys_to_virt(self.memmap.virt, self.size) };

        let mut data = [0u32; 9];

        data[0] = 28;
        data[1] = 0x00000000;
        data[2] = 0x3000e;
        data[3] = 4;
        data[4] = 4;
        data[5] = self.handle;
        data[6] = 0x00000000;

        unsafe { mailbox.send(data.as_ptr() as *const libc::c_void).unwrap() };

        data[0] = 28;
        data[1] = 0x00000000;
        data[2] = 0x3000f;
        data[3] = 4;
        data[4] = 4;
        data[5] = self.handle;
        data[6] = 0x00000000;

        unsafe { mailbox.send(data.as_ptr() as *const libc::c_void).unwrap() };
    }
}
