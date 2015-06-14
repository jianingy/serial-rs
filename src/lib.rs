extern crate time;

use std::default::Default;
use std::error::Error as StdError;
use std::ffi::OsStr;
use std::fmt;
use std::io;

use time::Duration;

pub use BaudRate::*;
pub use CharSize::*;
pub use Parity::*;
pub use StopBits::*;
pub use FlowControl::*;

/// A module that exports traits that are useful to have in scope.
///
/// It is intended to be glob imported:
///
/// ```no_run
/// use serial::prelude::*;
/// ```
pub mod prelude {
    pub use ::{SerialPort,SerialPortSettings};
}

#[cfg(unix)]
pub mod posix;

#[cfg(windows)]
pub mod windows;


/// A type for results generated by interacting with serial ports.
///
/// The `Err` type is hard-wired to [`serial::Error`](struct.Error.html).
pub type Result<T> = std::result::Result<T,::Error>;

/// Categories of errors that can occur when interacting with serial ports.
///
/// This list is intended to grow over time and it is not recommended to exhaustively match against it.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ErrorKind {
    /// The device is not available.
    ///
    /// This could indicate that the device is in use by another process or was disconnected while
    /// performing I/O.
    NoDevice,

    /// A parameter was incorrect.
    InvalidInput,

    /// An I/O error occured.
    ///
    /// The type of I/O error is determined by the inner `io::ErrorKind`.
    Io(io::ErrorKind)
}

/// An error type for serial port operations.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    description: String
}

impl Error {
    pub fn new<T: Into<String>>(kind: ErrorKind, description: T) -> Self {
        Error {
            kind: kind,
            description: description.into()
        }
    }

    /// Returns the corresponding `ErrorKind` for this error.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        fmt.write_str(&self.description)
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        &self.description
    }
}

impl From<io::Error> for Error {
    fn from(io_error: io::Error) -> Error {
        Error::new(ErrorKind::Io(io_error.kind()), format!("{}", io_error))
    }
}

impl From<Error> for io::Error {
    fn from(error: Error) -> io::Error {
        let kind = match error.kind {
            ErrorKind::NoDevice => io::ErrorKind::NotFound,
            ErrorKind::InvalidInput => io::ErrorKind::InvalidInput,
            ErrorKind::Io(kind) => kind
        };

        io::Error::new(kind, error.description)
    }
}


/// A convenience function for opening a native serial port.
///
/// The argument must be one that's understood by the target operating system to identify a serial
/// port. On Unix systems, it should be a path to a TTY device file. On Windows, it should be the
/// name of a COM port.
///
/// ## Errors
///
/// This function returns an error if the device could not be opened and initialized:
///
/// * `NoDevice` if the device could not be opened. This could indicate that the device is
///   already in use.
/// * `InvalidInput` if `port` is not a valid device name.
/// * `Io` for any other error while opening or initializing the device.
///
/// ## Examples
///
/// Provide a system-specific string that identifies a serial port:
///
/// ```no_run
/// let port = serial::open("/dev/ttyUSB0").unwrap();
/// ```
///
/// Hard-coding the device name dimishes the cross-platform utility of `serial::open()`. To
/// preserve cross-platform functionality, device names should come from external sources:
///
/// ```no_run
/// use std::env;
///
/// for arg in env::args_os().skip(1) {
///     let port = serial::open(&arg).unwrap();
/// }
/// ```
#[cfg(unix)]
pub fn open<T: AsRef<OsStr> + ?Sized>(port: &T) -> ::Result<posix::TTYPort> {
    use std::path::Path;
    posix::TTYPort::open(Path::new(port))
}

/// A convenience function for opening a native serial port.
///
/// The argument must be one that's understood by the target operating system to identify a serial
/// port. On Unix systems, it should be a path to a TTY device file. On Windows, it should be the
/// name of a COM port.
///
/// ## Errors
///
/// This function returns an error if the device could not be opened and initialized:
///
/// * `NoDevice` if the device could not be opened. This could indicate that the device is
///   already in use.
/// * `InvalidInput` if `port` is not a valid device name.
/// * `Io` for any other error while opening or initializing the device.
///
/// ## Examples
///
/// Provide a system-specific string that identifies a serial port:
///
/// ```no_run
/// let port = serial::open("COM1").unwrap();
/// ```
///
/// Hard-coding the device name dimishes the cross-platform utility of `serial::open()`. To
/// preserve cross-platform functionality, device names should come from external sources:
///
/// ```no_run
/// use std::env;
///
/// for arg in env::args_os().skip(1) {
///     let port = serial::open(&arg).unwrap();
/// }
/// ```
#[cfg(windows)]
pub fn open<T: AsRef<OsStr> + ?Sized>(port: &T) -> ::Result<windows::COMPort> {
    windows::COMPort::open(port)
}


