use std::io;

use crate::{
    field::{bit, bits, write_bit_field, Field},
    mem::{map_phys_to_virt, unmap_phys_to_virt, MemMap},
    platform::{Platform, PAGE_SIZE},
};

pub const DMA_OFFSET: usize = 0x00007000;
pub const DMA_ENABLE_OFFSET: usize = 0xFF0;
pub const DMA_CHANNEL_OFFSET: usize = 0x100;

pub const DMA_CONTROL_BLOCK_SIZE: usize = 32;

pub const DMA_CB_TI: usize = 0x00;
pub const DMA_CB_SOURCE_AD: usize = 0x04;
pub const DMA_CB_DEST_AD: usize = 0x08;
pub const DMA_CB_TXFR_LEN: usize = 0x0C;
pub const DMA_CB_STRIDE: usize = 0x10;
pub const DMA_CB_NEXTCONBK: usize = 0x14;

pub const DMA_CS: usize = 0x00;
pub const DMA_CONBLK_AD: usize = 0x04;
pub const DMA_DEBUG: usize = 0x20;

pub const DMA_DEBUG_DMA_ID: Field<u32> = bits(15, 8);

pub const DMA_CS_RESET: Field<u32> = bit(31);
pub const DMA_CS_ABORT: Field<u32> = bit(30);
pub const DMA_CS_DISDEBUG: Field<u32> = bit(29);
pub const DMA_CS_WAIT_FOR_OUTSTANDING_WRITES: Field<u32> = bit(28);
pub const DMA_CS_PANIC_PRIORITY: Field<u32> = bits(23, 20);
pub const DMA_CS_PRIORITY: Field<u32> = bits(19, 16);
pub const DMA_CS_ERROR: Field<u32> = bit(8);
pub const DMA_CS_WAITING_FOR_OUTSTANDING_WRITES: Field<u32> = bit(6);
pub const DMA_CS_DREQ_STOPS_DMA: Field<u32> = bit(5);
pub const DMA_CS_PAUSED: Field<u32> = bit(4);
pub const DMA_CS_DREQ: Field<u32> = bit(3);
pub const DMA_CS_INT: Field<u32> = bit(2);
pub const DMA_CS_END: Field<u32> = bit(1);
pub const DMA_CS_ACTIVE: Field<u32> = bit(0);

pub const DMA_TI_NO_WIDE_BURSTS: Field<u32> = bit(26);
pub const DMA_TI_WAITS: Field<u32> = bits(25, 21);
pub const DMA_TI_PERMAP: Field<u32> = bits(20, 16);
pub const DMA_TI_BURST_LENGTH: Field<u32> = bits(15, 12);
pub const DMA_TI_SRC_IGNORE: Field<u32> = bit(11);
pub const DMA_TI_SRC_DREQ: Field<u32> = bit(10);
pub const DMA_TI_SRC_WIDTH: Field<u32> = bit(9);
pub const DMA_TI_SRC_INC: Field<u32> = bit(8);
pub const DMA_TI_DEST_IGNORE: Field<u32> = bit(7);
pub const DMA_TI_DEST_DREQ: Field<u32> = bit(6);
pub const DMA_TI_DEST_WIDTH: Field<u32> = bit(5);
pub const DMA_TI_DEST_INC: Field<u32> = bit(4);
pub const DMA_TI_WAIT_RESP: Field<u32> = bit(3);
pub const DMA_TI_TDMODE: Field<u32> = bit(1);
pub const DMA_TI_INTEN: Field<u32> = bit(0);

pub const DMA_PERMAP_DSI1: u8 = 1;
pub const DMA_PERMAP_PCM_TX: u8 = 2;
pub const DMA_PERMAP_PCM_RX: u8 = 3;
pub const DMA_PERMAP_SMI: u8 = 4;
pub const DMA_PERMAP_PWM: u8 = 5;
pub const DMA_PERMAP_SPI_TX: u8 = 6;
pub const DMA_PERMAP_SPI_RX: u8 = 7;
pub const DMA_PERMAP_BSC_SPI_SLAVE_TX: u8 = 8;
pub const DMA_PERMAP_BSC_SPI_SLAVE_RX: u8 = 9;
pub const DMA_PERMAP_E_MMC: u8 = 11;
pub const DMA_PERMAP_UART_TX: u8 = 12;
pub const DMA_PERMAP_SD_HOST: u8 = 13;
pub const DMA_PERMAP_UART_RX: u8 = 14;
pub const DMA_PERMAP_DSI2: u8 = 15;
pub const DMA_PERMAP_SLIMBUS_MCTX: u8 = 16;
pub const DMA_PERMAP_HDMI: u8 = 17;
pub const DMA_PERMAP_SLIMBUS_MCRX: u8 = 18;
pub const DMA_PERMAP_SLIMBUS_DC0: u8 = 19;
pub const DMA_PERMAP_SLIMBUS_DC1: u8 = 20;
pub const DMA_PERMAP_SLIMBUS_DC2: u8 = 21;
pub const DMA_PERMAP_SLIMBUS_DC3: u8 = 22;
pub const DMA_PERMAP_SLIMBUS_DC4: u8 = 23;
pub const DMA_PERMAP_SCALER_FIFO_0_AND_SMI: u8 = 24;
pub const DMA_PERMAP_SCALER_FIFO_1_AND_SMI: u8 = 25;
pub const DMA_PERMAP_SCALER_FIFO_2_AND_SMI: u8 = 26;
pub const DMA_PERMAP_SLIMBUS_DC5: u8 = 27;
pub const DMA_PERMAP_SLIMBUS_DC6: u8 = 28;
pub const DMA_PERMAP_SLIMBUS_DC7: u8 = 29;
pub const DMA_PERMAP_SLIMBUS_DC8: u8 = 30;
pub const DMA_PERMAP_SLIMBUS_DC9: u8 = 31;

