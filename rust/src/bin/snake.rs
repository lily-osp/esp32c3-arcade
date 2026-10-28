#![no_std]
#![no_main]

use core::cell::RefCell;
use core::fmt::Write;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_hal_bus::i2c::RefCellDevice;
use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    i2c::master::{Config, I2c},
    time::{Instant, Rate},
};
use esp_println::println;
use heapless::String;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

use esp32c3_arcade::{
    mpu6050::Mpu6050,
    rng::Rng,
    snake::SnakeGame,
    ui::{body_style, draw_centered, title_style},
};

esp_bootloader_esp_idf::esp_app_desc!();

const SHAKE_THRESHOLD: f32 = 5.0;

#[esp_hal::main]
fn main() -> ! {
    let p = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    let i2c = I2c::new(p.I2C0, Config::default().with_frequency(Rate::from_khz(400)))
        .unwrap()
        .with_sda(p.GPIO9)
        .with_scl(p.GPIO8);

    let i2c_bus = RefCell::new(i2c);
    let oled_dev = RefCellDevice::new(&i2c_bus);
    let mpu_dev = RefCellDevice::new(&i2c_bus);

    let interface = I2CDisplayInterface::new(oled_dev);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();
    println!("SSD1306 initialized!");

    let mut mpu = Mpu6050::new(mpu_dev);
    mpu.init().unwrap();
    println!("MPU6050 initialized!");

    let start_instant = Instant::now();
    let get_millis = || (Instant::now() - start_instant).as_millis();

    display.clear(BinaryColor::Off).ok();
    draw_centered(&mut display, "SNAKE", 36, title_style()).ok();
    display.flush().unwrap();
    delay.delay_millis(500);

    let (mut prev_ax, mut prev_ay, mut prev_az) = mpu.read_accel().unwrap_or((0.0, 0.0, 9.8));
    let seed = ((prev_ax * 1000.0) as i32 as u32)
        ^ (((prev_ay * 1000.0) as i32 as u32) << 8)
        ^ (((prev_az * 1000.0) as i32 as u32) << 16)
        ^ get_millis() as u32;
    let mut rng = Rng::new(seed);

    let mut game = SnakeGame::new();
    let mut waiting_for_shake = true;
    let mut shake_ready_at = get_millis() + 1000;

    loop {
        let now = get_millis();
        let (ax, ay, az) = mpu.read_accel().unwrap_or((prev_ax, prev_ay, prev_az));

        if waiting_for_shake {
            display.clear(BinaryColor::Off).ok();
            draw_centered(&mut display, "SNAKE", 25, title_style()).ok();
            draw_centered(&mut display, "SHAKE TO START", 48, body_style()).ok();
            display.flush().unwrap();

            let shake = (ax - prev_ax).abs() + (ay - prev_ay).abs() + (az - prev_az).abs();
            prev_ax = ax;
            prev_ay = ay;
            prev_az = az;

            if now >= shake_ready_at && shake > SHAKE_THRESHOLD {
                waiting_for_shake = false;
                game.reset(&mut rng, now);
                delay.delay_millis(180);
            } else {
                delay.delay_millis(25);
                continue;
            }
        }

        let game_over = game.update(&mut rng, ax, ay, now);

        if game_over {
            display.clear(BinaryColor::Off).ok();
            draw_centered(&mut display, "GAME OVER", 25, title_style()).ok();
            let mut score_str: String<20> = String::new();
            write!(score_str, "SCORE {}", game.score).ok();
            draw_centered(&mut display, &score_str, 48, body_style()).ok();
            display.flush().unwrap();
            delay.delay_millis(1200);

            let (fresh_ax, fresh_ay, fresh_az) = mpu.read_accel().unwrap_or((0.0, 0.0, 9.8));
            prev_ax = fresh_ax;
            prev_ay = fresh_ay;
            prev_az = fresh_az;
            shake_ready_at = get_millis() + 1000;
            waiting_for_shake = true;
            continue;
        }

        display.clear(BinaryColor::Off).ok();
        game.draw(&mut display).ok();
        display.flush().unwrap();
        delay.delay_millis(4);
    }
}
