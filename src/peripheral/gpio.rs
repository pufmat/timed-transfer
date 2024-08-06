use std::io;

use crate::{
    map_phys_to_virt,
    platform::{Platform, PAGE_SIZE},
    unmap_phys_to_virt, MemMap,
};

pub const GPIO_OFFSET: usize = 0x00200000;

pub struct Peripheral {
    regs: MemMap,
    pub pins: Pins,
}

impl Peripheral {
    pub fn open(base: &Platform) -> Result<Peripheral, io::Error> {
        let bus = base.bus.wrapping_byte_add(GPIO_OFFSET);
        let phys = base.phys.wrapping_byte_add(GPIO_OFFSET);
        let virt = unsafe { map_phys_to_virt(phys, PAGE_SIZE) }?;
        let regs = MemMap { bus, phys, virt };

        Ok(Peripheral {
            regs,
            pins: Pins {
                pin0: Pin0::new(&regs),
                pin1: Pin1::new(&regs),
                pin2: Pin2::new(&regs),
                pin3: Pin3::new(&regs),
                pin4: Pin4::new(&regs),
                pin5: Pin5::new(&regs),
                pin6: Pin6::new(&regs),
                pin7: Pin7::new(&regs),
                pin8: Pin8::new(&regs),
                pin9: Pin9::new(&regs),
                pin10: Pin10::new(&regs),
                pin11: Pin11::new(&regs),
                pin12: Pin12::new(&regs),
                pin13: Pin13::new(&regs),
                pin14: Pin14::new(&regs),
                pin15: Pin15::new(&regs),
                pin16: Pin16::new(&regs),
                pin17: Pin17::new(&regs),
                pin18: Pin18::new(&regs),
                pin19: Pin19::new(&regs),
                pin20: Pin20::new(&regs),
                pin21: Pin21::new(&regs),
                pin22: Pin22::new(&regs),
                pin23: Pin23::new(&regs),
                pin24: Pin24::new(&regs),
                pin25: Pin25::new(&regs),
                pin26: Pin26::new(&regs),
                pin27: Pin27::new(&regs),
            },
        })
    }
}

impl Drop for Peripheral {
    fn drop(&mut self) {
        unsafe { unmap_phys_to_virt(self.regs.virt, PAGE_SIZE) };
    }
}

pub struct Pins {
    pub pin0: Pin0,
    pub pin1: Pin1,
    pub pin2: Pin2,
    pub pin3: Pin3,
    pub pin4: Pin4,
    pub pin5: Pin5,
    pub pin6: Pin6,
    pub pin7: Pin7,
    pub pin8: Pin8,
    pub pin9: Pin9,
    pub pin10: Pin10,
    pub pin11: Pin11,
    pub pin12: Pin12,
    pub pin13: Pin13,
    pub pin14: Pin14,
    pub pin15: Pin15,
    pub pin16: Pin16,
    pub pin17: Pin17,
    pub pin18: Pin18,
    pub pin19: Pin19,
    pub pin20: Pin20,
    pub pin21: Pin21,
    pub pin22: Pin22,
    pub pin23: Pin23,
    pub pin24: Pin24,
    pub pin25: Pin25,
    pub pin26: Pin26,
    pub pin27: Pin27,
}

pub trait Pin {
    const INDEX: u32;

    fn set_mode(&mut self, mode: Mode);
}

macro_rules! pin {
    ($name:ident, $index:expr) => {
        pub struct $name {
            regs: MemMap,
        }

        impl $name {
            fn new(regs: &MemMap) -> Self {
                Self { regs: *regs }
            }
        }

        impl Pin for $name {
            const INDEX: u32 = $index;

            fn set_mode(&mut self, mode: Mode) {
                let offset = (($index / 10) * 4) as usize;
                let shift = ($index % 10) * 3;
                let mode = match mode {
                    Mode::Input => 0b000u32,
                    Mode::Output => 0b001u32,
                    Mode::Alt0 => 0b100u32,
                    Mode::Alt1 => 0b101u32,
                    Mode::Alt2 => 0b110u32,
                    Mode::Alt3 => 0b111u32,
                    Mode::Alt4 => 0b011u32,
                    Mode::Alt5 => 0b010u32,
                };

                let mut gpio = unsafe { self.regs.virt.byte_add(offset).read_volatile() };
                gpio = (gpio & !(0b111 << shift)) | (mode << shift);
                unsafe { self.regs.virt.byte_add(offset).write_volatile(gpio) };
            }
        }
    };
}

pub enum Mode {
    Input,
    Output,
    Alt0,
    Alt1,
    Alt2,
    Alt3,
    Alt4,
    Alt5,
}

pin!(Pin0, 0);
pin!(Pin1, 1);
pin!(Pin2, 2);
pin!(Pin3, 3);
pin!(Pin4, 4);
pin!(Pin5, 5);
pin!(Pin6, 6);
pin!(Pin7, 7);
pin!(Pin8, 8);
pin!(Pin9, 9);
pin!(Pin10, 10);
pin!(Pin11, 11);
pin!(Pin12, 12);
pin!(Pin13, 13);
pin!(Pin14, 14);
pin!(Pin15, 15);
pin!(Pin16, 16);
pin!(Pin17, 17);
pin!(Pin18, 18);
pin!(Pin19, 19);
pin!(Pin20, 20);
pin!(Pin21, 21);
pin!(Pin22, 22);
pin!(Pin23, 23);
pin!(Pin24, 24);
pin!(Pin25, 25);
pin!(Pin26, 26);
pin!(Pin27, 27);
