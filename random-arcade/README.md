# Random Arcade

A unified random game launcher for the ESP32-C3, SSD1306 OLED, and MPU6050. It embeds and runs the actual production sketches from the four game folders, rather than reduced reimplementations:

- `../pong/pong.ino`
- `../snake/snake.ino`
- `../shooter/shooter.ino`
- `../tilt-racer/tilt-racer.ino`

Each round randomly selects one game. A new game is selected after each round without immediately repeating the previous game.

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

The OLED uses I2C address `0x3C`; the MPU6050 uses `0x68`. Both share the GPIO9/GPIO8 I2C bus.

## How it works

1. The launcher chooses a random game.
2. It runs that game's real `setup()` and `loop()` inside a separate namespace.
3. The selected sketch keeps its own mechanics, progression, HUD, difficulty, power-ups, and game-over behavior.
4. When that sketch returns to its shake-to-start state after a completed round, the launcher selects another game.
5. The next game is never the same as the previous one.

## Controls

- Pong: tilt vertically using MPU X; first player to 10 points wins.
- Snake: tilt using MPU X/Y; the snake wraps around the edges.
- Shooter: tilt vertically using MPU X; the ship fires automatically.
- Tilt Racer: tilt horizontally using MPU Y; avoid traffic as the road accelerates.
- Shake after a round ends to start the newly selected game.

## Build and upload

```sh
arduino-cli compile --fqbn esp32:esp32:esp32c3 03_projects/games/random-arcade
arduino-cli upload --fqbn esp32:esp32:esp32c3 -p /dev/ttyACM0 03_projects/games/random-arcade
```
