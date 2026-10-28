use core::fmt::Write;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle, Rectangle},
    text::Text,
    Drawable, Pixel,
};
use heapless::String;

use crate::rng::Rng;

pub const W: i32 = 128;
pub const H: i32 = 64;
pub const ROAD_LEFT: i32 = 9;
pub const ROAD_RIGHT: i32 = 118;
pub const PLAYER_Y: f32 = 53.0;
pub const CAR_W: i32 = 9;
pub const CAR_H: i32 = 7;
pub const MAX_OBSTACLES: usize = 6;
pub const START_LIVES: i8 = 3;
pub const INVINCIBILITY_MS: u64 = 1000;
pub const SPAWN_MIN_MS: u64 = 430;

#[derive(Clone, Copy, Default)]
pub struct Obstacle {
    pub x: f32,
    pub y: f32,
    pub active: bool,
}

pub struct TiltRacerGame {
    pub player_x: f32,
    pub player_velocity: f32,
    pub road_speed: f32,
    pub road_offset: f32,
    pub score: i32,
    pub lives: i8,
    pub last_spawn_ms: u64,
    pub last_score_ms: u64,
    pub invincible_until_ms: u64,
    pub obstacles: [Obstacle; MAX_OBSTACLES],
}

impl TiltRacerGame {
    pub fn new() -> Self {
        Self {
            player_x: (W / 2) as f32,
            player_velocity: 0.0,
            road_speed: 1.35,
            road_offset: 0.0,
            score: 0,
            lives: START_LIVES,
            last_spawn_ms: 0,
            last_score_ms: 0,
            invincible_until_ms: 0,
            obstacles: [Obstacle::default(); MAX_OBSTACLES],
        }
    }

    pub fn reset(&mut self, now_ms: u64) {
        self.player_x = (W / 2) as f32;
        self.player_velocity = 0.0;
        self.road_speed = 1.35;
        self.road_offset = 0.0;
        self.score = 0;
        self.lives = START_LIVES;
        self.invincible_until_ms = 0;
        self.last_spawn_ms = now_ms;
        self.last_score_ms = now_ms;

        for obs in &mut self.obstacles {
            obs.active = false;
        }
    }

    fn active_obstacle_count(&self) -> usize {
        self.obstacles.iter().filter(|o| o.active).count()
    }

    fn has_nearby_obstacle(&self, x: f32, y: f32, distance: f32) -> bool {
        for obs in &self.obstacles {
            if !obs.active {
                continue;
            }
            if (obs.x - x).abs() < (CAR_W + 4) as f32 && (obs.y - y).abs() < distance {
                return true;
            }
        }
        false
    }

    pub fn spawn_obstacle(&mut self, rng: &mut Rng, now_ms: u64) {
        let elapsed = now_ms.saturating_sub(self.last_spawn_ms);
        let mut spawn_interval = 900;
        if self.road_speed > 2.0 {
            spawn_interval = 720;
        }
        if self.road_speed > 2.8 {
            spawn_interval = 570;
        }
        if self.road_speed > 3.5 {
            spawn_interval = SPAWN_MIN_MS;
        }

        if elapsed < spawn_interval {
            return;
        }
        if self.active_obstacle_count() >= MAX_OBSTACLES - 1 {
            return;
        }

        let spawn_y = -(CAR_H as f32) - (rng.gen_range(0, 13) as f32);
        let min_x = (ROAD_LEFT + CAR_W / 2) as i32;
        let max_x = (ROAD_RIGHT - CAR_W / 2 + 1) as i32;
        let spawn_x = rng.gen_range(min_x, max_x) as f32;

        if self.has_nearby_obstacle(spawn_x, spawn_y, 22.0) {
            return;
        }

        for obs in &mut self.obstacles {
            if !obs.active {
                obs.active = true;
                obs.x = spawn_x;
                obs.y = spawn_y;
                self.last_spawn_ms = now_ms;
                return;
            }
        }
    }

    pub fn update_player(&mut self, ay: f32) {
        // MPU Y controls screen horizontal position
        self.player_velocity += ay * 0.10;
        self.player_velocity *= 0.86;
        self.player_velocity = self.player_velocity.clamp(-2.8, 2.8);

        self.player_x += self.player_velocity;
        let min_x = (ROAD_LEFT as f32) + (CAR_W as f32 / 2.0);
        let max_x = (ROAD_RIGHT as f32) - (CAR_W as f32 / 2.0);
        self.player_x = self.player_x.clamp(min_x, max_x);
    }

    fn overlaps(ax: f32, ay: f32, aw: f32, ah: f32, bx: f32, by: f32, bw: f32, bh: f32) -> bool {
        (ax - aw / 2.0) < (bx + bw / 2.0)
            && (ax + aw / 2.0) > (bx - bw / 2.0)
            && (ay - ah / 2.0) < (by + bh / 2.0)
            && (ay + ah / 2.0) > (by - bh / 2.0)
    }

