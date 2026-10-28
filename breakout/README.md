# Breakout

A tilt-controlled brick breaker for the ESP32-C3, SSD1306 OLED, and MPU-6050.

## Hardware and pinout

| Device | Pin | ESP32-C3 |
|---|---|---|
| SSD1306 OLED | VCC | 3.3V |
| SSD1306 OLED | GND | GND |
| SSD1306 OLED | SDA | GPIO9 |
| SSD1306 OLED | SCL | GPIO8 |
| MPU-6050 | VCC | 3.3V |
| MPU-6050 | GND | GND |
| MPU-6050 | SDA | GPIO9 |
| MPU-6050 | SCL | GPIO8 |

The OLED is at I2C address `0x3C`; the MPU-6050 is at `0x68`. Both share the GPIO9/GPIO8 I2C bus.

## Controls

- The screen displays `BREAKOUT` followed by `SHAKE TO START`.
- Shaking the device initiates a round and announces the randomized difficulty tier, followed by a countdown (`3`, `2`, `1`, `GO`).
- Tilt horizontally (MPU Y-axis) to move the paddle left and right.
- The ball rests on the paddle until tilted horizontally to launch.

## Game rules

- Bricks: 4 rows of 8 bricks (32 total). Reinforced bricks require 2 or 3 hits depending on the selected difficulty tier.
- Power-ups: Destroyed bricks have a chance to drop collectible capsules:
  - Wide paddle: Expands paddle width for 8 seconds.
  - Slow ball: Dampens ball velocity by 25%.
  - Life +1: Restores one life, capped at 5.
- Paddle bounce: Ball reflection angle scales with the distance from the paddle's center.
- Clearing all bricks advances to the next level with a score bonus and increased ball speed.
- The top HUD displays score, remaining lives, and the active difficulty tag (`EZ`, `MD`, `HD`, `XT`).

## Difficulty

A difficulty tier is selected randomly at the start of each match:
- Easy (40%): 26px paddle, 5 lives, 35% power-up rate, 1-hit bricks.
- Medium (30%): 22px paddle, 4 lives, 25% power-up rate, top row 2-hit bricks.
- Hard (20%): 18px paddle, 3 lives, 18% power-up rate, top 2 rows 2-hit bricks.
- Extreme (10%): 15px paddle, 2 lives, 10% power-up rate, top row 3-hit and second row 2-hit bricks.

## Build and upload

```sh
arduino-cli compile --fqbn esp32:esp32:esp32c3 03_projects/games/breakout/breakout.ino
arduino-cli upload --fqbn esp32:esp32:esp32c3 -p /dev/ttyACM0 03_projects/games/breakout/breakout.ino
```
