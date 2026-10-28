use core::fmt::Write;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, Rectangle, Triangle},
    text::Text,
    Drawable, Pixel,
};
use heapless::String;

use crate::rng::Rng;

pub const W: i32 = 128;
pub const H: i32 = 64;
pub const PLAYER_X: i32 = 3;
pub const PLAYER_W: i32 = 12;
pub const PLAYER_H: i32 = 10;
pub const ENEMY_W: i32 = 9;
pub const ENEMY_H: i32 = 7;
pub const BOSS_W: i32 = 20;
pub const BOSS_H: i32 = 14;
pub const BOSS_MAX_HP: i32 = 45;
pub const MIN_BOSS_HP: i32 = 20;
pub const MAX_LIVES: i8 = 12;
pub const MAX_BULLETS: usize = 12;
pub const MAX_HOSTILE_BULLETS: usize = 8;
pub const MAX_ENEMIES: usize = 8;
pub const MAX_EXPLOSIONS: usize = 10;

const HEART_OUTLINE: [u8; 6] = [0x6C, 0x82, 0x82, 0x44, 0x28, 0x10];
const HEART_HALF: [u8; 6] = [0x6C, 0xF2, 0xF2, 0x74, 0x28, 0x10];
const HEART_FULL: [u8; 6] = [0x6C, 0xFE, 0xFE, 0x7C, 0x38, 0x10];

#[derive(Clone, Copy, Default)]
pub struct Bullet {
    pub x: f32,
    pub y: f32,
    pub active: bool,
}

#[derive(Clone, Copy, Default)]
pub struct HostileBullet {
    pub x: f32,
    pub y: f32,
    pub active: bool,
}

#[derive(Clone, Copy, Default)]
pub struct Enemy {
    pub x: f32,
    pub y: f32,
    pub vy: f32,
    pub active: bool,
}

#[derive(Clone, Copy, Default)]
pub struct Explosion {
    pub x: f32,
    pub y: f32,
    pub frame: u8,
    pub active: bool,
}

#[derive(Clone, Copy, Default)]
pub struct Boss {
    pub x: f32,
    pub y: f32,
    pub vy: f32,
    pub hp: i32,
    pub active: bool,
}

pub struct ShooterGame {
    pub player_y: f32,
    pub score: i32,
    pub lives: i8,
    pub enemy_speed: f32,
    pub difficulty: u8,
    pub shot_interval_ms: u32,
    pub spawn_interval_ms: u32,
    pub max_enemy_speed: f32,
    pub double_shot_chance: u32,
    pub enemies_defeated: i32,
    pub next_boss_at: i32,
    pub boss: Boss,
    pub double_shot: bool,
    pub triple_shot: bool,
    pub enemy_fire_unlocked: bool,
    pub powerup_until_ms: u64,
    pub fire_rate_multiplier: f32,
    pub fire_rate_boosts: u8,
    pub last_shot_ms: u64,
    pub last_spawn_ms: u64,
    pub bullets: [Bullet; MAX_BULLETS],
    pub hostile_bullets: [HostileBullet; MAX_HOSTILE_BULLETS],
    pub enemies: [Enemy; MAX_ENEMIES],
    pub explosions: [Explosion; MAX_EXPLOSIONS],
}

impl ShooterGame {
    pub fn new() -> Self {
        Self {
            player_y: 27.0,
            score: 0,
            lives: 7,
            enemy_speed: 1.0,
            difficulty: 0,
            shot_interval_ms: 220,
            spawn_interval_ms: 750,
            max_enemy_speed: 3.0,
            double_shot_chance: 5,
            enemies_defeated: 0,
            next_boss_at: 110,
            boss: Boss::default(),
            double_shot: false,
            triple_shot: false,
            enemy_fire_unlocked: false,
            powerup_until_ms: 0,
            fire_rate_multiplier: 1.0,
            fire_rate_boosts: 0,
            last_shot_ms: 0,
            last_spawn_ms: 0,
            bullets: [Bullet::default(); MAX_BULLETS],
            hostile_bullets: [HostileBullet::default(); MAX_HOSTILE_BULLETS],
            enemies: [Enemy::default(); MAX_ENEMIES],
            explosions: [Explosion::default(); MAX_EXPLOSIONS],
        }
    }

