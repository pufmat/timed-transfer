use std::{
    fs::{File, OpenOptions},
    io,
    os::{fd::AsRawFd, unix::fs::OpenOptionsExt},
};

pub const MEM_FLAG_DISCARDABLE: u32 = 1 << 0;
pub const MEM_FLAG_NORMAL: u32 = 0 << 2;
pub const MEM_FLAG_DIRECT: u32 = 1 << 2;
pub const MEM_FLAG_COHERENT: u32 = 2 << 2;
pub const MEM_FLAG_L1_NONALLOCATING: u32 = MEM_FLAG_DIRECT | MEM_FLAG_COHERENT;
pub const MEM_FLAG_ZERO: u32 = 1 << 4;
pub const MEM_FLAG_NO_INIT: u32 = 1 << 5;
pub const MEM_FLAG_HINT_PERMALOCK: u32 = 1 << 6;

pub struct Mailbox {
    file: File,
}

impl Mailbox {
    pub fn open() -> Result<Mailbox, io::Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_SYNC)
            .open("/dev/vcio")?;

        Ok(Mailbox { file })
    }

    pub unsafe fn send(&self, ptr: *const libc::c_void) -> Result<i32, io::Error> {
        let result = unsafe { libc::ioctl(self.file.as_raw_fd(), 0xC0046400, ptr) };

        if result < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(result)
    }
}
