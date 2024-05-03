use accelerometer::Accelerometer;
use embedded_hal::i2c::{self, I2c, SevenBitAddress};
use mint::Vector3;
use reg::{Register, I2C_SUB_MULTI};
use cast::{u16, f32};

pub mod reg;

#[derive(Debug)]
pub struct H3LIS200DL<T: I2c> {
    i2c: T,
    addr: SevenBitAddress,
}

#[derive(Debug)]
pub enum H3LIS200DLError<T: i2c::Error> {
    WrongChipId(u8),
    I2CError(T),
}
impl<T: i2c::Error> From<T> for H3LIS200DLError<T> {
    fn from(value: T) -> Self {
        Self::I2CError(value)
    }
}
impl<T: I2c> H3LIS200DL<T> {
    pub fn new(i2c: T, addr: SevenBitAddress) -> Result<Self, H3LIS200DLError<T::Error>> {
        let mut sens = Self { i2c, addr };
        let detected_chip_id = sens.get_device_id()?;
        if detected_chip_id != reg::DEVICE_ID {
            return Err(H3LIS200DLError::WrongChipId(detected_chip_id));
        }
        Ok(sens)
    }
    fn read_reg(&mut self, reg: Register) -> Result<u8, H3LIS200DLError<T::Error>> {
        let mut buf = [0u8];
        self.i2c.write_read(self.addr, &[reg.addr()], &mut buf)?;
        Ok(buf[0])
    }
    fn read_regs(&mut self, reg: Register, buffer: &mut [u8]) -> Result<(), H3LIS200DLError<T::Error>> {
        Ok(self.i2c
            .write_read(self.addr, &[reg.addr() | I2C_SUB_MULTI], buffer)?)
    }
    fn write_reg(&mut self, reg: Register, value: u8) -> Result<(), H3LIS200DLError<T::Error>> {
        self.i2c.write(self.addr, &[reg.addr(), value])?;
        Ok(())
    }
    fn get_device_id(&mut self) -> Result<u8, H3LIS200DLError<T::Error>> {
        self.read_reg(Register::WHO_AM_I)
    }
    pub fn get_accel(&mut self) -> Result<Vector3<f32>, H3LIS200DLError<T::Error>> {
        let mut buf = [0u8; 6];
        self.read_regs(Register::OUT_X, &mut buf)?;
        let x_i = (u16(buf[0]) + (u16(buf[1]) << 8)) as i16;
        let y_i = (u16(buf[2]) + (u16(buf[3]) << 8)) as i16;
        let z_i = (u16(buf[4]) + (u16(buf[5]) << 8)) as i16;
        fn convert_to_f32(v: i16) -> f32 {
            f32(v >> 4) * (0.78)
        }
        Ok(Vector3 {
            x: convert_to_f32(x_i),
            y: convert_to_f32(y_i),
            z: convert_to_f32(z_i),
        })
    }
}