/// Serial port baud rates.
#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum BaudRate {
    /** 110 baud. */     Baud110,
    /** 300 baud. */     Baud300,
    /** 600 baud. */     Baud600,
    /** 1200 baud. */    Baud1200,
    /** 2400 baud. */    Baud2400,
    /** 4800 baud. */    Baud4800,
    /** 9600 baud. */    Baud9600,
    /** 19,200 baud. */  Baud19200,
    /** 38,400 baud. */  Baud38400,
    /** 57,600 baud. */  Baud57600,
    /** 115,200 baud. */ Baud115200,

    /// Non-standard baud rates.
    ///
    /// `BaudOther` can be used to set arbitrary baud rates by setting its member to be the desired
    /// baud rate.
    ///
    /// ```no_run
    /// serial::BaudOther(4_000_000); // 4,000,000 baud
    /// ```
    ///
    /// Non-standard baud rates may not be supported by all hardware.
    BaudOther(usize)
}

/// Number of bits per character.
#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum CharSize {
    /** 5 bits per character. */ Bits5,
    /** 6 bits per character. */ Bits6,
    /** 7 bits per character. */ Bits7,
    /** 8 bits per character. */ Bits8
}

/// Parity checking modes.
///
/// When parity checking is enabled (`ParityOdd` or `ParityEven`) an extra bit is transmitted with
/// each character. The value of the parity bit is arranged so that the number of 1 bits in the
/// character (including the parity bit) is an even number (`ParityEven`) or an odd number
/// (`ParityOdd`).
///
/// Parity checking is disabled by setting `ParityNone`, in which case parity bits are not
/// transmitted.
#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum Parity {
    /// No parity bit.
    ParityNone,

    /// Parity bit sets odd number of 1 bits.
    ParityOdd,

    /// Parity bit sets even number of 1 bits.
    ParityEven
}

/// Number of stop bits.
///
/// Stop bits are transmitted after every character.
#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum StopBits {
    /// One stop bit.
    Stop1,

    /// Two stop bits.
    Stop2
}

/// Flow control modes.
#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum FlowControl {
    /// No flow control.
    FlowNone,

    /// Flow control using XON/XOFF bytes.
    FlowSoftware,

    /// Flow control using RTS/CTS signals.
    FlowHardware
}

/// A trait for implementing serial devices.
///
/// This trait is meant to be used to implement new serial port devices. To use a serial port
/// device, the [`SerialPort`](trait.SerialPort.html) trait should be used instead. Any type that
/// implements the `SerialDevice` trait will automatically implement the `SerialPort` trait as
/// well.
///
/// To implement a new serial port device, it's necessary to define a type that can manipulate the
/// serial port device's settings (baud rate, parity mode, etc). This type is defined by the
/// `Settings` associated type. The current settings should be determined by reading from the
/// hardware or operating system for every call to `read_settings()`. The settings can then be
/// manipulated in memory before being commited to the device with `write_settings()`.
///
/// Types that implement `SerialDevice` must also implement `std::io::Read` and `std::io::Write`.
/// The `read()` and `write()` operations of these traits should honor the timeout that has been
/// set with the most recent successful call to `set_timeout()`. This timeout value should also be
/// accessible by calling the `timeout()` method.
///
/// Lastly, a serial port device should provide access to some basic control signals: RTS, DTR,
/// CTS, DSR, RI, and CD. The values for the control signals are represented as boolean values,
/// with `true` indicating the the control signal is active.
pub trait SerialDevice: io::Read+io::Write {
    /// A type that implements the settings for the serial port device.
    ///
    /// The `Settings` type is used to retrieve and modify the serial port's settings. This type
    /// should own any native structures used to manipulate the device's settings, but it should
    /// not cause any changes in the underlying hardware until written to the device with
    /// `write_settings()`.
    type Settings: SerialPortSettings;

    /// Returns the device's current settings.
    ///
    /// This function attempts to read the current settings from the hardware. The hardware's
    /// current settings may not match the settings that were most recently written to the hardware
    /// with `write_settings()`.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the settings could not be read from the underlying
    /// hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn read_settings(&self) -> ::Result<Self::Settings>;