    pub fn reset(&mut self, rng: &mut Rng, now_ms: u64) {
        self.player_y = (H as f32 - PLAYER_H as f32) / 2.0;
        self.score = 0;
        self.lives = 7;
        self.enemies_defeated = 0;
        self.next_boss_at = rng.gen_range(90, 131);
        self.boss = Boss {
            x: W as f32 + BOSS_W as f32,
            y: 25.0,
            vy: 0.6,
            hp: 0,
            active: false,
        };
        self.double_shot = false;
        self.triple_shot = false;
        self.enemy_fire_unlocked = false;
        self.powerup_until_ms = 0;
        self.fire_rate_multiplier = 1.0;
        self.fire_rate_boosts = 0;

        let roll = rng.gen_range(0, 10);
        if roll < 4 {
            self.difficulty = 0;
            self.enemy_speed = 1.0;
            self.spawn_interval_ms = 900;
            self.shot_interval_ms = 190;
            self.max_enemy_speed = 2.0;
            self.double_shot_chance = 5;
        } else if roll < 7 {
            self.difficulty = 1;
            self.enemy_speed = 1.0;
            self.spawn_interval_ms = 750;
            self.shot_interval_ms = 220;
            self.max_enemy_speed = 3.0;
            self.double_shot_chance = 6;
        } else if roll < 9 {
            self.difficulty = 2;
            self.enemy_speed = 2.0;
            self.spawn_interval_ms = 600;
            self.shot_interval_ms = 245;
            self.max_enemy_speed = 4.0;
            self.double_shot_chance = 7;
        } else {
            self.difficulty = 3;
            self.enemy_speed = 3.0;
            self.spawn_interval_ms = 450;
            self.shot_interval_ms = 270;
            self.max_enemy_speed = 5.0;
            self.double_shot_chance = 8;
        }

        self.last_shot_ms = now_ms;
        self.last_spawn_ms = now_ms;

        for b in &mut self.bullets {
            b.active = false;
        }
        for hb in &mut self.hostile_bullets {
            hb.active = false;
        }
        for e in &mut self.enemies {
            e.active = false;
        }
        for ex in &mut self.explosions {
            ex.active = false;
        }
    }

    fn spawn_bullet(&mut self, y: f32) {
        for b in &mut self.bullets {
            if !b.active {
                b.active = true;
                b.x = PLAYER_X as f32 + PLAYER_W as f32;
                b.y = y;
                return;
            }
        }
    }

    fn available_bullet_slots(&self) -> usize {
        self.bullets.iter().filter(|b| !b.active).count()
    }

    pub fn fire(&mut self, now_ms: u64) {
        let current_interval = (self.shot_interval_ms as f32 / self.fire_rate_multiplier) as u64;
        if now_ms.saturating_sub(self.last_shot_ms) < current_interval {
            return;
        }

        let shot_count = if self.triple_shot {
            3
        } else if self.double_shot || self.boss.active {
            2
        } else {
            1
        };

        if self.available_bullet_slots() < shot_count {
            return;
        }

        self.last_shot_ms = now_ms;
        if shot_count == 3 {
            self.spawn_bullet(self.player_y + 1.0);
            self.spawn_bullet(self.player_y + PLAYER_H as f32 / 2.0);
            self.spawn_bullet(self.player_y + PLAYER_H as f32 - 1.0);
        } else if shot_count == 2 {
            self.spawn_bullet(self.player_y + 2.0);
            self.spawn_bullet(self.player_y + PLAYER_H as f32 - 2.0);
        } else {
            self.spawn_bullet(self.player_y + PLAYER_H as f32 / 2.0);
        }
    }

    fn spawn_hostile_bullet(&mut self, x: f32, y: f32) {
        for hb in &mut self.hostile_bullets {
            if !hb.active {
                hb.active = true;
                hb.x = x;
                hb.y = y;
                return;
            }
        }
    }

    pub fn spawn_boss(&mut self, rng: &mut Rng) {
        self.boss.active = true;
        self.boss.x = W as f32 + BOSS_W as f32;
        self.boss.y = rng.gen_range(12, H - BOSS_H - 1) as f32;
        let mut vy = rng.gen_range(4, 9) as f32 / 10.0;
        if rng.gen_range(0, 2) == 1 {
            vy = -vy;
        }
        self.boss.vy = vy;
        self.boss.hp = rng.gen_range(MIN_BOSS_HP, BOSS_MAX_HP + 1);
        self.next_boss_at = self.enemies_defeated + rng.gen_range(90, 131);
    }

