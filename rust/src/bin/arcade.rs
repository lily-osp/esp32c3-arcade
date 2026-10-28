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
    breakout::BreakoutGame,
    mpu6050::Mpu6050,
    pong::PongGame,
    rng::Rng,
    shooter::ShooterGame,
    snake::SnakeGame,
    tilt_racer::TiltRacerGame,
    ui::{body_style, draw_centered, title_style},
};

esp_bootloader_esp_idf::esp_app_desc!();

#[derive(Clone, Copy, PartialEq, Eq)]
enum GameId {
    Shooter,
    Pong,
    Snake,
    TiltRacer,
    Breakout,
}

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

    // Welcome splash
    display.clear(BinaryColor::Off).ok();
    draw_centered(&mut display, "ARCADE", 26, title_style()).ok();
    draw_centered(&mut display, "5 GAMES STACK", 48, body_style()).ok();
    display.flush().unwrap();
    delay.delay_millis(700);

    let (mut prev_ax, mut prev_ay, mut prev_az) = mpu.read_accel().unwrap_or((0.0, 0.0, 9.8));
    let seed = ((prev_ax * 1000.0) as i32 as u32)
        ^ (((prev_ay * 1000.0) as i32 as u32) << 8)
        ^ (((prev_az * 1000.0) as i32 as u32) << 16)
        ^ get_millis() as u32;
    let mut rng = Rng::new(seed);

    let mut shooter = ShooterGame::new();
    let mut pong = PongGame::new();
    let mut snake = SnakeGame::new();
    let mut racer = TiltRacerGame::new();
    let mut breakout = BreakoutGame::new();

    let mut previous_game: Option<GameId> = None;
    let mut waiting_for_shake = true;
    let mut shake_ready_at = get_millis() + 1000;
    let mut last_frame = get_millis();

    let pick_next_game = |rng: &mut Rng, prev: Option<GameId>| -> GameId {
        let games = [
            GameId::Shooter,
            GameId::Pong,
            GameId::Snake,
            GameId::TiltRacer,
            GameId::Breakout,
        ];
        let mut pick = games[rng.gen_range(0, 5) as usize];
        while Some(pick) == prev {
            pick = games[rng.gen_range(0, 5) as usize];
        }
        pick
    };

    let mut current_game = pick_next_game(&mut rng, previous_game);

    loop {
        let now = get_millis();
        let (ax, ay, az) = mpu.read_accel().unwrap_or((prev_ax, prev_ay, prev_az));

        if waiting_for_shake {
            display.clear(BinaryColor::Off).ok();
            let game_title = match current_game {
                GameId::Shooter => "SHOOTER",
                GameId::Pong => "PONG",
                GameId::Snake => "SNAKE",
                GameId::TiltRacer => "TILT RACER",
                GameId::Breakout => "BREAKOUT",
            };
            draw_centered(&mut display, game_title, 25, title_style()).ok();
            draw_centered(&mut display, "SHAKE TO START", 48, body_style()).ok();
            display.flush().unwrap();

            let shake = (ax - prev_ax).abs() + (ay - prev_ay).abs() + (az - prev_az).abs();
            prev_ax = ax;
            prev_ay = ay;
            prev_az = az;

            let threshold = match current_game {
                GameId::Snake | GameId::TiltRacer | GameId::Breakout => 5.0,
                _ => 8.0,
            };

            if now >= shake_ready_at && shake > threshold {
                waiting_for_shake = false;

                match current_game {
                    GameId::Shooter => shooter.reset(&mut rng, now),
                    GameId::Pong => pong.reset_match(&mut rng),
                    GameId::Snake => snake.reset(&mut rng, now),
                    GameId::TiltRacer => racer.reset(now),
                    GameId::Breakout => breakout.reset_game(&mut rng),
                }

                // 3..2..1 Countdown (except Snake starts directly)
                if current_game != GameId::Snake {
                    for num in (1..=3).rev() {
                        display.clear(BinaryColor::Off).ok();
                        let mut text: String<4> = String::new();
                        write!(text, "{}", num).ok();
                        draw_centered(&mut display, &text, 36, title_style()).ok();
                        display.flush().unwrap();
                        delay.delay_millis(1000);
                    }

                    display.clear(BinaryColor::Off).ok();
                    draw_centered(&mut display, "GO", 36, title_style()).ok();
                    display.flush().unwrap();
                    delay.delay_millis(300);
                } else {
                    delay.delay_millis(180);
                }

                last_frame = get_millis();
            } else {
                delay.delay_millis(25);
                continue;
            }
        }

        // Active game frame execution
        let elapsed = now.saturating_sub(last_frame);
        let target_frame_ms = match current_game {
            GameId::Pong => 18,
            GameId::Snake => 4,
            _ => 20,
        };

        if elapsed < target_frame_ms {
            delay.delay_millis(2);
            continue;
        }
        last_frame = now;

        let mut round_over = false;

        match current_game {
            GameId::Shooter => {
                shooter.update(&mut rng, ax, now);
                if shooter.lives <= 0 || shooter.score <= -1 {
                    display.clear(BinaryColor::Off).ok();
                    draw_centered(&mut display, "GAME OVER", 25, title_style()).ok();
                    let mut score_str: String<24> = String::new();
                    write!(score_str, "SCORE {}", shooter.score).ok();
                    draw_centered(&mut display, &score_str, 48, body_style()).ok();
                    display.flush().unwrap();
                    delay.delay_millis(1400);
                    round_over = true;
                } else {
                    display.clear(BinaryColor::Off).ok();
                    shooter.draw(&mut display).ok();
                    display.flush().unwrap();
                }
            }
            GameId::Pong => {
                if let Some(player_won) = pong.update(&mut rng, ax, now) {
                    display.clear(BinaryColor::Off).ok();
                    let winner_msg = if player_won { "YOU WIN" } else { "AI WINS" };
                    draw_centered(&mut display, winner_msg, 25, title_style()).ok();
                    let mut final_score: String<16> = String::new();
                    write!(final_score, "{}:{}", pong.player_score, pong.ai_score).ok();
                    draw_centered(&mut display, &final_score, 48, body_style()).ok();
                    display.flush().unwrap();
                    delay.delay_millis(1800);
                    round_over = true;
                } else {
                    display.clear(BinaryColor::Off).ok();
                    pong.draw(&mut display).ok();
                    display.flush().unwrap();
                }
            }
            GameId::Snake => {
                let hit_self = snake.update(&mut rng, ax, ay, now);
                if hit_self {
                    display.clear(BinaryColor::Off).ok();
                    draw_centered(&mut display, "GAME OVER", 25, title_style()).ok();
                    let mut score_str: String<20> = String::new();
                    write!(score_str, "SCORE {}", snake.score).ok();
                    draw_centered(&mut display, &score_str, 48, body_style()).ok();
                    display.flush().unwrap();
                    delay.delay_millis(1200);
                    round_over = true;
                } else {
                    display.clear(BinaryColor::Off).ok();
                    snake.draw(&mut display).ok();
                    display.flush().unwrap();
                }
            }
            GameId::TiltRacer => {
                racer.update(&mut rng, ay, now);
                if racer.lives <= 0 {
                    display.clear(BinaryColor::Off).ok();
                    draw_centered(&mut display, "GAME OVER", 25, title_style()).ok();
                    let mut score_str: String<20> = String::new();
                    write!(score_str, "SCORE {}", racer.score).ok();
                    draw_centered(&mut display, &score_str, 48, body_style()).ok();
                    display.flush().unwrap();
                    delay.delay_millis(1400);
                    round_over = true;
                } else {
                    display.clear(BinaryColor::Off).ok();
                    racer.draw(&mut display, now).ok();
                    display.flush().unwrap();
                }
            }
            GameId::Breakout => {
                if let Some(status) = breakout.update(&mut rng, ay, now) {
                    if !status {
                        display.clear(BinaryColor::Off).ok();
                        draw_centered(&mut display, "GAME OVER", 25, title_style()).ok();
                        let mut score_str: String<20> = String::new();
                        write!(score_str, "SCORE {}", breakout.score).ok();
                        draw_centered(&mut display, &score_str, 48, body_style()).ok();
                        display.flush().unwrap();
                        delay.delay_millis(1500);
                        round_over = true;
                    }
                } else {
                    display.clear(BinaryColor::Off).ok();
                    breakout.draw(&mut display).ok();
                    display.flush().unwrap();
                }
            }
        }

        if round_over {
            previous_game = Some(current_game);
            current_game = pick_next_game(&mut rng, previous_game);

            let (fresh_ax, fresh_ay, fresh_az) = mpu.read_accel().unwrap_or((0.0, 0.0, 9.8));
            prev_ax = fresh_ax;
            prev_ay = fresh_ay;
            prev_az = fresh_az;
            shake_ready_at = get_millis() + 1000;
            waiting_for_shake = true;
        }
    }
}
