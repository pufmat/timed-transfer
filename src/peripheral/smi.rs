use std::io;

use crate::{
    field::{bit, bits, read_bit_field, write_bit_field, Field},
    mem::{map_phys_to_virt, unmap_phys_to_virt, MemMap},
    platform::{Platform, PAGE_SIZE},
};

pub const SMI_OFFSET: usize = 0x00600000;

pub const SMI_CS: usize = 0x00;
pub const SMI_L: usize = 0x04;
pub const SMI_A: usize = 0x08;
pub const SMI_D: usize = 0x0C;

pub const SMI_DSR0: usize = 0x10;
pub const SMI_DSW0: usize = 0x14;
pub const SMI_DSR1: usize = 0x18;
pub const SMI_DSW1: usize = 0x1C;
pub const SMI_DSR2: usize = 0x20;
pub const SMI_DSW2: usize = 0x24;
pub const SMI_DSR3: usize = 0x28;
pub const SMI_DSW3: usize = 0x2C;

pub const SMI_DC: usize = 0x30;

pub const SMI_DCS: usize = 0x34;
pub const SMI_DA: usize = 0x38;
pub const SMI_DD: usize = 0x3C;

pub const SMI_FD: usize = 0x40;

pub const SMI_CS_RXF: Field<u32> = bit(31);
pub const SMI_CS_TXE: Field<u32> = bit(30);
pub const SMI_CS_RXD: Field<u32> = bit(29);
pub const SMI_CS_TXD: Field<u32> = bit(28);
pub const SMI_CS_RXR: Field<u32> = bit(27);
pub const SMI_CS_TXW: Field<u32> = bit(26);
pub const SMI_CS_AFERR: Field<u32> = bit(25);
pub const SMI_CS_PRDY: Field<u32> = bit(24);
pub const SMI_CS_EDREQ: Field<u32> = bit(15);
pub const SMI_CS_PXLDAT: Field<u32> = bit(14);
pub const SMI_CS_SETERR: Field<u32> = bit(13);
pub const SMI_CS_PVMODE: Field<u32> = bit(12);
pub const SMI_CS_INTR: Field<u32> = bit(11);
pub const SMI_CS_INTT: Field<u32> = bit(10);
pub const SMI_CS_INTD: Field<u32> = bit(9);
pub const SMI_CS_TEEN: Field<u32> = bit(8);
pub const SMI_CS_PAD: Field<u32> = bits(7, 6);
pub const SMI_CS_WRITE: Field<u32> = bit(5);
pub const SMI_CS_CLEAR: Field<u32> = bit(4);
pub const SMI_CS_START: Field<u32> = bit(3);
pub const SMI_CS_ACTIVE: Field<u32> = bit(2);
pub const SMI_CS_DONE: Field<u32> = bit(1);
pub const SMI_CS_ENABLE: Field<u32> = bit(0);

pub const SMI_DSR_RWIDTH: Field<u32> = bits(31, 30);
pub const SMI_DSR_RSETUP: Field<u32> = bits(29, 24);
pub const SMI_DSR_MODE68: Field<u32> = bit(23);
pub const SMI_DSR_FSETUP: Field<u32> = bit(22);
pub const SMI_DSR_RHOLD: Field<u32> = bits(21, 16);
pub const SMI_DSR_RPACEALL: Field<u32> = bit(15);
pub const SMI_DSR_RPACE: Field<u32> = bits(14, 8);
pub const SMI_DSR_RDREQ: Field<u32> = bit(7);
pub const SMI_DSR_RSTROBE: Field<u32> = bits(6, 0);

pub const SMI_DSW_WWIDTH: Field<u32> = bits(31, 30);
pub const SMI_DSW_WSETUP: Field<u32> = bits(29, 24);
pub const SMI_DSW_WFORMAT: Field<u32> = bit(23);
pub const SMI_DSW_WSWAP: Field<u32> = bit(22);
pub const SMI_DSW_WHOLD: Field<u32> = bits(21, 16);
pub const SMI_DSW_WPACEALL: Field<u32> = bit(15);
pub const SMI_DSW_WPACE: Field<u32> = bits(14, 8);
pub const SMI_DSW_WDREQ: Field<u32> = bit(7);
pub const SMI_DSW_WSTROBE: Field<u32> = bits(6, 0);

pub const SMI_A_DEVICE: Field<u32> = bits(9, 8);
pub const SMI_A_ADDR: Field<u32> = bits(5, 0);

