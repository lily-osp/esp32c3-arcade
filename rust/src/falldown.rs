use core::fmt::Write;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, Rectangle},
    text::Text,
    Drawable, Pixel,
};
use heapless::String;

use crate::rng::Rng;

pub const W: i32 = 128;
pub const H: i32 = 64;
pub const HUD_H: i32 = 8;
pub const CEILING_Y: f32 = 10.0;
pub const BALL_RADIUS: f32 = 2.0;
pub const FLOOR_H: i32 = 2;
pub const NUM_FLOORS: usize = 5;
pub const FLOOR_SPACING: f32 = 13.0;

#[derive(Clone, Copy)]
pub struct Floor {
    pub y: f32,
    pub gap_x: f32,
    pub gap_w: f32,
    pub gap_vx: f32,
    pub passed: bool,
    pub gem_active: bool,
    pub gem_x: f32,
    pub gem_type: u8, // 0 = Bonus (+50), 1 = Slow (4s), 2 = Life (+1)
}

impl Default for Floor {
    fn default() -> Self {
        Self {
            y: 0.0,
            gap_x: 40.0,
            gap_w: 24.0,
            gap_vx: 0.0,
            passed: false,
            gem_active: false,
            gem_x: 0.0,
            gem_type: 0,
        }
    }
}

fn generate_floor(y: f32, difficulty: u8, base_gap_w: f32, gem_chance: u32, rng: &mut Rng) -> Floor {
    let gap_w = base_gap_w;
    let max_gap_x = (W as f32) - 4.0 - gap_w;
    let gap_x = rng.gen_range(4, max_gap_x as i32) as f32;

    // Gap drift velocity for harder modes
    let gap_vx = if difficulty == 1 && rng.gen_percent(30) {
        if rng.gen_percent(50) { 0.35 } else { -0.35 }
    } else if difficulty >= 2 && rng.gen_percent(60) {
        let sign = if rng.gen_percent(50) { 1.0 } else { -1.0 };
        sign * (0.4 + (difficulty as f32 * 0.15))
    } else {
        0.0
    };

    // Determine gem placement
    let has_gem = rng.gen_percent(gem_chance);
    let mut gem_x = 10.0;
    let mut gem_type = 0;
    if has_gem {
        let gem_roll = rng.gen_range(0, 10);
        gem_type = if gem_roll < 5 {
            0 // Bonus +50 pts (50%)
        } else if gem_roll < 8 {
            1 // Slow motion 4s (30%)
        } else {
            2 // Extra life (20%)
        };

        // Spawn gem on left or right solid segment
        if gap_x > 20.0 && rng.gen_percent(50) {
            gem_x = rng.gen_range(4, (gap_x - 4.0) as i32) as f32;
        } else {
            let right_start = (gap_x + gap_w + 4.0) as i32;
            if right_start < W - 6 {
                gem_x = rng.gen_range(right_start, W - 6) as f32;
            } else {
                gem_x = 8.0;
            }
        }
    }

    Floor {
        y,
        gap_x,
        gap_w,
        gap_vx,
        passed: false,
        gem_active: has_gem,
        gem_x,
        gem_type,
    }
}

pub struct FallDownGame {
    pub ball_x: f32,
    pub ball_y: f32,
    pub ball_vx: f32,
    pub ball_vy: f32,
    pub score: i32,
    pub lives: i32,
    pub difficulty: u8, // 0 = Easy, 1 = Medium, 2 = Hard, 3 = Extreme
    pub base_gap_w: f32,
    pub base_rise_speed: f32,
    pub max_rise_speed: f32,
    pub gem_chance: u32,
    pub floors: [Floor; NUM_FLOORS],
    pub slow_motion_until_ms: u64,
    pub invulnerable_until_ms: u64,
}

