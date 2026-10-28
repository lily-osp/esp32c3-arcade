# Shooter

A horizontal, tilt-controlled shooter for the ESP32-C3. The ship is on the left, bullets travel right, and enemies enter from the right.

## Hardware and pinout

| Device | Pin | ESP32-C3 |
|---|---|---|
| SSD1306 OLED | VCC | 3.3V |
| SSD1306 OLED | GND | GND |
| SSD1306 OLED | SDA | GPIO9 |
| SSD1306 OLED | SCL | GPIO8 |
| MPU6050 | VCC | 3.3V |
| MPU6050 | GND | GND |
| MPU6050 | SDA | GPIO9 |
| MPU6050 | SCL | GPIO8 |

The OLED is at `0x3C`; the MPU6050 is at `0x68`. Both share the GPIO9/GPIO8 I2C bus.

## Controls and start sequence

- The OLED shows a boot splash followed by `SHAKE TO START`.
- A stronger shake starts a three-second countdown: `3`, `2`, `1`, `GO`.
- Tilt vertically to move the ship. This board uses MPU X acceleration for screen up/down movement.
- The ship fires automatically.

## Core rules

- The player starts with 7 lives; the maximum is 12.
- Lives are rendered as four heart slots without an `L:` label. Each slot represents three lives: full = 3, half = 2, outline = 1. For example, 7 lives displays as full + full + outline, while 12 displays as four full hearts.
- Missing an enemy or colliding with one costs a life.
- The score is shown as a compact overlay; the full 128×64 screen remains available to gameplay.
- Enemy speed increases as kills accumulate.
- Explosions are rendered when enemies are destroyed.
- Player bullets can only destroy a regular enemy or boss after its entire sprite is inside the display.
- Game over returns to the shake-to-start screen.

## Difficulty

A mode is selected randomly for each game:

- Easy: 40% — slower enemies and spawn rate
- Medium: 30% — balanced settings
- Hard: 20% — faster enemies and spawns
- Extreme: 10% — fastest settings

## Power-ups and progression

- Each regular enemy has a difficulty-scaled chance to activate double-shot for 5 seconds:
  - Easy 5%, Medium 6%, Hard 7%, Extreme 8%
- There is also a 0.3% chance for a regular defeat to grant triple-shot instead of double-shot. Triple-shot emits three bullets atomically for 7 seconds.
- Double-shot lasts 5 seconds and triple-shot lasts 7 seconds. Both timers are hidden from the HUD.
- Fire-rate bonuses are permanent until game over and stack up to three times. Their count is kept internally and is hidden from the HUD.
- The first fire-rate bonus starts at a 0.5% chance and increases by 0.5% every 25 regular kills. The second and third each use a capped 1% chance.
- Each bonus increases the current fire rate by 5%: `1.00×`, `1.05×`, `1.1025×`, then `1.1576×`.

## Bosses

- A boss appears after a random 90–130 regular enemy defeats.
- The next boss interval is randomized independently.
- Each boss has random health from 20 to 45 HP, moves slowly from right to left, and shows a health bar.
- A boss forces double-shot while it is active and has a 3% chance per update to fire at the player.
- Regular enemies can also fire at the player. Their base per-update chance is Easy 1.5%, Medium 1.5%, Hard 2%, or Extreme 2%, increasing by 0.05% for every 15 current score points. The chance falls again when score falls.
- Enemy projectiles are disabled until the score first reaches 10. Once unlocked, enemy fire remains enabled for the rest of the round even if score falls.
- Being hit by an enemy projectile removes 2 score points and does not cost a life. Repeated hits can reach the `-1` game-over sentinel.
- Defeating a boss does not increase score. It restores 2 lives, capped at 20, and creates a larger explosion.
- If a boss reaches the left edge undefeated, it costs 1 life and 10 score, with score penalties able to reach the `-1` game-over sentinel. It then re-enters from the right.
- Any score of `-1` ends the game.

## Build and upload

```sh
arduino-cli compile --fqbn esp32:esp32:esp32c3 03_projects/games/shooter/shooter.ino
arduino-cli upload --fqbn esp32:esp32:esp32c3 -p /dev/ttyACM0 03_projects/games/shooter/shooter.ino
```
