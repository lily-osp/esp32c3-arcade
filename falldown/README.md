# Fall Down

A tilt-controlled vertical descent game for the ESP32-C3, SSD1306 OLED, and MPU-6050.

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

The OLED is at I2C address `0x3C`; the MPU-6050 is at `0x68`. Both devices share the GPIO9/GPIO8 I2C bus.

## Controls

- The screen displays `FALL DOWN` followed by `SHAKE TO START`.
- Shaking the device starts the round with an announced difficulty tier and a countdown (`3`, `2`, `1`, `GO`).
- Tilt horizontally (MPU Y-axis) to steer the marble left and right.
- Momentum and friction allow smooth steering and wall bounces.

## Game rules

- Floors continuously rise toward the top of the display.
- Each floor contains a gap. Tilt the marble to fall through gaps before reaching the ceiling.
- Descending through a gap awards 10 points.
- Touching the hazard spikes at the ceiling deducts one life and respawns the marble lower on the screen with temporary invulnerability.
- Reaching zero lives triggers game over.

## Collectibles

Platforms occasionally spawn items:
- Bonus star: +50 points.
- Slow motion: Halves floor ascent speed for 4 seconds (`SLOW` status on HUD).
- Extra life: Adds 1 life, capped at 5.

## Difficulty

A difficulty tier is selected randomly at the start of each match:
- Easy (40%): 26px gap width, 0.52 px/frame ascent speed, 4 lives, 35% gem spawn, static gaps.
- Medium (30%): 21px gap width, 0.68 px/frame ascent speed, 3 lives, 25% gem spawn, 30% horizontally moving gaps.
- Hard (20%): 16px gap width, 0.85 px/frame ascent speed, 2 lives, 18% gem spawn, 60% moving gaps.
- Extreme (10%): 13px gap width, 1.05 px/frame ascent speed, 1 life, 12% gem spawn, 80% fast-moving gaps.

## Build and upload

```sh
arduino-cli compile --fqbn esp32:esp32:esp32c3 03_projects/games/falldown/falldown.ino
arduino-cli upload --fqbn esp32:esp32:esp32c3 -p /dev/ttyACM0 03_projects/games/falldown/falldown.ino
```