impl FallDownGame {
    pub fn new() -> Self {
        Self {
            ball_x: 64.0,
            ball_y: 20.0,
            ball_vx: 0.0,
            ball_vy: 0.0,
            score: 0,
            lives: 3,
            difficulty: 0,
            base_gap_w: 24.0,
            base_rise_speed: 0.55,
            max_rise_speed: 1.10,
            gem_chance: 30,
            floors: [Floor::default(); NUM_FLOORS],
            slow_motion_until_ms: 0,
            invulnerable_until_ms: 0,
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

    pub fn reset_game(&mut self, rng: &mut Rng, now_ms: u64) {
        self.score = 0;
        self.slow_motion_until_ms = 0;
        self.invulnerable_until_ms = now_ms + 1500;

        // 4-tier weighted random difficulty: Easy 40%, Medium 30%, Hard 20%, Extreme 10%
        let roll = rng.gen_range(0, 10);
        if roll < 4 {
            self.difficulty = 0; // Easy
            self.base_gap_w = 26.0;
            self.base_rise_speed = 0.52;
            self.max_rise_speed = 0.95;
            self.lives = 4;
            self.gem_chance = 35;
        } else if roll < 7 {
            self.difficulty = 1; // Medium
            self.base_gap_w = 21.0;
            self.base_rise_speed = 0.68;
            self.max_rise_speed = 1.15;
            self.lives = 3;
            self.gem_chance = 25;
        } else if roll < 9 {
            self.difficulty = 2; // Hard
            self.base_gap_w = 16.0;
            self.base_rise_speed = 0.85;
            self.max_rise_speed = 1.35;
            self.lives = 2;
            self.gem_chance = 18;
        } else {
            self.difficulty = 3; // Extreme
            self.base_gap_w = 13.0;
            self.base_rise_speed = 1.05;
            self.max_rise_speed = 1.55;
            self.lives = 1;
            self.gem_chance = 12;
        }

        // Initialize evenly spaced floors
        for i in 0..NUM_FLOORS {
            let start_y = (H as f32) - (i as f32 * FLOOR_SPACING);
            self.floors[i] = generate_floor(start_y, self.difficulty, self.base_gap_w, self.gem_chance, rng);
            // Don't spawn gems on starting safe floors
            self.floors[i].gem_active = false;
        }

        // Place ball above first floor
        self.ball_x = 64.0;
        self.ball_y = 18.0;
        self.ball_vx = 0.0;
        self.ball_vy = 0.0;
    }

    // Returns: Some(false) if game over, None if ongoing
    pub fn update(&mut self, rng: &mut Rng, ay: f32, now_ms: u64) -> Option<bool> {
        let difficulty = self.difficulty;
        let base_gap_w = self.base_gap_w;
        let gem_chance = self.gem_chance;

        // Calculate current floor rise speed
        let speed_progression = (self.score as f32 / 100.0) * 0.03;
        let mut rise_speed = (self.base_rise_speed + speed_progression).min(self.max_rise_speed);
        if now_ms < self.slow_motion_until_ms {
            rise_speed *= 0.50; // Slow-mo powerup active
        }

        // Horizontal tilt steering
        // Tilt sensitivity with friction
        let steer_accel = ay * 0.58;
        self.ball_vx = (self.ball_vx + steer_accel) * 0.86;
        self.ball_x += self.ball_vx;

        // Wall collisions
        let min_x = BALL_RADIUS + 1.0;
        let max_x = (W as f32) - BALL_RADIUS - 1.0;
        if self.ball_x <= min_x {
            self.ball_x = min_x;
            self.ball_vx = -self.ball_vx * 0.25;
        } else if self.ball_x >= max_x {
            self.ball_x = max_x;
            self.ball_vx = -self.ball_vx * 0.25;
        }

        // Gravity
        self.ball_vy = (self.ball_vy + 0.30).min(3.6);
        let prev_ball_y = self.ball_y;
        self.ball_y += self.ball_vy;

        // Floor collision and passing logic
        let mut on_floor = false;
        let mut floor_lift_y = 0.0;

        for floor in &mut self.floors {
            let floor_top = floor.y;

            // Check if ball crossed this floor downwards
            let ball_bottom = self.ball_y + BALL_RADIUS;
            let prev_ball_bottom = prev_ball_y + BALL_RADIUS;

            // Ball passed floor height
            if prev_ball_bottom <= floor_top + 1.5 && ball_bottom >= floor_top {
                // Check if ball is inside the gap
                let in_gap = self.ball_x >= (floor.gap_x + 1.0)
                    && self.ball_x <= (floor.gap_x + floor.gap_w - 1.0);

                if in_gap {
                    // Passed through gap!
                    if !floor.passed {
                        floor.passed = true;
                        self.score += 10;
                    }
                } else {
                    // Ball rests on top of floor
                    self.ball_y = floor_top - BALL_RADIUS;
                    self.ball_vy = 0.0;
                    on_floor = true;
                    floor_lift_y = rise_speed;
                }
            }

            // Gem collection
            if floor.gem_active {
                let dist_x = (self.ball_x - floor.gem_x).abs();
                let dist_y = (self.ball_y - (floor.y - 2.0)).abs();
                if dist_x < 5.0 && dist_y < 5.0 {
                    floor.gem_active = false;
                    match floor.gem_type {
                        0 => self.score += 50, // Bonus score
                        1 => self.slow_motion_until_ms = now_ms + 4000, // 4 sec slow motion
                        _ => {
                            if self.lives < 5 {
                                self.lives += 1;
                            } else {
                                self.score += 100;
                            }
                        }
                    }
                }
            }
        }

        // If ball is resting on a floor, lift it up with the floor
        if on_floor {
            self.ball_y -= floor_lift_y;
        }

        // Move floors upward and drift moving gaps
        let mut highest_floor_y = 0.0f32;
        for floor in &self.floors {
            if floor.y > highest_floor_y {
                highest_floor_y = floor.y;
            }
        }

        for floor in &mut self.floors {
            floor.y -= rise_speed;

            // Moving gap update
            if floor.gap_vx.abs() > 0.01 {
                floor.gap_x += floor.gap_vx;
                let max_gx = (W as f32) - 4.0 - floor.gap_w;
                if floor.gap_x <= 4.0 {
                    floor.gap_x = 4.0;
                    floor.gap_vx = -floor.gap_vx;
                } else if floor.gap_x >= max_gx {
                    floor.gap_x = max_gx;
                    floor.gap_vx = -floor.gap_vx;
                }
            }

            // Floor scrolled above ceiling -> recycle to bottom
            if floor.y < (HUD_H as f32 - 1.0) {
                let new_y = highest_floor_y.max(H as f32) + FLOOR_SPACING;
                *floor = generate_floor(new_y, difficulty, base_gap_w, gem_chance, rng);
                highest_floor_y = new_y;
            }
        }

        // Ceiling collision (crushed against hazard ceiling)
        if self.ball_y - BALL_RADIUS <= CEILING_Y {
            if now_ms >= self.invulnerable_until_ms {
                self.lives -= 1;
                if self.lives <= 0 {
                    return Some(false); // Game Over
                }

                // Respawn invulnerable and drop ball lower
                self.ball_y = 36.0;
                self.ball_x = 64.0;
                self.ball_vy = 0.0;
                self.invulnerable_until_ms = now_ms + 1800;
            } else {
                // Currently invulnerable, nudge away from ceiling
                self.ball_y = CEILING_Y + BALL_RADIUS + 1.0;
                self.ball_vy = 1.0;
            }
        }

        // Bottom boundary (ball falls off bottom screen, e.g. free fall)
        if self.ball_y + BALL_RADIUS > (H as f32) {
            self.ball_y = (H as f32) - BALL_RADIUS;
            self.ball_vy = 0.0;
        }

        None
    }

    pub fn draw<D>(&self, target: &mut D, now_ms: u64) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        // Ceiling spikes / hazard line
        Line::new(Point::new(0, HUD_H), Point::new(W - 1, HUD_H))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(target)?;

        // Little spikes under ceiling line every 6 pixels
        let mut sx = 3;
        while sx < W - 3 {
            Pixel(Point::new(sx, HUD_H + 1), BinaryColor::On).draw(target)?;
            sx += 6;
        }

        // Floors
        for floor in &self.floors {
            let fy = floor.y as i32;
            if fy < HUD_H || fy >= H {
                continue;
            }

            let gx = floor.gap_x as i32;
            let gw = floor.gap_w as i32;

            // Left segment
            if gx > 0 {
                Rectangle::new(Point::new(0, fy), Size::new(gx as u32, FLOOR_H as u32))
                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                    .draw(target)?;
            }

            // Right segment
            let right_x = gx + gw;
            if right_x < W {
                let right_w = W - right_x;
                Rectangle::new(Point::new(right_x, fy), Size::new(right_w as u32, FLOOR_H as u32))
                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                    .draw(target)?;
            }

            // Gem item on platform
            if floor.gem_active {
                let gem_x = floor.gem_x as i32;
                let gem_y = (floor.y - 3.0) as i32;
                if gem_y >= HUD_H && gem_y < H - 2 {
                    match floor.gem_type {
                        0 => {
                            // Bonus Star: diamond shape
                            Pixel(Point::new(gem_x, gem_y - 2), BinaryColor::On).draw(target)?;
                            Pixel(Point::new(gem_x - 1, gem_y - 1), BinaryColor::On).draw(target)?;
                            Pixel(Point::new(gem_x + 1, gem_y - 1), BinaryColor::On).draw(target)?;
                            Pixel(Point::new(gem_x, gem_y), BinaryColor::On).draw(target)?;
                        }
                        1 => {
                            // Slow Motion: horizontal bar with inner dot
                            Rectangle::new(Point::new(gem_x - 2, gem_y - 2), Size::new(5, 3))
                                .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                                .draw(target)?;
                            Pixel(Point::new(gem_x, gem_y - 1), BinaryColor::Off).draw(target)?;
                        }
                        _ => {
                            // Life Heart: 3x3 heart shape
                            Pixel(Point::new(gem_x - 1, gem_y - 2), BinaryColor::On).draw(target)?;
                            Pixel(Point::new(gem_x + 1, gem_y - 2), BinaryColor::On).draw(target)?;
                            Pixel(Point::new(gem_x, gem_y - 1), BinaryColor::On).draw(target)?;
                            Pixel(Point::new(gem_x, gem_y), BinaryColor::On).draw(target)?;
                        }
                    }
                }
            }
        }

        // Ball (blink if invulnerable)
        let is_visible = if now_ms < self.invulnerable_until_ms {
            (now_ms / 100) % 2 == 0
        } else {
            true
        };

        if is_visible {
            let bx = self.ball_x as i32;
            let by = self.ball_y as i32;
            Circle::new(Point::new(bx - 2, by - 2), 5)
                .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                .draw(target)?;
            // Glint pixel inside
            Pixel(Point::new(bx, by), BinaryColor::Off).draw(target)?;
        }

        // HUD: Top Bar
        let mut hud_str: String<28> = String::new();
        let diff_tag = match self.difficulty {
            0 => "EZ",
            1 => "MD",
            2 => "HD",
            _ => "XT",
        };

        if now_ms < self.slow_motion_until_ms {
            write!(hud_str, "S:{} L:{} SLOW", self.score, self.lives).ok();
        } else {
            write!(hud_str, "S:{} L:{} {}", self.score, self.lives, diff_tag).ok();
        }

        let hud_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();
        Text::new(&hud_str, Point::new(1, 7), hud_style).draw(target)?;

        Ok(())
    }
}