pub const SMI_DC_DMAEN: Field<u32> = bit(28);
pub const SMI_DC_DMAP: Field<u32> = bit(24);
pub const SMI_DC_PANICR: Field<u32> = bits(23, 18);
pub const SMI_DC_PANICW: Field<u32> = bits(17, 12);
pub const SMI_DC_REQR: Field<u32> = bits(11, 6);
pub const SMI_DC_REQW: Field<u32> = bits(5, 0);

pub const SMI_CLOCK_OFFSET: usize = 0x00101000;

pub const SMI_CLOCK_PASSWD: u32 = 0x5a;

pub const SMI_CLOCK_CTL: usize = 0xb0;
pub const SMI_CLOCK_DIV: usize = 0xb4;

pub const SMI_CLOCK_CTL_PASSWD: Field<u32> = bits(31, 24);
pub const SMI_CLOCK_CTL_MASH: Field<u32> = bits(10, 9);
pub const SMI_CLOCK_CTL_FLIP: Field<u32> = bit(8);
pub const SMI_CLOCK_CTL_BUSY: Field<u32> = bit(7);
pub const SMI_CLOCK_CTL_KILL: Field<u32> = bit(5);
pub const SMI_CLOCK_CTL_ENAB: Field<u32> = bit(4);
pub const SMI_CLOCK_CTL_SRC: Field<u32> = bits(3, 0);

pub const SMI_CLOCK_DIV_PASSWD: Field<u32> = bits(31, 24);
pub const SMI_CLOCK_DIV_DIVI: Field<u32> = bits(23, 12);
pub const SMI_CLOCK_DIV_DIVF: Field<u32> = bits(11, 0);

pub struct Peripheral {
    regs: MemMap,
    pub controller: Controller,
    pub devices: Devices,
}

impl Peripheral {
    pub fn open(base: &Platform) -> Result<Peripheral, io::Error> {
        let bus = base.bus.wrapping_byte_add(SMI_OFFSET);
        let phys = base.phys.wrapping_byte_add(SMI_OFFSET);
        let virt = unsafe { map_phys_to_virt(phys, PAGE_SIZE) }?;
        let regs = MemMap { bus, phys, virt };

        let clock_bus = base.bus.wrapping_byte_add(SMI_CLOCK_OFFSET);
        let clock_phys = base.phys.wrapping_byte_add(SMI_CLOCK_OFFSET);
        let clock_virt = unsafe { map_phys_to_virt(clock_phys, PAGE_SIZE) }?;
        let clock_regs = MemMap {
            bus: clock_bus,
            phys: clock_phys,
            virt: clock_virt,
        };

        Ok(Peripheral {
            regs,
            controller: Controller { regs, clock_regs },
            devices: Devices {
                device0: Device0 {
                    dsr_virt: regs.virt.wrapping_byte_add(SMI_DSR0),
                    dsw_virt: regs.virt.wrapping_byte_add(SMI_DSW0),
                },
                device1: Device1 {
                    dsr_virt: regs.virt.wrapping_byte_add(SMI_DSR1),
                    dsw_virt: regs.virt.wrapping_byte_add(SMI_DSW1),
                },
                device2: Device2 {
                    dsr_virt: regs.virt.wrapping_byte_add(SMI_DSR2),
                    dsw_virt: regs.virt.wrapping_byte_add(SMI_DSW2),
                },
                device3: Device3 {
                    dsr_virt: regs.virt.wrapping_byte_add(SMI_DSR3),
                    dsw_virt: regs.virt.wrapping_byte_add(SMI_DSW3),
                },
            },
        })
    }
}

impl Drop for Peripheral {
    fn drop(&mut self) {
        unsafe { unmap_phys_to_virt(self.regs.virt, PAGE_SIZE) };
    }
}

pub struct Controller {
    pub(crate) regs: MemMap,
    clock_regs: MemMap,
}

impl Controller {
    pub fn select<D: Device>(&mut self, _device: &D) {
        let mut a = unsafe { self.regs.virt.byte_add(SMI_A).read_volatile() };
        write_bit_field(&mut a, SMI_A_DEVICE, D::INDEX);
        unsafe { self.regs.virt.byte_add(SMI_A).write_volatile(a) };
    }

