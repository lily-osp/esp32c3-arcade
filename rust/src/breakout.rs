use core::fmt::Write;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle},
    text::Text,
    Drawable, Pixel,
};
use heapless::String;
use micromath::F32Ext;

use crate::rng::Rng;

pub const W: i32 = 128;
pub const H: i32 = 64;
pub const PADDLE_Y: i32 = 58;
pub const PADDLE_H: i32 = 4;
pub const BALL_SIZE: i32 = 3;
pub const BRICK_ROWS: usize = 4;
pub const BRICK_COLS: usize = 8;
pub const BRICK_W: i32 = 13;
pub const BRICK_H: i32 = 4;

#[derive(Clone, Copy, Default)]
pub struct Brick {
    pub hp: u8,
    pub has_powerup: bool,
}

#[derive(Clone, Copy, Default)]
pub struct Powerup {
    pub x: f32,
    pub y: f32,
    pub kind: u8, // 0 = Wide Paddle, 1 = Slow Ball, 2 = Life +1
    pub active: bool,
}

pub struct BreakoutGame {
    pub paddle_x: f32,
    pub paddle_w: i32,
    pub base_paddle_w: i32,
    pub ball_x: f32,
    pub ball_y: f32,
    pub ball_vx: f32,
    pub ball_vy: f32,
    pub score: i32,
    pub lives: i8,
    pub level: u8,
    pub difficulty: u8, // 0 = Easy, 1 = Medium, 2 = Hard, 3 = Extreme
    pub powerup_chance: u32,
    pub base_speed: f32,
    pub max_speed: f32,
    pub bricks: [[Brick; BRICK_COLS]; BRICK_ROWS],
    pub powerups: [Powerup; 3],
    pub wide_paddle_until_ms: u64,
    pub ball_stuck: bool,
}

impl BreakoutGame {
    pub fn new() -> Self {
        Self {
            paddle_x: 51.0,
            paddle_w: 26,
            base_paddle_w: 26,
            ball_x: 64.0,
            ball_y: 53.0,
            ball_vx: 1.1,
            ball_vy: -1.3,
            score: 0,
            lives: 5,
            level: 1,
            difficulty: 0,
            powerup_chance: 35,
            base_speed: 1.1,
            max_speed: 2.8,
            bricks: [[Brick::default(); BRICK_COLS]; BRICK_ROWS],
            powerups: [Powerup::default(); 3],
            wide_paddle_until_ms: 0,
            ball_stuck: true,
        }
    }

