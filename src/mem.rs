use std::{
    fs::OpenOptions,
    io,
    os::{fd::AsRawFd, unix::fs::OpenOptionsExt},
    ptr,
};

#[derive(Clone, Copy, Debug)]
pub struct MemMap {
    pub bus: *mut u32,
    pub phys: *mut u32,
    pub virt: *mut u32,
}

pub unsafe fn map_phys_to_virt(phys: *const u32, size: usize) -> Result<*mut u32, io::Error> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_SYNC)
        .open("/dev/mem")?;

    let result = libc::mmap(
        ptr::null_mut(),
        size,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_SHARED,
        file.as_raw_fd(),
        phys as libc::off_t,
    );

    drop(file);

    if result == libc::MAP_FAILED {
        return Err(io::Error::last_os_error());
    }

    Ok(result as *mut u32)
}

pub unsafe fn unmap_phys_to_virt(ptr: *mut u32, size: usize) {
    libc::munmap(ptr as *mut libc::c_void, size);
}
