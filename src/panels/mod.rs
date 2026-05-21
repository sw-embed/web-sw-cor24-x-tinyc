//! UI panels rendered inside the App layout.
//!
//! Each peripheral has its own component module so the run loop in `main.rs`
//! stays small and so additional peripherals (I2C device panels in a later
//! saga step) can be added by dropping a new file in here.

pub mod i2c;
pub mod led;
pub mod listing;
pub mod registers;
pub mod rtc;
pub mod switch;
pub mod uart;

pub use i2c::I2cPanel;
pub use led::LedPanel;
pub use registers::RegistersPanel;
pub use rtc::RtcPanel;
pub use switch::SwitchPanel;
pub use uart::UartPanel;