    /// Applies new settings to the serial device.
    ///
    /// This function attempts to apply all settings to the serial device. Some settings may not be
    /// supported by the underlying hardware, in which case the result is dependent on the
    /// implementation. A successful return value does not guarantee that all settings were
    /// appliied successfully. To check which settings were applied by a successful write,
    /// applications should use the `read_settings()` method to obtain the latest configuration
    /// state from the device.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the settings could not be applied to the underlying
    /// hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `InvalidInput` if a setting is not compatible with the underlying hardware.
    /// * `Io` for any other type of I/O error.
    fn write_settings(&mut self, settings: &Self::Settings) -> ::Result<()>;

    /// Returns the current timeout.
    fn timeout(&self) -> Duration;

    /// Sets the timeout for future I/O operations.
    fn set_timeout(&mut self, timeout: Duration) -> ::Result<()>;

    /// Sets the state of the RTS (Request To Send) control signal.
    ///
    /// Setting a value of `true` asserts the RTS control signal. `false` clears the signal.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the RTS control signal could not be set to the desired
    /// state on the underlying hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn set_rts(&mut self, level: bool) -> ::Result<()>;

    /// Sets the state of the DTR (Data Terminal Ready) control signal.
    ///
    /// Setting a value of `true` asserts the DTR control signal. `false` clears the signal.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the DTR control signal could not be set to the desired
    /// state on the underlying hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn set_dtr(&mut self, level: bool) -> ::Result<()>;

    /// Reads the state of the CTS (Clear To Send) control signal.
    ///
    /// This function returns a boolean that indicates whether the CTS control signal is asserted.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the state of the CTS control signal could not be read
    /// from the underlying hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn read_cts(&mut self) -> ::Result<bool>;

    /// Reads the state of the DSR (Data Set Ready) control signal.
    ///
    /// This function returns a boolean that indicates whether the DSR control signal is asserted.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the state of the DSR control signal could not be read
    /// from the underlying hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn read_dsr(&mut self) -> ::Result<bool>;

    /// Reads the state of the RI (Ring Indicator) control signal.
    ///
    /// This function returns a boolean that indicates whether the RI control signal is asserted.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the state of the RI control signal could not be read from
    /// the underlying hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn read_ri(&mut self) -> ::Result<bool>;

    /// Reads the state of the CD (Carrier Detect) control signal.
    ///
    /// This function returns a boolean that indicates whether the CD control signal is asserted.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the state of the CD control signal could not be read from
    /// the underlying hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn read_cd(&mut self) -> ::Result<bool>;
}

/// A trait for serial port devices.
///
/// Serial port input and output is implemented through the `std::io::Read` and `std::io::Write`
/// traits. A timeout can be set with the `set_timeout()` method and applies to all subsequent I/O
/// operations.
///
/// The `SerialPort` trait exposes several common control signals. Each control signal is
/// represented as a boolean, where `true` indicates that the signal is asserted.
pub trait SerialPort: io::Read+io::Write {
    /// Returns the current timeout.
    fn timeout(&self) -> Duration;

    /// Sets the timeout for future I/O operations.
    fn set_timeout(&mut self, timeout: Duration) -> ::Result<()>;

    /// Configures a serial port device.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the settings could not be applied to the underlying
    /// hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `InvalidInput` if a setting is not compatible with the underlying hardware.
    /// * `Io` for any other type of I/O error.
    fn configure(&mut self, settings: &PortSettings) -> ::Result<()>;

    /// Alter the serial port's configuration.
    ///
    /// This method expects a function, which takes a mutable reference to the serial port's
    /// configuration settings. The serial port's current settings, read from the device, are
    /// yielded to the provided function. After the function returns, any changes made to the
    /// settings object will be written back to the device.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the `setup` function returns an error or if there was an
    /// error while reading or writing the device's configuration settings:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `InvalidInput` if a setting is not compatible with the underlying hardware.
    /// * `Io` for any other type of I/O error.
    /// * Any error returned by the `setup` function.
    ///
    /// ## Example
    ///
    /// The following is a function that toggles a serial port's settings between one and two stop
    /// bits:
    ///
    /// ```no_run
    /// use std::io;
    /// use serial::prelude::*;
    ///
    /// fn toggle_stop_bits<T: SerialPort>(port: &mut T) -> serial::Result<()> {
    ///     port.reconfigure(&|settings| {
    ///         let stop_bits = match settings.stop_bits() {
    ///             Some(serial::Stop1)        => serial::Stop2,
    ///             Some(serial::Stop2) | None => serial::Stop1
    ///         };
    ///
    ///         settings.set_stop_bits(stop_bits);
    ///         Ok(())
    ///     })
    /// }
    /// ```
    fn reconfigure(&mut self, setup: &Fn (&mut SerialPortSettings) -> ::Result<()>) -> ::Result<()>;

