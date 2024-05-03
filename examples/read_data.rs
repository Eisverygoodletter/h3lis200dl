use h3lis200dl::reg::ADDR_SECONDARY;
use h3lis200dl::H3LIS200DL;
use linux_embedded_hal::Delay;
use linux_embedded_hal::I2cdev;
fn main() {
    let i2c = I2cdev::new("/dev/i2c-1").unwrap();
    let mut sens = H3LIS200DL::new(i2c, ADDR_SECONDARY).unwrap();
    println!("{:?}", sens.get_accel());
}