    pub fn spawn_enemy(&mut self, rng: &mut Rng, now_ms: u64) {
        if now_ms.saturating_sub(self.last_spawn_ms) < self.spawn_interval_ms as u64 {
            return;
        }
        self.last_spawn_ms = now_ms;

        for e in &mut self.enemies {
            if !e.active {
                e.active = true;
                e.x = W as f32 + 2.0;
                e.y = rng.gen_range(10, H - ENEMY_H - 1) as f32;
                e.vy = rng.gen_range(-5, 6) as f32 / 10.0;
                return;
            }
        }
    }

    fn spawn_explosion(&mut self, x: f32, y: f32) {
        for ex in &mut self.explosions {
            if !ex.active {
                ex.active = true;
                ex.x = x;
                ex.y = y;
                ex.frame = 0;
                return;
            }
        }
    }

    fn apply_score_penalty(&mut self, amount: i32) {
        self.score -= amount;
        if self.score < -1 {
            self.score = -1;
        }
    }

    fn enemy_fire_chance_ppm(&self) -> u32 {
        let base_percent = if self.difficulty <= 1 { 1.5 } else { 2.0 };
        let score_bonus = if self.score > 0 {
            (self.score / 15) as f32 * 0.05
        } else {
            0.0
        };
        ((base_percent + score_bonus) * 10_000.0) as u32
    }

    fn fire_rate_chance_ppm(&self) -> u32 {
        if self.fire_rate_boosts >= 3 {
            return 0;
        }
        if self.fire_rate_boosts >= 1 {
            return 10_000;
        }
        let chance_percent = 0.5 + (self.enemies_defeated / 25) as f32 * 0.5;
        (chance_percent * 10_000.0) as u32
    }

    pub fn update(&mut self, rng: &mut Rng, accel_x: f32, now_ms: u64) {
        // Player tilt movement (X tilts up/down)
        self.player_y += accel_x * -0.75;
        if self.player_y < 10.0 {
            self.player_y = 10.0;
        }
        let max_y = (H - PLAYER_H) as f32;
        if self.player_y > max_y {
            self.player_y = max_y;
        }

        // Powerup expiration
        if (self.double_shot || self.triple_shot) && now_ms >= self.powerup_until_ms {
            self.double_shot = false;
            self.triple_shot = false;
        }

        // Spawning
        self.fire(now_ms);
        self.spawn_enemy(rng, now_ms);

        // Update player bullets
        for b in &mut self.bullets {
            if !b.active {
                continue;
            }
            b.x += 3.5;
            if b.x > W as f32 + 3.0 {
                b.active = false;
            }
        }

        // Update hostile bullets
        for hb in &mut self.hostile_bullets {
            if !hb.active {
                continue;
            }
            hb.x -= 2.2;
            if hb.x < -3.0 {
                hb.active = false;
            }
        }

        // Update enemies
        let ppm = self.enemy_fire_chance_ppm();
        for i in 0..MAX_ENEMIES {
            if !self.enemies[i].active {
                continue;
            }
            self.enemies[i].x -= self.enemy_speed;
            self.enemies[i].y += self.enemies[i].vy;
            if self.enemies[i].y < 10.0 || self.enemies[i].y > (H - ENEMY_H - 1) as f32 {
                self.enemies[i].vy = -self.enemies[i].vy;
                self.enemies[i].y = self.enemies[i].y.clamp(10.0, (H - ENEMY_H - 1) as f32);
            }

            if self.enemies[i].x < -(ENEMY_W as f32) {
                self.enemies[i].active = false;
                self.lives -= 1;
                continue;
            }

            if !self.enemy_fire_unlocked && self.score >= 10 {
                self.enemy_fire_unlocked = true;
            }
            if self.enemy_fire_unlocked && rng.gen_bool_ppm(ppm) {
                let hbx = self.enemies[i].x - 1.0;
                let hby = self.enemies[i].y + ENEMY_H as f32 / 2.0;
                self.spawn_hostile_bullet(hbx, hby);
            }
        }

        // Update boss
        if self.boss.active {
            self.boss.x -= 0.18;
            if self.boss.x < -(BOSS_W as f32) {
                self.boss.x = W as f32 + BOSS_W as f32;
                self.lives -= 1;
                self.apply_score_penalty(10);
            }

            if rng.gen_bool_ppm(30_000) {
                self.spawn_hostile_bullet(self.boss.x - 1.0, self.boss.y + BOSS_H as f32 / 2.0);
            }

            self.boss.y += self.boss.vy;
            if self.boss.y < 10.0 || self.boss.y > (H - BOSS_H - 1) as f32 {
                self.boss.vy = -self.boss.vy;
                self.boss.y = self.boss.y.clamp(10.0, (H - BOSS_H - 1) as f32);
            }
        }

        // Update explosions
        for ex in &mut self.explosions {
            if !ex.active {
                continue;
            }
            ex.frame += 1;
            if ex.frame > 5 {
                ex.active = false;
            }
        }

        // Collision Checks
        self.check_collisions(rng, now_ms);
    }