    pub fn update(&mut self, rng: &mut Rng, ay: f32, now_ms: u64) {
        self.update_player(ay);

        // Road scroll
        self.road_offset += self.road_speed;
        if self.road_offset >= 12.0 {
            self.road_offset -= 12.0;
        }

        // Obstacles move downward
        for obs in &mut self.obstacles {
            if !obs.active {
                continue;
            }
            obs.y += self.road_speed;
            if obs.y > (H + CAR_H) as f32 {
                obs.active = false;
            }
        }

        self.spawn_obstacle(rng, now_ms);

        // Collisions
        if now_ms >= self.invincible_until_ms {
            for obs in &mut self.obstacles {
                if !obs.active {
                    continue;
                }
                if Self::overlaps(
                    self.player_x,
                    PLAYER_Y,
                    CAR_W as f32,
                    CAR_H as f32,
                    obs.x,
                    obs.y,
                    CAR_W as f32,
                    CAR_H as f32,
                ) {
                    obs.active = false;
                    self.lives -= 1;
                    self.invincible_until_ms = now_ms + INVINCIBILITY_MS;
                    self.player_velocity = 0.0;
                    if self.lives > 0 {
                        self.player_x = (W / 2) as f32;
                    }
                    break;
                }
            }
        }

        // Score tick every 500ms
        if now_ms.saturating_sub(self.last_score_ms) >= 500 {
            self.score += 1;
            self.last_score_ms = now_ms;
            if self.road_speed < 4.2 {
                self.road_speed += 0.035;
            }
        }
    }

    fn draw_car<D: DrawTarget<Color = BinaryColor>>(
        target: &mut D,
        center_x: f32,
        center_y: f32,
        is_player: bool,
    ) -> Result<(), D::Error> {
        let x = (center_x - CAR_W as f32 / 2.0) as i32;
        let y = (center_y - CAR_H as f32 / 2.0) as i32;

        // Car chassis & body
        Rectangle::new(Point::new(x + 1, y), Size::new((CAR_W - 2) as u32, CAR_H as u32))
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
            .draw(target)?;

        Rectangle::new(Point::new(x, y + 2), Size::new(CAR_W as u32, 3))
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
            .draw(target)?;

        // Wheels
        Pixel(Point::new(x + 2, y + 1), BinaryColor::Off).draw(target)?;
        Pixel(Point::new(x + CAR_W - 3, y + 1), BinaryColor::Off).draw(target)?;
        Pixel(Point::new(x + 1, y + CAR_H - 1), BinaryColor::Off).draw(target)?;
        Pixel(Point::new(x + CAR_W - 2, y + CAR_H - 1), BinaryColor::Off).draw(target)?;

        if is_player {
            Line::new(Point::new(x + 3, y + 3), Point::new(x + 5, y + 3))
                .into_styled(PrimitiveStyle::with_stroke(BinaryColor::Off, 1))
                .draw(target)?;
        } else {
            Pixel(Point::new(x + CAR_W / 2, y + 5), BinaryColor::Off).draw(target)?;
        }

        Ok(())
    }

    pub fn draw<D: DrawTarget<Color = BinaryColor>>(&self, target: &mut D, now_ms: u64) -> Result<(), D::Error> {
        // Road outer boundaries
        Line::new(Point::new(ROAD_LEFT, 0), Point::new(ROAD_LEFT, H))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(target)?;

        Line::new(Point::new(ROAD_RIGHT, 0), Point::new(ROAD_RIGHT, H))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(target)?;

        // Lane dividing markers
        for lane in 1..=2 {
            let x = ROAD_LEFT + lane * 36;
            let mut y = -12;
            while y < H {
                let marker_y = y + self.road_offset as i32;
                if marker_y + 6 >= 0 && marker_y < H {
                    Line::new(Point::new(x, marker_y), Point::new(x, marker_y + 6))
                        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
                        .draw(target)?;
                }
                y += 12;
            }
        }

        // Obstacles
        for obs in &self.obstacles {
            if obs.active {
                Self::draw_car(target, obs.x, obs.y, false)?;
            }
        }

        // Player with invincibility blink
        let blinking = now_ms < self.invincible_until_ms && ((now_ms / 100) % 2 == 0);
        if !blinking {
            Self::draw_car(target, self.player_x, PLAYER_Y, true)?;
        }

        // HUD
        let mut hud_str: String<24> = String::new();
        write!(hud_str, "S:{} L:{}", self.score, self.lives).ok();
        let hud_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();
        Text::new(&hud_str, Point::new(1, 8), hud_style).draw(target)?;

        Ok(())
    }
}