    pub fn difficulty_name(&self) -> &'static str {
        match self.difficulty {
            0 => "EASY",
            1 => "MEDIUM",
            2 => "HARD",
            _ => "EXTREME",
        }
    }

    pub fn reset_game(&mut self, rng: &mut Rng) {
        self.score = 0;
        self.level = 1;
        self.wide_paddle_until_ms = 0;
        for p in &mut self.powerups {
            p.active = false;
        }

        // Weighted random difficulty: Easy 40%, Medium 30%, Hard 20%, Extreme 10%
        let roll = rng.gen_range(0, 10);
        if roll < 4 {
            self.difficulty = 0; // Easy
            self.base_paddle_w = 26;
            self.powerup_chance = 35;
            self.lives = 5;
            self.base_speed = 1.1;
            self.max_speed = 2.8;
        } else if roll < 7 {
            self.difficulty = 1; // Medium
            self.base_paddle_w = 22;
            self.powerup_chance = 25;
            self.lives = 4;
            self.base_speed = 1.4;
            self.max_speed = 3.4;
        } else if roll < 9 {
            self.difficulty = 2; // Hard
            self.base_paddle_w = 18;
            self.powerup_chance = 18;
            self.lives = 3;
            self.base_speed = 1.7;
            self.max_speed = 4.0;
        } else {
            self.difficulty = 3; // Extreme
            self.base_paddle_w = 15;
            self.powerup_chance = 10;
            self.lives = 2;
            self.base_speed = 2.0;
            self.max_speed = 4.6;
        }

        self.paddle_w = self.base_paddle_w;
        self.paddle_x = ((W - self.paddle_w) / 2) as f32;
        self.reset_level(rng);
    }

    pub fn reset_level(&mut self, rng: &mut Rng) {
        for row in 0..BRICK_ROWS {
            let hp = match self.difficulty {
                0 => 1, // Easy: all 1-hit bricks
                1 => if row == 0 { 2 } else { 1 }, // Medium: top row 2 hits
                2 => if row <= 1 { 2 } else { 1 }, // Hard: top 2 rows 2 hits
                _ => if row == 0 { 3 } else if row == 1 { 2 } else { 1 }, // Extreme: 3 & 2 hits
            };
            for col in 0..BRICK_COLS {
                let has_powerup = rng.gen_percent(self.powerup_chance);
                self.bricks[row][col] = Brick { hp, has_powerup };
            }
        }
        self.reset_ball();
    }

    pub fn reset_ball(&mut self) {
        self.ball_stuck = true;
        self.ball_x = self.paddle_x + (self.paddle_w as f32 / 2.0);
        self.ball_y = (PADDLE_Y - BALL_SIZE - 1) as f32;
        let speed_mult = 1.0 + ((self.level - 1) as f32 * 0.10);
        let speed = (self.base_speed * speed_mult).min(self.max_speed);
        self.ball_vx = speed * 0.75;
        self.ball_vy = -speed;
    }

    pub fn update_paddle(&mut self, ay: f32) {
        // High responsive tilt steering
        self.paddle_x += ay * 1.15;
        let max_x = (W - self.paddle_w) as f32;
        self.paddle_x = self.paddle_x.clamp(0.0, max_x);

        if self.ball_stuck {
            self.ball_x = self.paddle_x + (self.paddle_w as f32 / 2.0);
            self.ball_y = (PADDLE_Y - BALL_SIZE - 1) as f32;
        }
    }

    fn spawn_powerup(&mut self, x: f32, y: f32, rng: &mut Rng) {
        for p in &mut self.powerups {
            if !p.active {
                p.active = true;
                p.x = x;
                p.y = y;
                p.kind = rng.gen_range(0, 3) as u8;
                return;
            }
        }
    }

    fn active_bricks_count(&self) -> usize {
        let mut count = 0;
        for row in 0..BRICK_ROWS {
            for col in 0..BRICK_COLS {
                if self.bricks[row][col].hp > 0 {
                    count += 1;
                }
            }
        }
        count
    }

    // Returns: Some(false) if game over, None if ongoing
    pub fn update(&mut self, rng: &mut Rng, ay: f32, now_ms: u64) -> Option<bool> {
        self.update_paddle(ay);

        // Power-up expiration
        if now_ms >= self.wide_paddle_until_ms {
            self.paddle_w = self.base_paddle_w;
        }

        // Release ball if stuck when tilted
        if self.ball_stuck {
            if ay.abs() > 1.8 {
                self.ball_stuck = false;
                let sign = if ay > 0.0 { 1.0 } else { -1.0 };
                self.ball_vx = sign * (self.base_speed * 0.85);
            }
            return None;
        }

        // Move ball
        self.ball_x += self.ball_vx;
        self.ball_y += self.ball_vy;

        // Bounce walls
        if self.ball_x <= 0.0 {
            self.ball_x = 0.0;
            self.ball_vx = self.ball_vx.abs();
        }
        if self.ball_x >= (W - BALL_SIZE) as f32 {
            self.ball_x = (W - BALL_SIZE) as f32;
            self.ball_vx = -self.ball_vx.abs();
        }
        if self.ball_y <= 8.0 {
            // Below HUD
            self.ball_y = 8.0;
            self.ball_vy = self.ball_vy.abs();
        }

        // Bounce Paddle
        if self.ball_vy > 0.0
            && (self.ball_y + BALL_SIZE as f32) >= PADDLE_Y as f32
            && (self.ball_y + BALL_SIZE as f32) <= (PADDLE_Y + PADDLE_H + 2) as f32
            && self.ball_x + BALL_SIZE as f32 >= self.paddle_x
            && self.ball_x <= self.paddle_x + self.paddle_w as f32
        {
            self.ball_y = (PADDLE_Y - BALL_SIZE) as f32;
            let paddle_center = self.paddle_x + (self.paddle_w as f32 / 2.0);
            let hit_offset = (self.ball_x + (BALL_SIZE as f32 / 2.0) - paddle_center)
                / (self.paddle_w as f32 / 2.0);

            let speed = (self.ball_vx * self.ball_vx + self.ball_vy * self.ball_vy).sqrt();
            let new_speed = (speed * 1.01).clamp(self.base_speed, self.max_speed);

            self.ball_vx = hit_offset * (new_speed * 0.85);
            self.ball_vy = -((new_speed * new_speed - self.ball_vx * self.ball_vx)
                .max(0.8)
                .sqrt());
        }

        // Brick collision
        for row in 0..BRICK_ROWS {
            let brick_y = 10 + (row as i32 * (BRICK_H + 2));
            for col in 0..BRICK_COLS {
                if self.bricks[row][col].hp == 0 {
                    continue;
                }
                let brick_x = 4 + (col as i32 * (BRICK_W + 2));

                let collides = (self.ball_x + BALL_SIZE as f32) >= brick_x as f32
                    && self.ball_x <= (brick_x + BRICK_W) as f32
                    && (self.ball_y + BALL_SIZE as f32) >= brick_y as f32
                    && self.ball_y <= (brick_y + BRICK_H) as f32;

                if collides {
                    self.bricks[row][col].hp -= 1;
                    self.score += 10;
                    self.ball_vy = -self.ball_vy;

                    if self.bricks[row][col].hp == 0 && self.bricks[row][col].has_powerup {
                        self.spawn_powerup(
                            brick_x as f32 + (BRICK_W as f32 / 2.0),
                            brick_y as f32 + BRICK_H as f32,
                            rng,
                        );
                    }
                    break;
                }
            }
        }

        // Powerups update
        for p in &mut self.powerups {
            if !p.active {
                continue;
            }
            p.y += 0.9;

            // Catch powerup
            if p.y >= PADDLE_Y as f32
                && p.y <= (PADDLE_Y + PADDLE_H + 4) as f32
                && p.x >= self.paddle_x - 2.0
                && p.x <= self.paddle_x + (self.paddle_w as f32) + 2.0
            {
                p.active = false;
                match p.kind {
                    0 => {
                        // Wide paddle
                        self.paddle_w = self.base_paddle_w + 10;
                        self.wide_paddle_until_ms = now_ms + 8000;
                    }
                    1 => {
                        // Slow ball
                        self.ball_vx *= 0.70;
                        self.ball_vy *= 0.70;
                    }
                    _ => {
                        // Life +1
                        if self.lives < 6 {
                            self.lives += 1;
                        }
                    }
                }
            } else if p.y > H as f32 {
                p.active = false;
            }
        }

        // Ball falls off bottom
        if self.ball_y > H as f32 {
            self.lives -= 1;
            if self.lives <= 0 {
                return Some(false); // Game Over
            }
            self.reset_ball();
        }

        // Check level cleared
        if self.active_bricks_count() == 0 {
            self.level += 1;
            self.score += 100;
            self.reset_level(rng);
        }

        None
    }

    pub fn draw<D: DrawTarget<Color = BinaryColor>>(&self, target: &mut D) -> Result<(), D::Error> {
        // Bricks
        for row in 0..BRICK_ROWS {
            let by = 10 + (row as i32 * (BRICK_H + 2));
            for col in 0..BRICK_COLS {
                let brick = self.bricks[row][col];
                if brick.hp == 0 {
                    continue;
                }
                let bx = 4 + (col as i32 * (BRICK_W + 2));
                if brick.hp >= 2 {
                    // Reinforced brick: filled with inner border
                    Rectangle::new(Point::new(bx, by), Size::new(BRICK_W as u32, BRICK_H as u32))
                        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                        .draw(target)?;
                    Pixel(Point::new(bx + 1, by + 1), BinaryColor::Off).draw(target)?;
                    Pixel(Point::new(bx + BRICK_W - 2, by + 1), BinaryColor::Off).draw(target)?;
                } else {
                    // Standard brick: hollow outline
                    Rectangle::new(Point::new(bx, by), Size::new(BRICK_W as u32, BRICK_H as u32))
                        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
                        .draw(target)?;
                }
            }
        }

        // Paddle
        Rectangle::new(
            Point::new(self.paddle_x as i32, PADDLE_Y),
            Size::new(self.paddle_w as u32, PADDLE_H as u32),
        )
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        .draw(target)?;

        // Ball
        Rectangle::new(
            Point::new(self.ball_x as i32, self.ball_y as i32),
            Size::new(BALL_SIZE as u32, BALL_SIZE as u32),
        )
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        .draw(target)?;

        // Powerups (dropping capsule shapes)
        for p in &self.powerups {
            if !p.active {
                continue;
            }
            let px = p.x as i32;
            let py = p.y as i32;
            Circle::new(Point::new(px - 2, py - 2), 5)
                .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                .draw(target)?;
            Pixel(Point::new(px, py), BinaryColor::Off).draw(target)?;
        }

        // HUD: Score, Lives, Difficulty Tag
        let mut hud_str: String<24> = String::new();
        let diff_tag = match self.difficulty {
            0 => "EZ",
            1 => "MD",
            2 => "HD",
            _ => "XT",
        };
        write!(hud_str, "S:{} L:{} {}", self.score, self.lives, diff_tag).ok();
        let hud_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();
        Text::new(&hud_str, Point::new(1, 7), hud_style).draw(target)?;

        Ok(())
    }
}
