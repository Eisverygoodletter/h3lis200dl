use anyhow::Result;
use embedded_hal::i2c::SevenBitAddress;
pub const ADDR_PRIMARY: SevenBitAddress = 0b00011000;
pub const ADDR_SECONDARY: SevenBitAddress = 0b00011001;
pub(crate) const I2C_SUB_MULTI: u8 = 0b1000_0000;

mod maps {
    use bimap::BiMap;
    use lazy_static::lazy_static;

    use super::{DataRate, PowerMode};
    lazy_static! {
        pub static ref POWER_MODE_BITMAP: BiMap<u8, PowerMode> = {
            let mut b = BiMap::new();
            b.insert(0b0000_0000, PowerMode::PowerDown);
            b.insert(0b0010_0000, PowerMode::NormalMode);
            b.insert(0b0100_0000, PowerMode::HzHalf);
            b.insert(0b0110_0000, PowerMode::Hz1);
            b.insert(0b1000_0000, PowerMode::Hz2);
            b.insert(0b1010_0000, PowerMode::Hz5);
            b.insert(0b1100_0000, PowerMode::Hz10);
            b
        };
        pub static ref DATA_RATE_BITMAP: BiMap<u8, DataRate> = {
            let mut b = BiMap::new();
            b.insert(0, DataRate::Odr50);
            b.insert(0b0000_1000, DataRate::Odr100);
            b.insert(0b0001_0000, DataRate::Odr400);
            b.insert(0b0001_1000, DataRate::Odr1000);
            b
        };
    }
}
#[allow(clippy::upper_case_acronyms, non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum Register {
    WHO_AM_I = 0x0F,
    CTRL_REG1 = 0x20,
    CTRL_REG2 = 0x21,
    CTRL_REG3 = 0x22,
    CTRL_REG4 = 0x23,
    CTRL_REG5 = 0x24,
    HP_FILTER_RESET = 0x25,
    REFERENCE = 0x26,
    STATUS_REG = 0x27,
    OUT_X = 0x29,
    OUT_Y = 0x2B,
    OUT_Z = 0x2D,
    INT1_CFG = 0x30,
    INT1_SRC = 0x31,
    INT1_THS = 0x32,
    INT1_DURATION = 0x33,
    INT2_CFG = 0x34,
    INT2_SRC = 0x35,
    INT2_THS = 0x36,
    INT2_DURATION = 0x37,
}
impl Register {
    pub fn addr(&self) -> u8 {
        *self as u8
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PowerMode {
    PowerDown,
    NormalMode,
    HzHalf,
    Hz1,
    Hz2,
    Hz5,
    Hz10,
}

/// Data rate selection
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DataRate {
    Odr50 = 37,
    Odr100 = 74,
    Odr400 = 292,
    Odr1000 = 780,
}
impl DataRate {
    fn low_pass_filter_cutoff_frequency(&self) -> u32 {
        *self as u32
    }
}

/// WHO_AM_I identification
pub const DEVICE_ID: u8 = 0b0011_0010;

pub trait RegisterEncoding {
    fn from_u8(reading: u8) -> Result<Self>
    where
        Self: Sized;
    fn to_u8(&self) -> Result<u8>;
}

#[derive(Clone, Copy, Debug)]
pub struct CtrlReg1 {
    pub power_mode: PowerMode,
    pub data_rate: DataRate,
    pub x_axis_enabled: bool,
    pub y_axis_enabled: bool,
    pub z_axis_enabled: bool,
}

impl RegisterEncoding for CtrlReg1 {
    fn from_u8(reading: u8) -> Result<Self> {
        let power_mode = {
            let pm = 0b1110_0000 & reading;
            maps::POWER_MODE_BITMAP.get_by_left(&pm)
        }
        .expect("Invalid power mode");
        let data_rate = {
            let dr = 0b0001_1000 & reading;
            maps::DATA_RATE_BITMAP.get_by_left(&dr)
        }
        .expect("INTERNAL ERROR: all data rate cases were covered");
        let z = 0b0000_0100 & reading != 0;
        let y = 0b0000_0010 & reading != 0;
        let x = 0b0000_0001 & reading != 0;
        Ok(Self {
            power_mode: *power_mode,
            data_rate: *data_rate,
            x_axis_enabled: x,
            y_axis_enabled: y,
            z_axis_enabled: z,
        })
    }
    fn to_u8(&self) -> Result<u8> {
        let pm_bits = maps::POWER_MODE_BITMAP
            .get_by_right(&self.power_mode)
            .expect("All power modes covered");
        let dr_bits = maps::DATA_RATE_BITMAP
            .get_by_right(&self.data_rate)
            .expect("All data rates are covered");
        let z = if self.z_axis_enabled { 0b0000_0100 } else { 0 };
        let y = if self.y_axis_enabled { 0b0000_0010 } else { 0 };
        let x = if self.x_axis_enabled { 0b0000_0001 } else { 0 };
        Ok(pm_bits | dr_bits | z | y | x)
    }
}