pub struct Peripheral {
    regs: MemMap,
    pub channels: Channels,
}

impl Peripheral {
    pub fn open(base: &Platform) -> Result<Peripheral, io::Error> {
        let bus = base.bus.wrapping_byte_add(DMA_OFFSET);
        let phys = base.phys.wrapping_byte_add(DMA_OFFSET);
        let virt = unsafe { map_phys_to_virt(phys, PAGE_SIZE) }?;
        let regs = MemMap { bus, phys, virt };

        Ok(Peripheral {
            regs,
            channels: Channels {
                channel0: Channel0::new(&regs),
                channel1: Channel1::new(&regs),
                channel2: Channel2::new(&regs),
                channel3: Channel3::new(&regs),
                channel4: Channel4::new(&regs),
                channel5: Channel5::new(&regs),
                channel6: Channel6::new(&regs),
                channel7: Channel7::new(&regs),
                channel8: Channel8::new(&regs),
                channel9: Channel9::new(&regs),
                channel10: Channel10::new(&regs),
                channel11: Channel11::new(&regs),
                channel12: Channel12::new(&regs),
                channel13: Channel13::new(&regs),
                channel14: Channel14::new(&regs),
            },
        })
    }
}

impl Drop for Peripheral {
    fn drop(&mut self) {
        unsafe { unmap_phys_to_virt(self.regs.virt, PAGE_SIZE) };
    }
}

pub struct Channels {
    pub channel0: Channel0,
    pub channel1: Channel1,
    pub channel2: Channel2,
    pub channel3: Channel3,
    pub channel4: Channel4,
    pub channel5: Channel5,
    pub channel6: Channel6,
    pub channel7: Channel7,
    pub channel8: Channel8,
    pub channel9: Channel9,
    pub channel10: Channel10,
    pub channel11: Channel11,
    pub channel12: Channel12,
    pub channel13: Channel13,
    pub channel14: Channel14,
}

pub trait Channel {
    const INDEX: u32;

    fn enable(&mut self);
    fn disable(&mut self);
    fn set_control_block_address(&mut self, cba: u32);
    fn reset(&mut self);
    fn clear_end(&mut self);
    fn clear_error(&mut self);
    fn start(&mut self);
}

macro_rules! channel {
    ($name:ident, $index:expr) => {
        pub struct $name {
            enable_virt: *mut u32,
            regs: MemMap,
        }

        impl $name {
            fn new(regs: &MemMap) -> Self {
                let offset = $index * DMA_CHANNEL_OFFSET;

                Self {
                    enable_virt: regs.virt.wrapping_byte_add(DMA_ENABLE_OFFSET),
                    regs: MemMap {
                        bus: regs.bus.wrapping_byte_add(offset),
                        phys: regs.phys.wrapping_byte_add(offset),
                        virt: regs.virt.wrapping_byte_add(offset),
                    },
                }
            }
        }

        impl Channel for $name {
            const INDEX: u32 = $index;

            fn enable(&mut self) {
                let mut enable = unsafe { self.enable_virt.read_volatile() };
                enable |= 1 << $index;
                unsafe { self.enable_virt.write_volatile(enable) };
            }

            fn disable(&mut self) {
                let mut enable = unsafe { self.enable_virt.read_volatile() };
                enable &= !(1 << $index);
                unsafe { self.enable_virt.write_volatile(enable) };
            }

            fn set_control_block_address(&mut self, cba: u32) {
                unsafe { self.regs.virt.byte_add(DMA_CONBLK_AD).write_volatile(cba) };
            }

            fn reset(&mut self) {
                let mut cs = unsafe { self.regs.virt.byte_add(DMA_CS).read_volatile() };
                write_bit_field(&mut cs, DMA_CS_RESET, true);
                unsafe { self.regs.virt.byte_add(DMA_CS).write_volatile(cs) };
            }

            fn clear_end(&mut self) {
                let mut cs = unsafe { self.regs.virt.byte_add(DMA_CS).read_volatile() };
                write_bit_field(&mut cs, DMA_CS_END, true);
                unsafe { self.regs.virt.byte_add(DMA_CS).write_volatile(cs) };
            }

            fn clear_error(&mut self) {
                let mut cs = unsafe { self.regs.virt.byte_add(DMA_CS).read_volatile() };
                write_bit_field(&mut cs, DMA_CS_ERROR, true);
                unsafe { self.regs.virt.byte_add(DMA_CS).write_volatile(cs) };
            }

            fn start(&mut self) {
                let mut cs = unsafe { self.regs.virt.byte_add(DMA_CS).read_volatile() };
                write_bit_field(&mut cs, DMA_CS_ACTIVE, true);
                unsafe { self.regs.virt.byte_add(DMA_CS).write_volatile(cs) };
            }
        }
    };
}

channel!(Channel0, 0);
channel!(Channel1, 1);
channel!(Channel2, 2);
channel!(Channel3, 3);
channel!(Channel4, 4);
channel!(Channel5, 5);
channel!(Channel6, 6);
channel!(Channel7, 7);
channel!(Channel8, 8);
channel!(Channel9, 9);
channel!(Channel10, 10);
channel!(Channel11, 11);
channel!(Channel12, 12);
channel!(Channel13, 13);
channel!(Channel14, 14);
