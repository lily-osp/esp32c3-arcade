use core::fmt::Write;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
    Drawable, Pixel,
};
use heapless::String;

use crate::rng::Rng;

pub const W: i32 = 128;
pub const H: i32 = 64;
pub const CELL: i32 = 4;
pub const COLS: i32 = W / CELL; // 32
pub const ROWS: i32 = H / CELL; // 16
pub const MAXLEN: usize = 90;

pub struct SnakeGame {
    pub snake_x: [i32; MAXLEN],
    pub snake_y: [i32; MAXLEN],
    pub length: usize,
    pub dir_x: i32,
    pub dir_y: i32,
    pub food_x: i32,
    pub food_y: i32,
    pub score: i32,
    pub move_delay_ms: u32,
    pub last_move_ms: u64,
}

impl SnakeGame {
    pub fn new() -> Self {
        Self {
            snake_x: [0; MAXLEN],
            snake_y: [0; MAXLEN],
            length: 7,
            dir_x: 1,
            dir_y: 0,
            food_x: 15,
            food_y: 8,
            score: 0,
            move_delay_ms: 135,
            last_move_ms: 0,
        }
    }

    pub fn reset(&mut self, rng: &mut Rng, now_ms: u64) {
        self.length = 7;
        self.dir_x = 1;
        self.dir_y = 0;
        self.score = 0;
        self.move_delay_ms = 135;
        self.last_move_ms = now_ms;

        for i in 0..self.length {
            self.snake_x[i] = 10 - i as i32;
            self.snake_y[i] = ROWS / 2;
        }
        self.place_food(rng);
    }

    pub fn place_food(&mut self, rng: &mut Rng) {
        let mut occupied = true;
        while occupied {
            self.food_x = rng.gen_range(0, COLS);
            self.food_y = rng.gen_range(0, ROWS);
            occupied = false;
            for i in 0..self.length {
                if self.snake_x[i] == self.food_x && self.snake_y[i] == self.food_y {
                    occupied = true;
                    break;
                }
            }
        }
    }

    pub fn read_tilt(&mut self, ax: f32, ay: f32) {
        let horizontal = ay;
        let vertical = ax;

        if horizontal.abs() > vertical.abs() {
            if horizontal > 3.0 && self.dir_x != -1 {
                self.dir_x = 1;
                self.dir_y = 0;
            } else if horizontal < -3.0 && self.dir_x != 1 {
                self.dir_x = -1;
                self.dir_y = 0;
            }
        } else {
            if vertical > 3.0 && self.dir_y != 1 {
                self.dir_x = 0;
                self.dir_y = -1;
            } else if vertical < -3.0 && self.dir_y != -1 {
                self.dir_x = 0;
                self.dir_y = 1;
            }
        }
    }

    // Returns true if collided with self (Game Over)
    pub fn update(&mut self, rng: &mut Rng, ax: f32, ay: f32, now_ms: u64) -> bool {
        self.read_tilt(ax, ay);

        if now_ms.saturating_sub(self.last_move_ms) < self.move_delay_ms as u64 {
            return false;
        }
        self.last_move_ms = now_ms;

        let mut next_x = self.snake_x[0] + self.dir_x;
        let mut next_y = self.snake_y[0] + self.dir_y;

        // Wrap around borders
        if next_x < 0 {
            next_x = COLS - 1;
        }
        if next_x >= COLS {
            next_x = 0;
        }
        if next_y < 0 {
            next_y = ROWS - 1;
        }
        if next_y >= ROWS {
            next_y = 0;
        }

        // Self-collision check
        for i in 0..self.length {
            if self.snake_x[i] == next_x && self.snake_y[i] == next_y {
                return true; // Game Over
            }
        }

        let ate_food = next_x == self.food_x && next_y == self.food_y;
        if ate_food && self.length < MAXLEN {
            self.length += 1;
        }

        for i in (1..self.length).rev() {
            self.snake_x[i] = self.snake_x[i - 1];
            self.snake_y[i] = self.snake_y[i - 1];
        }
        self.snake_x[0] = next_x;
        self.snake_y[0] = next_y;

        if ate_food {
            self.score += 1;
            if self.move_delay_ms > 55 {
                self.move_delay_ms -= 3;
            }
            self.place_food(rng);
        }

        false
    }

    pub fn draw<D: DrawTarget<Color = BinaryColor>>(&self, target: &mut D) -> Result<(), D::Error> {
        // Snake segments
        for i in 0..self.length {
            let x = self.snake_x[i] * CELL;
            let y = self.snake_y[i] * CELL;
            if i == 0 {
                // Head
                Rectangle::new(Point::new(x, y), Size::new(CELL as u32, CELL as u32))
                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                    .draw(target)?;
                Pixel(Point::new(x + 1, y + 1), BinaryColor::Off).draw(target)?;
            } else {
                // Body
                Rectangle::new(Point::new(x, y), Size::new((CELL - 1) as u32, (CELL - 1) as u32))
                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                    .draw(target)?;
            }
        }

        // Food
        let fx = self.food_x * CELL + 1;
        let fy = self.food_y * CELL + 1;
        Rectangle::new(Point::new(fx, fy), Size::new((CELL - 1) as u32, (CELL - 1) as u32))
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
            .draw(target)?;

        // Score HUD
        let mut hud_str: String<16> = String::new();
        write!(hud_str, "S:{}", self.score).ok();
        let hud_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();
        Text::new(&hud_str, Point::new(0, 7), hud_style).draw(target)?;

        Ok(())
    }
}
