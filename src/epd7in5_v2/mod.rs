//! A simple Driver for the Waveshare 7.5" E-Ink Display (V2) via SPI
//!
//! # References
//!
//! - [Datasheet](https://www.waveshare.com/wiki/7.5inch_e-Paper_HAT)
//! - [Waveshare C driver](https://github.com/waveshare/e-Paper/blob/702def0/RaspberryPi%26JetsonNano/c/lib/e-Paper/EPD_7in5_V2.c)
//! - [Waveshare Python driver](https://github.com/waveshare/e-Paper/blob/702def0/RaspberryPi%26JetsonNano/python/lib/waveshare_epd/epd7in5_V2.py)
//!
//! Important note for V2:
//! Revision V2 has been released on 2019.11, the resolution is upgraded to 800×480, from 640×384 of V1.
//! The hardware and interface of V2 are compatible with V1, however, the related software should be updated.

use embedded_hal::{
    blocking::{delay::*, spi::Write},
    digital::v2::{InputPin, OutputPin},
};

use crate::color::Color;
use crate::interface::DisplayInterface;
use crate::traits::{InternalWiAdditions, RefreshLut, WaveshareDisplay};

pub(crate) mod command;
use self::command::Command;
use crate::buffer_len;

/// Full size buffer for use with the 7in5 v2 EPD
#[cfg(feature = "graphics")]
pub type Display7in5 = crate::graphics::Display<
    WIDTH,
    HEIGHT,
    false,
    { buffer_len(WIDTH as usize, HEIGHT as usize) },
    Color,
>;

/// Width of the display
pub const WIDTH: u32 = 800;
/// Height of the display
pub const HEIGHT: u32 = 480;
/// Default Background Color
pub const DEFAULT_BACKGROUND_COLOR: Color = Color::White;
const IS_BUSY_LOW: bool = true;

/// Epd7in5 (V2) driver
///
pub struct Epd7in5<SPI, CS, BUSY, DC, RST, DELAY> {
    /// Connection Interface
    interface: DisplayInterface<SPI, CS, BUSY, DC, RST, DELAY>,
}

impl<SPI, CS, BUSY, DC, RST, DELAY> InternalWiAdditions<SPI, CS, BUSY, DC, RST, DELAY>
    for Epd7in5<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    fn init(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        // Reset the device
        self.interface.reset(delay, 10, 2);

        // V2 procedure as described here:
        // https://github.com/waveshare/e-Paper/blob/master/RaspberryPi%26JetsonNano/python/lib/waveshare_epd/epd7in5bc_V2.py
        // and as per specs:
        // https://www.waveshare.com/w/upload/6/60/7.5inch_e-Paper_V2_Specification.pdf

        self.cmd_with_data(spi, Command::BoosterSoftStart, &[0x17, 0x17, 0x27, 0x17])?;
        self.cmd_with_data(spi, Command::PowerSetting, &[0x07, 0x17, 0x3F, 0x3F])?;
        self.command(spi, Command::PowerOn)?;
        self.wait_until_idle(spi, delay)?;
        self.cmd_with_data(spi, Command::PanelSetting, &[0x1F])?;
        self.cmd_with_data(spi, Command::PllControl, &[0x06])?;
        self.cmd_with_data(spi, Command::TconResolution, &[0x03, 0x20, 0x01, 0xE0])?;
        self.cmd_with_data(spi, Command::DualSpi, &[0x00])?;
        self.cmd_with_data(spi, Command::TconSetting, &[0x22])?;
        self.cmd_with_data(spi, Command::VcomAndDataIntervalSetting, &[0x10, 0x07])?;
        self.wait_until_idle(spi, delay)?;
        Ok(())
    }
}

impl<SPI, CS, BUSY, DC, RST, DELAY> WaveshareDisplay<SPI, CS, BUSY, DC, RST, DELAY>
    for Epd7in5<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    type DisplayColor = Color;
    fn new(
        spi: &mut SPI,
        cs: CS,
        busy: BUSY,
        dc: DC,
        rst: RST,
        delay: &mut DELAY,
    ) -> Result<Self, SPI::Error> {
        let interface = DisplayInterface::new(cs, busy, dc, rst);

        let mut epd = Epd7in5 { interface };

        epd.init(spi, delay)?;

        Ok(epd)
    }

    fn wake_up(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.init(spi, delay)
    }

    fn sleep(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.wait_until_idle(spi, delay)?;
        self.command(spi, Command::PowerOff)?;
        self.wait_until_idle(spi, delay)?;
        self.cmd_with_data(spi, Command::DeepSleep, &[0xA5])?;
        Ok(())
    }

    fn update_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.wait_until_idle(spi, delay)?;
        self.cmd_with_data(spi, Command::DataStartTransmission2, buffer)?;
        Ok(())
    }

    fn update_partial_frame(
        &mut self,
        _spi: &mut SPI,
        _buffer: &[u8],
        _x: u32,
        _y: u32,
        _width: u32,
        _height: u32,
    ) -> Result<(), SPI::Error> {
        unimplemented!();
    }

    fn display_frame(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.wait_until_idle(spi, delay)?;
        self.command(spi, Command::DisplayRefresh)?;
        Ok(())
    }

    fn update_and_display_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.update_frame(spi, buffer, delay)?;
        self.command(spi, Command::DisplayRefresh)?;
        Ok(())
    }

    fn width(&self) -> u32 {
        WIDTH
    }

    fn height(&self) -> u32 {
        HEIGHT
    }

    fn set_lut(
        &mut self,
        _spi: &mut SPI,
        _refresh_rate: Option<RefreshLut>,
    ) -> Result<(), SPI::Error> {
        unimplemented!();
    }

    fn is_busy(&self) -> bool {
        self.interface.is_busy(IS_BUSY_LOW)
    }
}

impl<SPI, CS, BUSY, DC, RST, DELAY> Epd7in5<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    fn command(&mut self, spi: &mut SPI, command: Command) -> Result<(), SPI::Error> {
        self.interface.cmd(spi, command)
    }

    fn cmd_with_data(
        &mut self,
        spi: &mut SPI,
        command: Command,
        data: &[u8],
    ) -> Result<(), SPI::Error> {
        self.interface.cmd_with_data(spi, command, data)
    }

    fn wait_until_idle(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        while self.interface.is_busy(IS_BUSY_LOW) {
            self.interface.cmd(spi, Command::GetStatus)?;
            delay.delay_ms(20);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epd_size() {
        assert_eq!(WIDTH, 800);
        assert_eq!(HEIGHT, 480);
        assert_eq!(DEFAULT_BACKGROUND_COLOR, Color::White);
    }
}