    pub fn set_clock_divisor(&mut self, divisor: u16) {
        let mut ctl = 0;
        write_bit_field(&mut ctl, SMI_CLOCK_CTL_PASSWD, SMI_CLOCK_PASSWD);
        unsafe {
            self.clock_regs
                .virt
                .byte_add(SMI_CLOCK_CTL)
                .write_volatile(ctl)
        };

        let mut ctl = 0;
        write_bit_field(&mut ctl, SMI_CLOCK_CTL_PASSWD, SMI_CLOCK_PASSWD);
        write_bit_field(&mut ctl, SMI_CLOCK_CTL_KILL, true);
        unsafe {
            self.clock_regs
                .virt
                .byte_add(SMI_CLOCK_CTL)
                .write_volatile(ctl)
        };

        loop {
            let ctl = unsafe { self.clock_regs.virt.byte_add(SMI_CLOCK_CTL).read_volatile() };
            if read_bit_field(ctl, SMI_CLOCK_CTL_BUSY) == 0 {
                break;
            }
        }

        let mut div = 0;
        write_bit_field(&mut div, SMI_CLOCK_DIV_PASSWD, SMI_CLOCK_PASSWD);
        write_bit_field(&mut div, SMI_CLOCK_DIV_DIVI, divisor);
        unsafe {
            self.clock_regs
                .virt
                .byte_add(SMI_CLOCK_DIV)
                .write_volatile(div)
        };

        let mut ctl = 0;
        write_bit_field(&mut ctl, SMI_CLOCK_CTL_PASSWD, SMI_CLOCK_PASSWD);
        write_bit_field(&mut ctl, SMI_CLOCK_CTL_SRC, 6u32);
        write_bit_field(&mut ctl, SMI_CLOCK_CTL_ENAB, true);
        unsafe {
            self.clock_regs
                .virt
                .byte_add(SMI_CLOCK_CTL)
                .write_volatile(ctl)
        };

        loop {
            let ctl = unsafe { self.clock_regs.virt.byte_add(SMI_CLOCK_CTL).read_volatile() };
            if read_bit_field(ctl, SMI_CLOCK_CTL_BUSY) == 1 {
                break;
            }
        }
    }

    pub fn set_dir(&mut self, dir: TransferDir) {
        let mut cs = unsafe { self.regs.virt.byte_add(SMI_CS).read_volatile() };
        write_bit_field(
            &mut cs,
            SMI_CS_WRITE,
            match dir {
                TransferDir::Write => true,
                TransferDir::Read => false,
            },
        );
        unsafe { self.regs.virt.byte_add(SMI_CS).write_volatile(cs) };
    }

    pub fn enable(&mut self) {
        let mut cs = unsafe { self.regs.virt.byte_add(SMI_CS).read_volatile() };
        write_bit_field(&mut cs, SMI_CS_ENABLE, true);
        unsafe { self.regs.virt.byte_add(SMI_CS).write_volatile(cs) };
    }

    pub fn disable(&mut self) {
        let mut cs = unsafe { self.regs.virt.byte_add(SMI_CS).read_volatile() };
        write_bit_field(&mut cs, SMI_CS_ENABLE, false);
        unsafe { self.regs.virt.byte_add(SMI_CS).write_volatile(cs) };
    }

    pub fn active(&self) -> bool {
        let cs = unsafe { self.regs.virt.byte_add(SMI_CS).read_volatile() };
        read_bit_field(cs, SMI_CS_ACTIVE) == 1
    }

    pub fn start(&mut self) {
        let mut cs = unsafe { self.regs.virt.byte_add(SMI_CS).read_volatile() };
        write_bit_field(&mut cs, SMI_CS_START, true);
        unsafe { self.regs.virt.byte_add(SMI_CS).write_volatile(cs) };
    }

    pub fn clear(&mut self) {
        let mut cs = unsafe { self.regs.virt.byte_add(SMI_CS).read_volatile() };
        write_bit_field(&mut cs, SMI_CS_CLEAR, true);
        unsafe { self.regs.virt.byte_add(SMI_CS).write_volatile(cs) };
    }

    pub fn zero(&mut self) {
        let cs = 0;
        unsafe { self.regs.virt.byte_add(SMI_CS).write_volatile(cs) };
    }

    pub fn zero_direct(&mut self) {
        let dcs = 0;
        unsafe { self.regs.virt.byte_add(SMI_DCS).write_volatile(dcs) };
    }

