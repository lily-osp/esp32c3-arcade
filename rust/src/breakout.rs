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
pub const BASE_PADDLE_W: i32 = 18;
pub const BALL_SIZE: i32 = 3;
pub const BRICK_ROWS: usize = 4;
pub const BRICK_COLS: usize = 8;
pub const BRICK_W: i32 = 13;
pub const BRICK_H: i32 = 4;
pub const START_LIVES: i8 = 3;

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
    pub ball_x: f32,
    pub ball_y: f32,
    pub ball_vx: f32,
    pub ball_vy: f32,
    pub score: i32,
    pub lives: i8,
    pub level: u8,
    pub bricks: [[Brick; BRICK_COLS]; BRICK_ROWS],
    pub powerups: [Powerup; 3],
    pub wide_paddle_until_ms: u64,
    pub ball_stuck: bool,
}

impl BreakoutGame {
    pub fn new() -> Self {
        Self {
            paddle_x: (W / 2 - BASE_PADDLE_W / 2) as f32,
            paddle_w: BASE_PADDLE_W,
            ball_x: (W / 2) as f32,
            ball_y: (PADDLE_Y - BALL_SIZE - 1) as f32,
            ball_vx: 1.5,
            ball_vy: -1.8,
            score: 0,
            lives: START_LIVES,
            level: 1,
            bricks: [[Brick::default(); BRICK_COLS]; BRICK_ROWS],
            powerups: [Powerup::default(); 3],
            wide_paddle_until_ms: 0,
            ball_stuck: true,
        }
    }

    pub fn reset_level(&mut self, rng: &mut Rng) {
        for row in 0..BRICK_ROWS {
            let hp = match row {
                0 => 2, // Top row: reinforced 2-hit bricks
                _ => 1, // Standard 1-hit bricks
            };
            for col in 0..BRICK_COLS {
                let has_powerup = rng.gen_percent(20);
                self.bricks[row][col] = Brick { hp, has_powerup };
            }
        }
        self.reset_ball();
    }

    pub fn reset_game(&mut self, rng: &mut Rng) {
        self.score = 0;
        self.lives = START_LIVES;
        self.level = 1;
        self.wide_paddle_until_ms = 0;
        self.paddle_w = BASE_PADDLE_W;
        for p in &mut self.powerups {
            p.active = false;
        }
        self.reset_level(rng);
    }

    pub fn reset_ball(&mut self) {
        self.ball_stuck = true;
        self.ball_x = self.paddle_x + (self.paddle_w as f32 / 2.0);
        self.ball_y = (PADDLE_Y - BALL_SIZE - 1) as f32;
        let speed_mult = 1.0 + (self.level as f32 * 0.15);
        self.ball_vx = 1.5 * speed_mult;
        self.ball_vy = -1.8 * speed_mult;
    }

    pub fn update_paddle(&mut self, ay: f32) {
        // MPU Y acceleration steers paddle horizontally
        self.paddle_x += ay * 0.95;
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

    // Returns: Some(true) if cleared/win, Some(false) if game over, None if ongoing
    pub fn update(&mut self, rng: &mut Rng, ay: f32, now_ms: u64) -> Option<bool> {
        self.update_paddle(ay);

        // Power-up expiration
        if now_ms >= self.wide_paddle_until_ms {
            self.paddle_w = BASE_PADDLE_W;
        }

        // Launch ball if stuck and player tilts or after 1 sec
        if self.ball_stuck {
            if ay.abs() > 2.0 {
                self.ball_stuck = false;
                self.ball_vx = if ay > 0.0 { 1.6 } else { -1.6 };
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
            let hit_offset = (self.ball_x + BALL_SIZE as f32 / 2.0 - paddle_center)
                / (self.paddle_w as f32 / 2.0);

            let speed = (self.ball_vx * self.ball_vx + self.ball_vy * self.ball_vy).sqrt();
            let new_speed = (speed * 1.02).clamp(2.0, 4.5);

            self.ball_vx = hit_offset * (new_speed * 0.85);
            self.ball_vy = -((new_speed * new_speed - self.ball_vx * self.ball_vx)
                .max(1.0)
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
                            brick_x as f32 + BRICK_W as f32 / 2.0,
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
            p.y += 1.0;

            // Catch powerup
            if p.y >= PADDLE_Y as f32
                && p.y <= (PADDLE_Y + PADDLE_H + 4) as f32
                && p.x >= self.paddle_x - 2.0
                && p.x <= self.paddle_x + self.paddle_w as f32 + 2.0
            {
                p.active = false;
                match p.kind {
                    0 => {
                        // Wide paddle
                        self.paddle_w = BASE_PADDLE_W + 8;
                        self.wide_paddle_until_ms = now_ms + 8000;
                    }
                    1 => {
                        // Slow ball
                        self.ball_vx *= 0.75;
                        self.ball_vy *= 0.75;
                    }
                    _ => {
                        // Life +1
                        if self.lives < 5 {
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
                if brick.hp == 2 {
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

        // Powerups (dropping diamonds/capsules)
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

        // HUD: Score and Lives
        let mut hud_str: String<24> = String::new();
        write!(hud_str, "S:{} L:{} LV:{}", self.score, self.lives, self.level).ok();
        let hud_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();
        Text::new(&hud_str, Point::new(1, 7), hud_style).draw(target)?;

        Ok(())
    }
}
