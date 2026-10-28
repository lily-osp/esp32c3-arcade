use micromath::F32Ext;

use core::fmt::Write;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::{Alignment, Text},
    Drawable, Pixel,
};
use heapless::String;

use crate::rng::Rng;

pub const W: i32 = 128;
pub const H: i32 = 64;
pub const PADDLE_H: i32 = 14;
pub const PADDLE_W: i32 = 3;
pub const BALL_SIZE: i32 = 3;
pub const TOP_MARGIN: f32 = 0.0;
pub const BOTTOM_MARGIN: f32 = (H - PADDLE_H) as f32;
pub const MAX_BALL_SPEED: f32 = 4.2;
pub const WIN_SCORE: i32 = 10;

pub struct PongGame {
    pub ball_x: f32,
    pub ball_y: f32,
    pub ball_vx: f32,
    pub ball_vy: f32,
    pub player_y: f32,
    pub ai_y: f32,
    pub player_score: i32,
    pub ai_score: i32,
    pub ai_mode: u8,
    pub ai_error: f32,
    pub point_delay_until: u64,
}

impl PongGame {
    pub fn new() -> Self {
        Self {
            ball_x: (W / 2) as f32,
            ball_y: (H / 2) as f32,
            ball_vx: 1.8,
            ball_vy: 0.7,
            player_y: 25.0,
            ai_y: 25.0,
            player_score: 0,
            ai_score: 0,
            ai_mode: 1,
            ai_error: 0.0,
            point_delay_until: 0,
        }
    }

    pub fn reset_match(&mut self, rng: &mut Rng) {
        self.player_score = 0;
        self.ai_score = 0;
        self.player_y = 25.0;
        self.ai_y = 25.0;
        self.point_delay_until = 0;

        let roll = rng.gen_range(0, 10);
        if roll < 4 {
            self.ai_mode = 0;
            self.ai_error = rng.gen_range(-12, 13) as f32;
        } else if roll < 7 {
            self.ai_mode = 1;
            self.ai_error = rng.gen_range(-5, 6) as f32;
        } else if roll < 9 {
            self.ai_mode = 2;
            self.ai_error = 0.0;
        } else {
            self.ai_mode = 3;
            self.ai_error = 0.0;
        }

        let dir = if rng.gen_range(0, 2) == 1 { 1.0 } else { -1.0 };
        self.serve_ball(dir, rng);
    }

    pub fn serve_ball(&mut self, direction: f32, rng: &mut Rng) {
        self.ball_x = (W / 2) as f32;
        self.ball_y = (H / 2) as f32;
        self.ball_vx = direction * 1.8;
        self.ball_vy = rng.gen_range(-8, 9) as f32 / 10.0;
    }

    pub fn update_player(&mut self, ax: f32) {
        // Positive ax tilts screen up
        self.player_y += ax * -0.85;
        self.player_y = self.player_y.clamp(TOP_MARGIN, BOTTOM_MARGIN);
    }

    pub fn update_ai(&mut self) {
        let target = self.ball_y - (PADDLE_H as f32 / 2.0) + self.ai_error;
        let diff = target - self.ai_y;

        let (response, max_step) = match self.ai_mode {
            0 => (0.045, 0.75),
            1 => (0.085, 1.5),
            2 => (0.18, 3.0),
            _ => (0.32, 4.5),
        };

        self.ai_y += (diff * response).clamp(-max_step, max_step);
        self.ai_y = self.ai_y.clamp(TOP_MARGIN, BOTTOM_MARGIN);
    }

    fn bounce_from_paddle(&mut self, paddle_y: f32, moving_right: bool) {
        let paddle_center = paddle_y + (PADDLE_H as f32 / 2.0);
        let hit_offset = (self.ball_y - paddle_center) / (PADDLE_H as f32 / 2.0);

        self.ball_vx = if moving_right {
            self.ball_vx.abs()
        } else {
            -self.ball_vx.abs()
        };
        self.ball_vx *= 1.04;
        self.ball_vy += hit_offset * 0.65;

        let speed = (self.ball_vx * self.ball_vx + self.ball_vy * self.ball_vy).sqrt();
        if speed > MAX_BALL_SPEED {
            self.ball_vx = (self.ball_vx / speed) * MAX_BALL_SPEED;
            self.ball_vy = (self.ball_vy / speed) * MAX_BALL_SPEED;
        }
    }