    fn check_collisions(&mut self, rng: &mut Rng, now_ms: u64) {
        // Bullets vs Boss
        if self.boss.active {
            let boss_visible = self.boss.x >= 0.0
                && self.boss.y >= 0.0
                && (self.boss.x + BOSS_W as f32) <= W as f32
                && (self.boss.y + BOSS_H as f32) <= H as f32;

            if boss_visible {
                let bx = self.boss.x;
                let by = self.boss.y;
                for b_idx in 0..MAX_BULLETS {
                    if !self.bullets[b_idx].active || !self.boss.active {
                        continue;
                    }
                    let hit = self.bullets[b_idx].x >= bx
                        && self.bullets[b_idx].x <= bx + BOSS_W as f32
                        && self.bullets[b_idx].y >= by - 2.0
                        && self.bullets[b_idx].y <= by + BOSS_H as f32;
                    if hit {
                        self.bullets[b_idx].active = false;
                        self.boss.hp -= 1;
                        if self.boss.hp <= 0 {
                            self.boss.active = false;
                            self.lives = (self.lives + 2).min(MAX_LIVES);
                            self.spawn_explosion(
                                bx + BOSS_W as f32 / 2.0,
                                by + BOSS_H as f32 / 2.0,
                            );
                            self.spawn_explosion(bx + 5.0, by + 4.0);
                            self.spawn_explosion(bx + BOSS_W as f32 - 5.0, by + 4.0);
                        }
                    }
                }
            }
        }

        // Bullets vs Enemies
        for b_idx in 0..MAX_BULLETS {
            if !self.bullets[b_idx].active {
                continue;
            }
            for e_idx in 0..MAX_ENEMIES {
                if !self.enemies[e_idx].active {
                    continue;
                }
                let ex = self.enemies[e_idx].x;
                let ey = self.enemies[e_idx].y;
                let enemy_visible =
                    ex >= 0.0 && ey >= 0.0 && (ex + ENEMY_W as f32) <= W as f32 && (ey + ENEMY_H as f32) <= H as f32;
                let hit = enemy_visible
                    && self.bullets[b_idx].x >= ex
                    && self.bullets[b_idx].x <= ex + ENEMY_W as f32
                    && self.bullets[b_idx].y >= ey - 2.0
                    && self.bullets[b_idx].y <= ey + ENEMY_H as f32;
                if hit {
                    self.bullets[b_idx].active = false;
                    self.enemies[e_idx].active = false;
                    self.spawn_explosion(ex + ENEMY_W as f32 / 2.0, ey + ENEMY_H as f32 / 2.0);
                    self.score += 1;
                    self.enemies_defeated += 1;

                    if self.score % 8 == 0 && self.enemy_speed < self.max_enemy_speed {
                        self.enemy_speed += 1.0;
                    }
                    if !self.boss.active && self.enemies_defeated >= self.next_boss_at {
                        self.spawn_boss(rng);
                    }

                    // Triple shot chance: 0.3%
                    if !self.double_shot && !self.triple_shot && rng.gen_bool_ppm(3_000) {
                        self.triple_shot = true;
                        self.powerup_until_ms = now_ms + 7000;
                    } else if !self.double_shot
                        && !self.triple_shot
                        && rng.gen_percent(self.double_shot_chance)
                    {
                        self.double_shot = true;
                        self.powerup_until_ms = now_ms + 5000;
                    }

                    // Permanent fire-rate boost
                    if self.fire_rate_boosts < 3 {
                        let ppm = self.fire_rate_chance_ppm();
                        if rng.gen_bool_ppm(ppm) {
                            self.fire_rate_multiplier *= 1.05;
                            self.fire_rate_boosts += 1;
                        }
                    }
                    break;
                }
            }
        }

        // Hostile bullets vs Player
        for hb_idx in 0..MAX_HOSTILE_BULLETS {
            if !self.hostile_bullets[hb_idx].active {
                continue;
            }
            let hx = self.hostile_bullets[hb_idx].x;
            let hy = self.hostile_bullets[hb_idx].y;
            let hit_player = hx <= (PLAYER_X + PLAYER_W) as f32
                && hx >= PLAYER_X as f32
                && hy >= self.player_y
                && hy <= self.player_y + PLAYER_H as f32;
            if hit_player {
                self.hostile_bullets[hb_idx].active = false;
                self.apply_score_penalty(2);
            }
        }

        // Enemies vs Player
        for e in &mut self.enemies {
            if !e.active {
                continue;
            }
            let hit_player = e.x <= (PLAYER_X + PLAYER_W) as f32
                && (e.x + ENEMY_W as f32) >= PLAYER_X as f32
                && (e.y + ENEMY_H as f32) >= self.player_y
                && e.y <= self.player_y + PLAYER_H as f32;
            if hit_player {
                e.active = false;
                self.lives -= 1;
            }
        }
    }

