pub const MPU6050_ADDR: u8 = 0x68;

pub struct Mpu6050<I2C> {
    i2c: I2C,
}

impl<I2C: embedded_hal::i2c::I2c> Mpu6050<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    pub fn init(&mut self) -> Result<(), I2C::Error> {
        // Wake up device (clear SLEEP bit in PWR_MGMT_1)
        self.i2c.write(MPU6050_ADDR, &[0x6B, 0x00])?;
        // Set DLPF to ~42 Hz (CONFIG register)
        self.i2c.write(MPU6050_ADDR, &[0x1A, 0x03])?;
        // Set accelerometer range to +/- 8g (ACCEL_CONFIG register)
        self.i2c.write(MPU6050_ADDR, &[0x1C, 0x10])?;
        Ok(())
    }

    pub fn read_accel(&mut self) -> Result<(f32, f32, f32), I2C::Error> {
        let mut buf = [0u8; 6];
        self.i2c.write_read(MPU6050_ADDR, &[0x3B], &mut buf)?;
        let raw_x = i16::from_be_bytes([buf[0], buf[1]]);
        let raw_y = i16::from_be_bytes([buf[2], buf[3]]);
        let raw_z = i16::from_be_bytes([buf[4], buf[5]]);

        let ax = (raw_x as f32 / 4096.0) * 9.80665;
        let ay = (raw_y as f32 / 4096.0) * 9.80665;
        let az = (raw_z as f32 / 4096.0) * 9.80665;
        Ok((ax, ay, az))
    }
}