    pub fn update(&mut self, rng: &mut Rng, ax: f32, now_ms: u64) -> Option<bool> {
        if now_ms < self.point_delay_until {
            return None;
        }

        self.update_player(ax);
        self.update_ai();

        self.ball_x += self.ball_vx;
        self.ball_y += self.ball_vy;

        // Top/bottom walls
        if self.ball_y <= TOP_MARGIN {
            self.ball_y = TOP_MARGIN;
            self.ball_vy = self.ball_vy.abs();
        }
        if self.ball_y >= (H - BALL_SIZE) as f32 {
            self.ball_y = (H - BALL_SIZE) as f32;
            self.ball_vy = -self.ball_vy.abs();
        }

        // Paddle collisions
        // Player paddle at x=2..5
        if self.ball_vx < 0.0
            && self.ball_x <= 5.0
            && self.ball_x >= 2.0
            && (self.ball_y + BALL_SIZE as f32) >= self.player_y
            && self.ball_y <= (self.player_y + PADDLE_H as f32)
        {
            self.ball_x = 5.0;
            self.bounce_from_paddle(self.player_y, true);
        }

        // AI paddle at x=W-5..W-2
        if self.ball_vx > 0.0
            && (self.ball_x + BALL_SIZE as f32) >= (W - 5) as f32
            && self.ball_x <= (W - 2) as f32
            && (self.ball_y + BALL_SIZE as f32) >= self.ai_y
            && self.ball_y <= (self.ai_y + PADDLE_H as f32)
        {
            self.ball_x = (W - 8) as f32;
            self.bounce_from_paddle(self.ai_y, false);
        }

        // Left out: AI scores
        if self.ball_x < -(BALL_SIZE as f32) {
            self.ai_score += 1;
            if self.player_score >= WIN_SCORE || self.ai_score >= WIN_SCORE {
                return Some(self.player_score >= WIN_SCORE);
            }
            self.serve_ball(1.0, rng);
            self.point_delay_until = now_ms + 350;
        }

        // Right out: Player scores
        if self.ball_x > W as f32 {
            self.player_score += 1;
            if self.player_score >= WIN_SCORE || self.ai_score >= WIN_SCORE {
                return Some(self.player_score >= WIN_SCORE);
            }
            self.serve_ball(-1.0, rng);
            self.point_delay_until = now_ms + 350;
        }

        None
    }

    pub fn draw<D: DrawTarget<Color = BinaryColor>>(&self, target: &mut D) -> Result<(), D::Error> {
        // Dotted net in the center
        for y in (0..H).step_by(5) {
            Pixel(Point::new(W / 2, y), BinaryColor::On).draw(target)?;
        }

        // Paddles
        Rectangle::new(Point::new(2, self.player_y as i32), Size::new(PADDLE_W as u32, PADDLE_H as u32))
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
            .draw(target)?;

        Rectangle::new(Point::new(W - 5, self.ai_y as i32), Size::new(PADDLE_W as u32, PADDLE_H as u32))
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
            .draw(target)?;

        // Ball
        Rectangle::new(Point::new(self.ball_x as i32, self.ball_y as i32), Size::new(BALL_SIZE as u32, BALL_SIZE as u32))
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
            .draw(target)?;

        // Score overlay
        let mut score_str: String<16> = String::new();
        write!(score_str, "{}:{}", self.player_score, self.ai_score).ok();
        let hud_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();
        Text::with_alignment(&score_str, Point::new(W / 2, 7), hud_style, Alignment::Center)
            .draw(target)?;

        Ok(())
    }
}