    pub fn draw<D: DrawTarget<Color = BinaryColor>>(&self, target: &mut D) -> Result<(), D::Error> {
        // Player Ship
        let ship_y = self.player_y as i32;
        // Fuselage
        Rectangle::new(Point::new(PLAYER_X, ship_y + 3), Size::new(6, 4))
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
            .draw(target)?;
        // Nose cone
        Triangle::new(
            Point::new(PLAYER_X + 3, ship_y),
            Point::new(PLAYER_X + PLAYER_W, ship_y + PLAYER_H / 2),
            Point::new(PLAYER_X + 3, ship_y + PLAYER_H),
        )
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        .draw(target)?;
        // Upper and lower wings
        Rectangle::new(Point::new(PLAYER_X + 1, ship_y + 1), Size::new(3, 2))
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
            .draw(target)?;
        Rectangle::new(
            Point::new(PLAYER_X + 1, ship_y + PLAYER_H - 3),
            Size::new(3, 2),
        )
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        .draw(target)?;
        // Cockpit window unlit
        Pixel(Point::new(PLAYER_X + 7, ship_y + 4), BinaryColor::Off).draw(target)?;
        Pixel(Point::new(PLAYER_X + 7, ship_y + 5), BinaryColor::Off).draw(target)?;
        // Engine plumes
        Pixel(Point::new(PLAYER_X - 1, ship_y + 3), BinaryColor::On).draw(target)?;
        Pixel(Point::new(PLAYER_X - 1, ship_y + 6), BinaryColor::On).draw(target)?;

        // Player Bullets (horizontal lines)
        for b in &self.bullets {
            if b.active {
                Line::new(
                    Point::new(b.x as i32, b.y as i32),
                    Point::new((b.x as i32) + 4, b.y as i32),
                )
                .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
                .draw(target)?;
            }
        }

        // Hostile Bullets (circles)
        for hb in &self.hostile_bullets {
            if hb.active {
                Circle::new(Point::new((hb.x as i32) - 1, (hb.y as i32) - 1), 3)
                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                    .draw(target)?;
            }
        }

        // Enemies
        for e in &self.enemies {
            if !e.active {
                continue;
            }
            let x = e.x as i32;
            let y = e.y as i32;
            Rectangle::new(Point::new(x + 2, y), Size::new(5, 2))
                .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                .draw(target)?;
            Rectangle::new(Point::new(x + 1, y + 2), Size::new(7, 3))
                .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                .draw(target)?;
            Pixel(Point::new(x, y + 3), BinaryColor::On).draw(target)?;
            Pixel(Point::new(x + 8, y + 3), BinaryColor::On).draw(target)?;
            Pixel(Point::new(x + 2, y + 5), BinaryColor::On).draw(target)?;
            Pixel(Point::new(x + 6, y + 5), BinaryColor::On).draw(target)?;
            Pixel(Point::new(x + 4, y + 1), BinaryColor::Off).draw(target)?;
        }

        // Explosions
        for ex in &self.explosions {
            if !ex.active {
                continue;
            }
            let x = ex.x as i32;
            let y = ex.y as i32;
            let r = (ex.frame + 1) as i32;
            Pixel(Point::new(x, y), BinaryColor::On).draw(target)?;
            Pixel(Point::new(x - r, y), BinaryColor::On).draw(target)?;
            Pixel(Point::new(x + r, y), BinaryColor::On).draw(target)?;
            Pixel(Point::new(x, y - r), BinaryColor::On).draw(target)?;
            Pixel(Point::new(x, y + r), BinaryColor::On).draw(target)?;
            if ex.frame >= 2 {
                Pixel(Point::new(x - r, y - r), BinaryColor::On).draw(target)?;
                Pixel(Point::new(x + r, y + r), BinaryColor::On).draw(target)?;
                Pixel(Point::new(x + r, y - r), BinaryColor::On).draw(target)?;
                Pixel(Point::new(x - r, y + r), BinaryColor::On).draw(target)?;
            }
        }

        // Boss
        if self.boss.active {
            let x = self.boss.x as i32;
            let y = self.boss.y as i32;
            Rectangle::new(Point::new(x + 4, y), Size::new(12, 2))
                .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                .draw(target)?;
            Rectangle::new(Point::new(x + 2, y + 2), Size::new(16, 8))
                .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                .draw(target)?;
            Rectangle::new(Point::new(x, y + 5), Size::new(BOSS_W as u32, 4))
                .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                .draw(target)?;
            Pixel(Point::new(x + 3, y + 10), BinaryColor::On).draw(target)?;
            Pixel(Point::new(x + 6, y + 11), BinaryColor::On).draw(target)?;
            Pixel(Point::new(x + 13, y + 11), BinaryColor::On).draw(target)?;
            Pixel(Point::new(x + 16, y + 10), BinaryColor::On).draw(target)?;
            Pixel(Point::new(x + 8, y + 4), BinaryColor::Off).draw(target)?;
            Pixel(Point::new(x + 11, y + 4), BinaryColor::Off).draw(target)?;

            // Boss health bar
            Rectangle::new(Point::new(82, 9), Size::new(44, 3))
                .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
                .draw(target)?;
            let hp_width = (self.boss.hp * 42) / BOSS_MAX_HP;
            if hp_width > 0 {
                Rectangle::new(Point::new(83, 10), Size::new(hp_width as u32, 1))
                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                    .draw(target)?;
            }
        }

        // HUD: Score with opaque background
        let mut hud_str: String<16> = String::new();
        write!(hud_str, "S:{}", self.score).ok();
        let hud_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();
        Text::new(&hud_str, Point::new(0, 7), hud_style).draw(target)?;

        // HUD: 4 Heart Slots
        let mut remaining_lives = self.lives;
        for heart in 0..4 {
            let state = if remaining_lives >= 3 {
                remaining_lives -= 3;
                3
            } else if remaining_lives >= 2 {
                remaining_lives -= 2;
                2
            } else if remaining_lives >= 1 {
                remaining_lives -= 1;
                1
            } else {
                0
            };
            draw_heart(target, 91 + heart * 9, 1, state)?;
        }

        Ok(())
    }
}

fn draw_heart<D: DrawTarget<Color = BinaryColor>>(
    target: &mut D,
    x: i32,
    y: i32,
    state: u8,
) -> Result<(), D::Error> {
    if state == 0 {
        return Ok(());
    }
    let bmp = match state {
        3 => HEART_FULL,
        2 => HEART_HALF,
        _ => HEART_OUTLINE,
    };
    for (row, &byte) in bmp.iter().enumerate() {
        for col in 0..7 {
            if (byte & (0x80 >> col)) != 0 {
                Pixel(Point::new(x + col, y + row as i32), BinaryColor::On).draw(target)?;
            }
        }
    }
    Ok(())
}