    pub fn set_control(&mut self, control: &Control) {
        let mut dc = 0;
        write_bit_field(&mut dc, SMI_DC_DMAEN, control.dma_enabled);
        write_bit_field(&mut dc, SMI_DC_DMAP, control.external_dreq_mode);
        write_bit_field(&mut dc, SMI_DC_PANICR, control.read_panic_threshold);
        write_bit_field(&mut dc, SMI_DC_PANICW, control.write_panic_threshold);
        write_bit_field(&mut dc, SMI_DC_REQR, control.read_dreq_threshold);
        write_bit_field(&mut dc, SMI_DC_REQW, control.write_dreq_threshold);
        unsafe { self.regs.virt.byte_add(SMI_DC).write_volatile(dc) };
    }

    pub fn set_length(&mut self, length: u32) {
        unsafe { self.regs.virt.byte_add(SMI_L).write_volatile(length) };
    }
}

pub struct Devices {
    pub device0: Device0,
    pub device1: Device1,
    pub device2: Device2,
    pub device3: Device3,
}

pub trait Device {
    const INDEX: u32;

    fn set_read_settings(&mut self, settings: &ReadSettings);
    fn set_write_settings(&mut self, settings: &WriteSettings);
}

macro_rules! device {
    ($name:ident, $index:expr) => {
        pub struct $name {
            dsr_virt: *mut u32,
            dsw_virt: *mut u32,
        }

        impl Device for $name {
            const INDEX: u32 = $index;

            fn set_read_settings(&mut self, settings: &ReadSettings) {
                let mut dsr = 0;
                write_bit_field(
                    &mut dsr,
                    SMI_DSR_RWIDTH,
                    match settings.width {
                        TransferWidth::Bit8 => 0u32,
                        TransferWidth::Bit16 => 1u32,
                        TransferWidth::Bit18 => 2u32,
                        TransferWidth::Bit9 => 3u32,
                    },
                );
                write_bit_field(&mut dsr, SMI_DSR_RSETUP, settings.setup);
                write_bit_field(&mut dsr, SMI_DSR_RSTROBE, settings.strobe);
                write_bit_field(&mut dsr, SMI_DSR_RHOLD, settings.hold);
                write_bit_field(&mut dsr, SMI_DSR_RPACE, settings.pace);
                write_bit_field(&mut dsr, SMI_DSR_RDREQ, settings.dreq);
                unsafe { self.dsr_virt.write_volatile(dsr) };
            }

            fn set_write_settings(&mut self, settings: &WriteSettings) {
                let mut dsw = 0;
                write_bit_field(
                    &mut dsw,
                    SMI_DSW_WWIDTH,
                    match settings.width {
                        TransferWidth::Bit8 => 0u32,
                        TransferWidth::Bit16 => 1u32,
                        TransferWidth::Bit18 => 2u32,
                        TransferWidth::Bit9 => 3u32,
                    },
                );
                write_bit_field(&mut dsw, SMI_DSW_WSETUP, settings.setup);
                write_bit_field(&mut dsw, SMI_DSW_WSTROBE, settings.strobe);
                write_bit_field(&mut dsw, SMI_DSW_WHOLD, settings.hold);
                write_bit_field(&mut dsw, SMI_DSW_WPACE, settings.pace);
                write_bit_field(&mut dsw, SMI_DSW_WDREQ, settings.dreq);
                unsafe { self.dsw_virt.write_volatile(dsw) };
            }
        }
    };
}

pub struct Control {
    pub dma_enabled: bool,
    pub external_dreq_mode: bool,
    pub read_panic_threshold: u8,
    pub write_panic_threshold: u8,
    pub read_dreq_threshold: u8,
    pub write_dreq_threshold: u8,
}

pub struct ReadSettings {
    pub width: TransferWidth,
    pub setup: u8,
    pub strobe: u8,
    pub hold: u8,
    pub pace: u8,
    pub dreq: bool,
}

pub struct WriteSettings {
    pub width: TransferWidth,
    pub setup: u8,
    pub strobe: u8,
    pub hold: u8,
    pub pace: u8,
    pub dreq: bool,
}

pub enum TransferWidth {
    Bit8,
    Bit9,
    Bit16,
    Bit18,
}

pub enum TransferDir {
    Write,
    Read,
}

pub enum ClockSource {
    Gnd,
    Oscillator,
    PllA,
    PllC,
    PllD,
    HdmiAuxiliary,
}

device!(Device0, 0);
device!(Device1, 1);
device!(Device2, 2);
device!(Device3, 3);
