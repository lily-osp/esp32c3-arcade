# Random Arcade

A multi-game arcade launcher for the ESP32-C3, SSD1306 OLED, and MPU-6050. It embeds and runs the actual production sketches from the six individual game folders in separate namespaces:

- `../pong/pong.ino`
- `../snake/snake.ino`
- `../shooter/shooter.ino`
- `../tilt-racer/tilt-racer.ino`
- `../breakout/breakout.ino`
- `../falldown/falldown.ino`

Each round selects a random game without repeating the game played in the previous round.

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

The OLED uses I2C address `0x3C`; the MPU-6050 uses `0x68`. Both share the GPIO9/GPIO8 I2C bus.

## How it works

1. The launcher chooses a random game.
2. It executes that game's `setup()` and `loop()` within an isolated namespace.
3. The selected game retains its mechanics, difficulty roll, power-ups, and game-over state.
4. When the sketch returns to its shake-to-start screen following a round, the launcher selects another game.
5. The next game is guaranteed to differ from the previous one.

## Controls

- Pong: Tilt vertically using MPU X.
- Snake: Tilt 4-way using MPU X/Y.
- Shooter: Tilt vertically using MPU X.
- Tilt Racer: Tilt horizontally using MPU Y.
- Breakout: Tilt horizontally using MPU Y.
- Fall Down: Tilt horizontally using MPU Y.
- Shake the board to begin each selected game.

## Build and upload

```sh
arduino-cli compile --fqbn esp32:esp32:esp32c3 03_projects/games/random-arcade
arduino-cli upload --fqbn esp32:esp32:esp32c3 -p /dev/ttyACM0 03_projects/games/random-arcade
```
