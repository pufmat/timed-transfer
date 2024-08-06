pub const RASPBERRY_PI_ZERO_1: Platform = Platform {
    phys: 0x20000000 as *mut u32,
    bus: 0x7E000000 as *mut u32,
};

pub const RASPBERRY_PI_ZERO_2: Platform = Platform {
    phys: 0x3F000000 as *mut u32,
    bus: 0x7E000000 as *mut u32,
};

pub const RASPBERRY_PI_1: Platform = Platform {
    phys: 0x20000000 as *mut u32,
    bus: 0x7E000000 as *mut u32,
};

pub const RASPBERRY_PI_2: Platform = Platform {
    phys: 0x3F000000 as *mut u32,
    bus: 0x7E000000 as *mut u32,
};

pub const RASPBERRY_PI_3: Platform = Platform {
    phys: 0x3F000000 as *mut u32,
    bus: 0x7E000000 as *mut u32,
};

pub const RASPBERRY_PI_4: Platform = Platform {
    phys: 0xFE000000 as *mut u32,
    bus: 0x7E000000 as *mut u32,
};

pub const PAGE_SIZE: usize = 0x1000;

pub struct Platform {
    pub bus: *mut u32,
    pub phys: *mut u32,
}
