/// Example rust-embedded driver
/// 
/// This includes more options than you'll usually need, and is intended
/// to be adapted (read: have bits removed) according to your use case.

use std::marker::PhantomData;

extern crate embedded_hal;
use embedded_hal::blocking::{delay, spi, i2c};
use embedded_hal::digital::v2::{InputPin, OutputPin};
/// Error type combining SPI, I2C, and Pin errors
/// You can remove anything you don't need / add anything you do
/// (as well as additional driver-specific values) here
#[derive(Debug, Clone, PartialEq)]
pub enum Error<I2cError, SpiError, PinError> {
    /// Underlying SPI device error
    Spi(SpiError),
    /// Underlying I2C device error
    I2c(I2cError),
    /// Underlying GPIO pin error
    Pin(PinError),
    
    /// Device failed to resume from reset
    ResetTimeout
}

/// Driver object is generic over peripheral traits 
/// TODO: Find-and-replace `ExampleDriver` this to match your object
/// 
/// - You probably don't need both I2C and SPI, but they're here to show
///   how they could be used
/// - You should include a unique type for each pin object as some HALs will export different types per-pin or per-bus
/// 
pub struct ExampleDriver<I2c, I2cError, Spi, SpiError, CsPin, BusyPin, ResetPin, PinError, Delay> {
    /// Device configuration
    config: Config,

    /// I2C device
    i2c: I2c,

    /// SPI device
    spi: Spi,

    /// Chip select pin (for SPI)
    /// Technically this _can_ be managed by the HAL, however:
    ///  - often it is not
    ///  - some hals do not expose transactional (write-read) methods
    ///    which are required for interacting with some devices
    /// So at this time it's easier to manage yourself
    cs: CsPin,

    /// Busy input pin
    busy: BusyPin,

    /// Reset output pin
    reset: ResetPin,

    /// Delay implementation
    delay: Delay,

    // Error types must be bound to the object
    _i2c_err: PhantomData<I2cError>,
    _spi_err: PhantomData<SpiError>,
    _pin_err: PhantomData<PinError>,
}

/// Driver configuration data
pub struct Config {
    /// Device polling time
    pub poll_ms: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            poll_ms: 100,
        }
    }
}

/// Device reset timeout
pub const RESET_TIMEOUT_MS: u32 = 100;

impl<I2c, I2cError, Spi, SpiError, CsPin, BusyPin, ResetPin, PinError, Delay> ExampleDriver <I2c, I2cError, Spi, SpiError, CsPin, BusyPin, ResetPin, PinError, Delay>
where
    I2c: i2c::Read<Error = I2cError> + i2c::Write<Error = I2cError>,
    Spi: spi::Transfer<u8, Error = SpiError> + spi::Write<u8, Error = SpiError>,
    CsPin: OutputPin<Error = PinError>,
    BusyPin: InputPin<Error = PinError>,
    ResetPin: OutputPin<Error = PinError>,
    Delay: delay::DelayMs<u32>,
{
    /// Create and initialise a new driver
    pub fn new(config: Config, i2c: I2c, spi: Spi, cs: CsPin, busy: BusyPin, reset: ResetPin, delay: Delay) -> Result<Self, Error<I2cError, SpiError, PinError>> {
        // Create the driver object
        let mut s = Self { 
            config, i2c, spi, cs, busy, reset, delay,
            _i2c_err: PhantomData,
            _spi_err: PhantomData,
            _pin_err: PhantomData,
        };

        // Do some setup
        // note: it's a good idea to check communication here by 
        // reading out a device version register or similar to ensure
        // you're actually talking to the device

        // (example) Reset device
        s.reset.set_low().map_err(|e| Error::Pin(e) )?;
        s.delay.delay_ms(10);
        s.reset.set_high().map_err(|e| Error::Pin(e) )?;

        // (example) Wait on busy
        let mut timeout = 0;
        while s.busy.is_low().map_err(|e| Error::Pin(e) )? {
            // Wait for the poll period
            timeout += s.config.poll_ms;
            s.delay.delay_ms(s.config.poll_ms);

            // Check for timeout
            if timeout > RESET_TIMEOUT_MS {
                return Err(Error::ResetTimeout);
            }
        }

        // (example) Write something to I2C
        s.i2c.write(0x01, &[0x01, 0x02]).map_err(|e| Error::I2c(e) )?;

        // (example) Write something to SPI (using manual CS)
        s.cs.set_low().map_err(|e| Error::Pin(e) )?;
        s.spi.write(&[0x02, 0x03]).map_err(|e| Error::Spi(e) )?;
        s.cs.set_high().map_err(|e| Error::Pin(e) )?;

        // Return the object
        Ok(s)
    }


}