    /// Sets the state of the RTS (Request To Send) control signal.
    ///
    /// Setting a value of `true` asserts the RTS control signal. `false` clears the signal.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the RTS control signal could not be set to the desired
    /// state on the underlying hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn set_rts(&mut self, level: bool) -> ::Result<()>;

    /// Sets the state of the DTR (Data Terminal Ready) control signal.
    ///
    /// Setting a value of `true` asserts the DTR control signal. `false` clears the signal.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the DTR control signal could not be set to the desired
    /// state on the underlying hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn set_dtr(&mut self, level: bool) -> ::Result<()>;

    /// Reads the state of the CTS (Clear To Send) control signal.
    ///
    /// This function returns a boolean that indicates whether the CTS control signal is asserted.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the state of the CTS control signal could not be read
    /// from the underlying hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn read_cts(&mut self) -> ::Result<bool>;

    /// Reads the state of the DSR (Data Set Ready) control signal.
    ///
    /// This function returns a boolean that indicates whether the DSR control signal is asserted.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the state of the DSR control signal could not be read
    /// from the underlying hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn read_dsr(&mut self) -> ::Result<bool>;

    /// Reads the state of the RI (Ring Indicator) control signal.
    ///
    /// This function returns a boolean that indicates whether the RI control signal is asserted.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the state of the RI control signal could not be read from
    /// the underlying hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn read_ri(&mut self) -> ::Result<bool>;

    /// Reads the state of the CD (Carrier Detect) control signal.
    ///
    /// This function returns a boolean that indicates whether the CD control signal is asserted.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the state of the CD control signal could not be read from
    /// the underlying hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn read_cd(&mut self) -> ::Result<bool>;
}

impl<T> SerialPort for T where T: SerialDevice {
    fn timeout(&self) -> Duration {
        T::timeout(self)
    }

    fn set_timeout(&mut self, timeout: Duration) -> ::Result<()> {
        T::set_timeout(self, timeout)
    }

    fn configure(&mut self, settings: &PortSettings) -> ::Result<()> {
        let mut device_settings = try!(T::read_settings(self));

        try!(device_settings.set_baud_rate(settings.baud_rate));
        device_settings.set_char_size(settings.char_size);
        device_settings.set_parity(settings.parity);
        device_settings.set_stop_bits(settings.stop_bits);
        device_settings.set_flow_control(settings.flow_control);

        T::write_settings(self, &device_settings)
    }

    fn reconfigure(&mut self, setup: &Fn (&mut SerialPortSettings) -> ::Result<()>) -> ::Result<()> {
        let mut device_settings = try!(T::read_settings(self));
        try!(setup(&mut device_settings));
        T::write_settings(self, &device_settings)
    }

    fn set_rts(&mut self, level: bool) -> ::Result<()> {
        T::set_rts(self, level)
    }

    fn set_dtr(&mut self, level: bool) -> ::Result<()> {
        T::set_dtr(self, level)
    }

    fn read_cts(&mut self) -> ::Result<bool> {
        T::read_cts(self)
    }

    fn read_dsr(&mut self) -> ::Result<bool> {
        T::read_dsr(self)
    }

    fn read_ri(&mut self) -> ::Result<bool> {
        T::read_ri(self)
    }

    fn read_cd(&mut self) -> ::Result<bool> {
        T::read_cd(self)
    }
}

/// A trait for objects that implement serial port configurations.
pub trait SerialPortSettings {
    /// Returns the current baud rate.
    ///
    /// This function returns `None` if the baud rate could not be determined. This may occur if
    /// the hardware is in an uninitialized state. Setting a baud rate with `set_baud_rate()`
    /// should initialize the baud rate to a supported value.
    fn baud_rate(&self) -> Option<BaudRate>;

    /// Returns the character size.
    ///
    /// This function returns `None` if the character size could not be determined. This may occur
    /// if the hardware is in an uninitialized state or is using a non-standard character size.
    /// Setting a baud rate with `set_char_size()` should initialize the character size to a
    /// supported value.
    fn char_size(&self) -> Option<CharSize>;

    /// Returns the parity-checking mode.
    ///
    /// This function returns `None` if the parity mode could not be determined. This may occur if
    /// the hardware is in an uninitialized state or is using a non-standard parity mode. Setting
    /// a parity mode with `set_parity()` should initialize the parity mode to a supported value.
    fn parity(&self) -> Option<Parity>;

