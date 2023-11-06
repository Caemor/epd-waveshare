use crate::traits::Command;
use core::marker::PhantomData;
use embedded_hal::{
    digital::{ErrorType, InputPin, OutputPin},
    spi::Operation,
};
use embedded_hal_async::{digital::Wait, spi::SpiDevice};
/// The Connection Interface of all (?) Waveshare EPD-Devices
///
/// SINGLE_BYTE_WRITE defines if a data block is written bytewise
/// or blockwise to the spi device
pub(crate) struct DisplayInterface<SPI, BUSY, DC, RST, const SINGLE_BYTE_WRITE: bool> {
    /// SPI
    _spi: PhantomData<SPI>,
    /// Low for busy, Wait until display is ready!
    busy: BUSY,
    /// Data/Command Control Pin (High for data, Low for command)
    dc: DC,
    /// Pin for Resetting
    rst: RST,
    /// number of ms the idle loop should sleep on
    delay_us: u32,
}

impl<SPI, BUSY, DC, RST, const SINGLE_BYTE_WRITE: bool>
    DisplayInterface<SPI, BUSY, DC, RST, SINGLE_BYTE_WRITE>
where
    SPI: SpiDevice,
    BUSY: InputPin + Wait,
    DC: OutputPin,
    RST: OutputPin,
{
    /// Creates a new `DisplayInterface` struct
    ///
    /// If no delay is given, a default delay of 10ms is used.
    pub fn new(busy: BUSY, dc: DC, rst: RST, delay_us: Option<u32>) -> Self {
        // default delay of 10ms
        let delay_us = delay_us.unwrap_or(10_000);
        DisplayInterface {
            _spi: PhantomData,
            busy,
            dc,
            rst,
            delay_us,
        }
    }

    /// Basic function for sending [Commands](Command).
    ///
    /// Enables direct interaction with the device with the help of [data()](DisplayInterface::data())
    pub(crate) async fn cmd<T: Command>(
        &mut self,
        spi: &mut SPI,
        command: T,
    ) -> Result<(), SPI::Error> {
        // low for commands
        let _ = self.dc.set_low();

        // Transfer the command over spi
        self.write(spi, &[command.address()]).await
    }

    /// Basic function for sending an array of u8-values of data over spi
    ///
    /// Enables direct interaction with the device with the help of [command()](Epd4in2::command())
    pub(crate) async fn data(&mut self, spi: &mut SPI, data: &[u8]) -> Result<(), SPI::Error> {
        // high for data
        let _ = self.dc.set_high();

        if SINGLE_BYTE_WRITE {
            for val in data.iter().copied() {
                // Transfer data one u8 at a time over spi
                self.write(spi, &[val]).await?;
            }
        } else {
            self.write(spi, data).await?;
        }

        Ok(())
    }

    /// Basic function for sending [Commands](Command) and the data belonging to it.
    ///
    /// TODO: directly use ::write? cs wouldn't needed to be changed twice than
    pub(crate) async fn cmd_with_data<T: Command>(
        &mut self,
        spi: &mut SPI,
        command: T,
        data: &[u8],
    ) -> Result<(), SPI::Error> {
        self.cmd(spi, command).await?;
        self.data(spi, data).await
    }

    /// Basic function for sending the same byte of data (one u8) multiple times over spi
    ///
    /// Enables direct interaction with the device with the help of [command()](ConnectionInterface::command())
    pub(crate) async fn data_x_times(
        &mut self,
        spi: &mut SPI,
        val: u8,
        repetitions: u32,
    ) -> Result<(), SPI::Error> {
        // high for data
        let _ = self.dc.set_high();
        // Transfer data (u8) over spi
        for _ in 0..repetitions {
            self.write(spi, &[val]).await?;
        }
        Ok(())
    }

    // spi write helper/abstraction function
    async fn write(&mut self, spi: &mut SPI, data: &[u8]) -> Result<(), SPI::Error> {
        // transfer spi data
        // Be careful!! Linux has a default limit of 4096 bytes per spi transfer
        // see https://raspberrypi.stackexchange.com/questions/65595/spi-transfer-fails-with-buffer-size-greater-than-4096
        if cfg!(target_os = "linux") {
            for data_chunk in data.chunks(4096) {
                spi.write(data_chunk).await?;
            }
            Ok(())
        } else {
            spi.write(data).await
        }
    }

    /// Waits until device isn't busy anymore (busy == HIGH)
    ///
    /// This is normally handled by the more complicated commands themselves,
    /// but in the case you send data and commands directly you might need to check
    /// if the device is still busy
    ///
    /// is_busy_low
    ///
    ///  - TRUE for epd4in2, epd2in13, epd2in7, epd5in83, epd7in5
    ///  - FALSE for epd2in9, epd1in54 (for all Display Type A ones?)
    ///
    /// Most likely there was a mistake with the 2in9 busy connection
    pub(crate) async fn wait_until_idle(
        &mut self,
        spi: &mut SPI,
        is_busy_low: bool,
    ) -> Result<(), BUSY::Error> {
        if is_busy_low {
            self.busy.wait_for_high().await?;
        } else {
            self.busy.wait_for_low().await?;
        }

        // TODO: remove
        if self.delay_us > 0 {
            self.delay(spi, self.delay_us).await;
        }
        Ok(())
    }

    /// Same as `wait_until_idle` for device needing a command to probe Busy pin
    pub(crate) async fn wait_until_idle_with_cmd<T: Command>(
        &mut self,
        spi: &mut SPI,
        is_busy_low: bool,
        status_command: T,
    ) -> Result<(), SPI::Error> {
        self.cmd(spi, status_command).await?;
        if self.delay_us > 0 {
            self.delay(spi, self.delay_us).await?;
        }

        // TODO
        while self.is_busy(is_busy_low) {
            self.cmd(spi, status_command).await?;
            if self.delay_us > 0 {
                self.delay(spi, self.delay_us).await?;
            }
        }
        Ok(())
    }

    pub(crate) async fn delay(&mut self, spi: &mut SPI, duration: u32) -> Result<(), SPI::Error> {
        spi.transaction(&mut [Operation::DelayUs(duration)]).await
    }

    /// Checks if device is still busy
    ///
    /// This is normally handled by the more complicated commands themselves,
    /// but in the case you send data and commands directly you might need to check
    /// if the device is still busy
    ///
    /// is_busy_low
    ///
    ///  - TRUE for epd4in2, epd2in13, epd2in7, epd5in83, epd7in5
    ///  - FALSE for epd2in9, epd1in54 (for all Display Type A ones?)
    ///
    /// Most likely there was a mistake with the 2in9 busy connection
    /// //TODO: use the #cfg feature to make this compile the right way for the certain types
    pub(crate) fn is_busy(&self, is_busy_low: bool) -> bool {
        (is_busy_low && self.busy.is_low().unwrap_or(false))
            || (!is_busy_low && self.busy.is_high().unwrap_or(false))
    }

    /// Resets the device.
    ///
    /// Often used to awake the module from deep sleep. See [Epd4in2::sleep()](Epd4in2::sleep())
    ///
    /// The timing of keeping the reset pin low seems to be important and different per device.
    /// Most displays seem to require keeping it low for 10ms, but the 7in5_v2 only seems to reset
    /// properly with 2ms
    pub(crate) async fn reset(
        &mut self,
        spi: &mut SPI,
        initial_delay: u32,
        duration: u32,
    ) {
        let _ = self.rst.set_high();
        self.delay(spi, initial_delay).await;

        let _ = self.rst.set_low();
        self.delay(spi, duration).await;
        let _ = self.rst.set_high();
        //TODO: the upstream libraries always sleep for 200ms here
        // 10ms works fine with just for the 7in5_v2 but this needs to be validated for other devices
        self.delay(spi, 200_000).await;
    }
}
