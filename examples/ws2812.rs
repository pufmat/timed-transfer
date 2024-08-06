use std::{
    io,
    sync::mpsc::{self, RecvTimeoutError},
    time::Duration,
};
use timed_transfer::{
    batch, dma,
    gpio::{self, Pin},
    platform, smi, Mailbox,
};

struct Ws2812<'a> {
    transfer: batch::Transfer<'a>,
    data: Vec<u32>,
}

impl<'a> Ws2812<'a> {
    pub fn new(mailbox: &'a Mailbox, len: usize) -> Result<Self, io::Error> {
        let transfer = batch::Transfer::new(mailbox, len * 72)?;
        let mut data = vec![0; transfer.size()];
        data.iter_mut().step_by(3).for_each(|v| *v = 0xffffffff);
        Ok(Self { transfer, data })
    }

    pub fn configure<'b, SmiDevice: smi::Device, DmaChannel: dma::Channel>(
        &'b mut self,
        smi_controller: &'b mut smi::Controller,
        smi_device: &'b mut SmiDevice,
        dma_channel: &'b mut DmaChannel,
    ) -> ConfiguredWs2812<'b, SmiDevice, DmaChannel>
    where
        'a: 'b,
    {
        ConfiguredWs2812 {
            transfer: self.transfer.configure(
                smi_controller,
                smi_device,
                dma_channel,
                Duration::from_nanos(400),
                self.transfer.size(),
            ),
            data: &mut self.data,
        }
    }
}

struct ConfiguredWs2812<'a, SmiDevice: smi::Device, DmaChannel: dma::Channel> {
    transfer: batch::ConfiguredTransfer<'a, SmiDevice, DmaChannel>,
    data: &'a mut [u32],
}

impl<'a, SmiDevice: smi::Device, DmaChannel: dma::Channel>
    ConfiguredWs2812<'a, SmiDevice, DmaChannel>
{
    pub fn len(&self) -> usize {
        self.data.len() / 72
    }

    pub fn set_color(&mut self, strip: usize, led: usize, red: u8, green: u8, blue: u8) {
        assert!(strip < 18);
        self.set_byte(strip, led * 3, green);
        self.set_byte(strip, led * 3 + 1, red);
        self.set_byte(strip, led * 3 + 2, blue);
    }

    fn set_byte(&mut self, strip: usize, index: usize, data: u8) {
        for i in 0..8 {
            let v = &mut self.data[(index * 8 + i) * 3 + 1];
            *v = (*v & !(1 << strip)) | (((data >> i) as u32 & 1) << strip);
        }
    }

    pub fn show(&mut self) {
        self.transfer.set_data(self.data);
        self.transfer.start();
    }
}

fn main() -> Result<(), io::Error> {
    let (tx, rx) = mpsc::channel();

    ctrlc::set_handler(move || tx.send(()).unwrap()).expect("Error setting Ctrl-C handler");

    let platform = platform::RASPBERRY_PI_ZERO_1;

    let mut smi = smi::Peripheral::open(&platform)?;
    let mut dma = dma::Peripheral::open(&platform)?;
    let mut gpio = gpio::Peripheral::open(&platform)?;
    let mailbox = Mailbox::open()?;

    let gpio_pins = &mut gpio.pins;
    gpio_pins.pin8.set_mode(gpio::Mode::Alt1); // SMI pin 0
    gpio_pins.pin9.set_mode(gpio::Mode::Alt1); // SMI pin 1
    gpio_pins.pin10.set_mode(gpio::Mode::Alt1); // SMI pin 2
    gpio_pins.pin11.set_mode(gpio::Mode::Alt1); // SMI pin 3
    gpio_pins.pin12.set_mode(gpio::Mode::Alt1); // SMI pin 4
    gpio_pins.pin13.set_mode(gpio::Mode::Alt1); // SMI pin 5
    gpio_pins.pin14.set_mode(gpio::Mode::Alt1); // SMI pin 6
    gpio_pins.pin15.set_mode(gpio::Mode::Alt1); // SMI pin 7
    gpio_pins.pin16.set_mode(gpio::Mode::Alt1); // SMI pin 8
    gpio_pins.pin17.set_mode(gpio::Mode::Alt1); // SMI pin 9
    gpio_pins.pin18.set_mode(gpio::Mode::Alt1); // SMI pin 10
    gpio_pins.pin19.set_mode(gpio::Mode::Alt1); // SMI pin 11
    gpio_pins.pin20.set_mode(gpio::Mode::Alt1); // SMI pin 12
    gpio_pins.pin21.set_mode(gpio::Mode::Alt1); // SMI pin 13
    gpio_pins.pin22.set_mode(gpio::Mode::Alt1); // SMI pin 14
    gpio_pins.pin23.set_mode(gpio::Mode::Alt1); // SMI pin 15
    gpio_pins.pin24.set_mode(gpio::Mode::Alt1); // SMI pin 16
    gpio_pins.pin25.set_mode(gpio::Mode::Alt1); // SMI pin 17

    // All strips are being transmitted at the same time.
    // Set the number of leds based on the longest strip.
    let mut strips = Ws2812::new(&mailbox, 30)?;

    let mut strips = strips.configure(
        &mut smi.controller,
        &mut smi.devices.device0,
        &mut dma.channels.channel5,
    );

    let mut time = 0;

    let delay = Duration::from_millis(50); // 20Hz
    while let Err(RecvTimeoutError::Timeout) = rx.recv_timeout(delay) {
        // for each strip
        for i in 0..18 {
            // for each led
            for j in 0..strips.len() {
                match i % 6 + 6 * ((time + j) % (3 + i / 6)) {
                    0 => strips.set_color(i, j, 0xff, 0x00, 0x00),
                    1 => strips.set_color(i, j, 0xff, 0xff, 0x00),
                    2 => strips.set_color(i, j, 0x00, 0xff, 0x00),
                    3 => strips.set_color(i, j, 0x00, 0xff, 0xff),
                    4 => strips.set_color(i, j, 0x00, 0x00, 0xff),
                    5 => strips.set_color(i, j, 0xff, 0x00, 0xff),
                    _ => strips.set_color(i, j, 0x00, 0x00, 0x00),
                }
            }
        }
        time += 1;

        strips.show();
    }

    Ok(())
}
