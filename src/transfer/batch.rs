use std::{io, time::Duration};

use crate::{dma, field::write_bit_field, mailbox::Mailbox, smi, GpuMem};

pub struct Transfer<'a> {
    gpu_mem: GpuMem<'a>,
    size: usize,
}

impl<'a> Transfer<'a> {
    pub fn new(mailbox: &'a Mailbox, size: usize) -> Result<Self, io::Error> {
        let byte_size = size * 4;
        let gpu_mem = GpuMem::alloc(mailbox, byte_size + dma::DMA_CONTROL_BLOCK_SIZE)?;

        let mut ti = 0;
        write_bit_field(&mut ti, dma::DMA_TI_DEST_DREQ, true);
        write_bit_field(&mut ti, dma::DMA_TI_SRC_INC, true);
        write_bit_field(&mut ti, dma::DMA_TI_WAIT_RESP, true);
        write_bit_field(&mut ti, dma::DMA_TI_PERMAP, dma::DMA_PERMAP_SMI);

        unsafe {
            let dma_cb_virt = gpu_mem.memmap().virt.byte_add(byte_size);
            dma_cb_virt.byte_add(dma::DMA_CB_TI).write_volatile(ti);
            dma_cb_virt
                .byte_add(dma::DMA_CB_SOURCE_AD)
                .write_volatile(gpu_mem.memmap().bus as u32);
            dma_cb_virt
                .byte_add(dma::DMA_CB_TXFR_LEN)
                .write_volatile(byte_size as u32);
        };

        Ok(Self { gpu_mem, size })
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn set_data(&mut self, data: &[u32]) {
        for i in 0..data.len().min(self.size) {
            unsafe { self.gpu_mem.memmap().virt.add(i).write_volatile(data[i]) }
        }
    }

    pub fn configure<'b, SmiDevice: smi::Device, DmaChannel: dma::Channel>(
        &'b mut self,
        smi_controller: &'b mut smi::Controller,
        smi_device: &'b mut SmiDevice,
        dma_channel: &'b mut DmaChannel,
        duration: Duration,
        size: usize,
    ) -> ConfiguredTransfer<'b, SmiDevice, DmaChannel>
    where
        'a: 'b,
    {
        let size = size.min(self.size);
        let byte_size = size * 4;

        let dma_cb_virt = self.gpu_mem.memmap().virt.wrapping_byte_add(self.size * 4);
        unsafe {
            dma_cb_virt
                .byte_add(dma::DMA_CB_TXFR_LEN)
                .write_volatile(byte_size as u32);
            dma_cb_virt
                .byte_add(dma::DMA_CB_DEST_AD)
                .write_volatile(smi_controller.regs.bus.wrapping_byte_add(smi::SMI_D) as u32);
        };

        let div = duration.as_nanos() / 2; // 1ns * 500Mhz

        // div = (4 + div_transfer) + div_clock
        // (4 + div_transfer) = (1 + div_setup) + (1 + div_strobe) + (1 + div_hold) + (1 + div_pace)
        // div_setup = [0, 63]
        // div_strobe = [0, 127]
        // div_hold = [0, 63]

        let div = div.max(4) - 4;
        let mut div_transfer = (div % 253) as u8;
        let div_clock = (div / 253) as u16;

        let div_setup = div_transfer.clamp(0, 63);
        div_transfer -= div_setup;
        let div_strobe = div_transfer.clamp(0, 127);
        div_transfer -= div_strobe;
        let div_hold = div_transfer.clamp(0, 63);

        smi_device.set_write_settings(&smi::WriteSettings {
            width: smi::TransferWidth::Bit18,
            setup: div_setup,
            strobe: div_strobe,
            hold: div_hold,
            pace: 0,
            dreq: false,
        });

        smi_controller.select(smi_device);
        smi_controller.zero();
        smi_controller.zero_direct();
        smi_controller.disable();
        smi_controller.clear();

        smi_controller.set_clock_divisor(div_clock);
        smi_controller.set_control(&smi::Control {
            dma_enabled: true,
            external_dreq_mode: false,
            read_panic_threshold: 48,
            write_panic_threshold: 16,
            read_dreq_threshold: 32,
            write_dreq_threshold: 32,
        });

        smi_controller.set_length(size as u32);
        smi_controller.set_dir(smi::TransferDir::Write);
        smi_controller.enable();

        dma_channel.enable();

        ConfiguredTransfer {
            gpu_mem: &self.gpu_mem,
            size: self.size,
            smi_controller,
            _smi_device: smi_device,
            dma_channel,
        }
    }
}

pub struct ConfiguredTransfer<'a, SmiDevice: smi::Device, DmaChannel: dma::Channel> {
    gpu_mem: &'a GpuMem<'a>,
    size: usize,
    smi_controller: &'a mut smi::Controller,
    _smi_device: &'a mut SmiDevice,
    dma_channel: &'a mut DmaChannel,
}

impl<'a, SmiDevice: smi::Device, DmaChannel: dma::Channel>
    ConfiguredTransfer<'a, SmiDevice, DmaChannel>
{
    pub fn size(&self) -> usize {
        self.size
    }

    pub fn set_data(&mut self, data: &[u32]) {
        for i in 0..data.len().max(self.size) {
            unsafe { self.gpu_mem.memmap().virt.add(i).write_volatile(data[i]) }
        }
    }

    pub fn start(&mut self) {
        // Prevent interrupting already running transfer.
        while self.smi_controller.active() {}

        self.dma_channel.reset();
        self.dma_channel.set_control_block_address(
            self.gpu_mem.memmap().bus.wrapping_byte_add(self.size * 4) as u32,
        );
        self.dma_channel.clear_end();
        self.dma_channel.clear_error();
        self.dma_channel.start();

        self.smi_controller.start();
    }
}

impl<'a, SmiDevice: smi::Device, DmaChannel: dma::Channel> Drop
    for ConfiguredTransfer<'a, SmiDevice, DmaChannel>
{
    fn drop(&mut self) {
        // Prevent loosing configuration when the transfer is still active.
        while self.smi_controller.active() {}
    }
}