    /// Returns the number of stop bits.
    ///
    /// This function returns `None` if the number of stop bits could not be determined. This may
    /// occur if the hardware is in an uninitialized state or is using an unsupported stop bit
    /// configuration. Setting the number of stop bits with `set_stop-bits()` should initialize the
    /// stop bits to a supported value.
    fn stop_bits(&self) -> Option<StopBits>;

    /// Returns the flow control mode.
    ///
    /// This function returns `None` if the flow control mode could not be determined. This may
    /// occur if the hardware is in an uninitialized state or is using an unsupported flow control
    /// mode. Setting a flow control mode with `set_flow_control()` should initialize the flow
    /// control mode to a supported value.
    fn flow_control(&self) -> Option<FlowControl>;

    /// Sets the baud rate.
    ///
    /// ## Errors
    ///
    /// If the implementation does not support the requested baud rate, this function may return an
    /// `InvalidInput` error. Even if the baud rate is accepted by `set_baud_rate()`, it may not be
    /// supported by the underlying hardware.
    fn set_baud_rate(&mut self, baud_rate: BaudRate) -> ::Result<()>;

    /// Sets the character size.
    fn set_char_size(&mut self, char_size: CharSize);

    /// Sets the parity-checking mode.
    fn set_parity(&mut self, parity: Parity);

    /// Sets the number of stop bits.
    fn set_stop_bits(&mut self, stop_bits: StopBits);

    /// Sets the flow control mode.
    fn set_flow_control(&mut self, flow_control: FlowControl);
}

/// A device-indepenent implementation of serial port settings.
#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub struct PortSettings {
    /// Baud rate.
    pub baud_rate: BaudRate,

    /// Character size.
    pub char_size: CharSize,

    /// Parity checking mode.
    pub parity: Parity,

    /// Number of stop bits.
    pub stop_bits: StopBits,

    /// Flow control mode.
    pub flow_control: FlowControl
}

impl Default for PortSettings {
    fn default() -> Self {
        PortSettings {
            baud_rate: BaudRate::Baud9600,
            char_size: CharSize::Bits8,
            parity: Parity::ParityNone,
            stop_bits: StopBits::Stop1,
            flow_control: FlowControl::FlowNone
        }
    }
}

impl SerialPortSettings for PortSettings {
    fn baud_rate(&self) -> Option<BaudRate> {
        Some(self.baud_rate)
    }

    fn char_size(&self) -> Option<CharSize> {
        Some(self.char_size)
    }

    fn parity(&self) -> Option<Parity> {
        Some(self.parity)
    }

    fn stop_bits(&self) -> Option<StopBits> {
        Some(self.stop_bits)
    }

    fn flow_control(&self) -> Option<FlowControl> {
        Some(self.flow_control)
    }

    fn set_baud_rate(&mut self, baud_rate: BaudRate) -> ::Result<()> {
        self.baud_rate = baud_rate;
        Ok(())
    }

    fn set_char_size(&mut self, char_size: CharSize) {
        self.char_size = char_size;
    }

    fn set_parity(&mut self, parity: Parity) {
        self.parity = parity;
    }

    fn set_stop_bits(&mut self, stop_bits: StopBits) {
        self.stop_bits = stop_bits;
    }

    fn set_flow_control(&mut self, flow_control: FlowControl) {
        self.flow_control = flow_control;
    }
}

#[cfg(test)]
mod tests {
    use std::default::Default;
    use super::*;

    #[test]
    fn port_settings_manipulates_baud_rate() {
        let mut settings: PortSettings = Default::default();
        settings.set_baud_rate(Baud115200).unwrap();
        assert_eq!(settings.baud_rate(), Some(Baud115200));
    }

    #[test]
    fn port_settings_manipulates_char_size() {
        let mut settings: PortSettings = Default::default();
        settings.set_char_size(Bits7);
        assert_eq!(settings.char_size(), Some(Bits7));
    }

    #[test]
    fn port_settings_manipulates_parity() {
        let mut settings: PortSettings = Default::default();
        settings.set_parity(ParityEven);
        assert_eq!(settings.parity(), Some(ParityEven));
    }

    #[test]
    fn port_settings_manipulates_stop_bits() {
        let mut settings: PortSettings = Default::default();
        settings.set_stop_bits(Stop2);
        assert_eq!(settings.stop_bits(), Some(Stop2));
    }

    #[test]
    fn port_settings_manipulates_flow_control() {
        let mut settings: PortSettings = Default::default();
        settings.set_flow_control(FlowSoftware);
        assert_eq!(settings.flow_control(), Some(FlowSoftware));
    }
}
