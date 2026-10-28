# Pong

Tilt-controlled Pong for the ESP32-C3, SSD1306 OLED, and MPU6050.

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

The OLED uses I2C address `0x3C`. The MPU6050 uses `0x68`. Both devices share the same I2C bus.

## Controls

- Shake the device to start.
- Tilt vertically to move the player paddle.
- The current board mapping uses MPU X acceleration for vertical screen movement.
- After a match ends, shake again to start the next match.

A three-second countdown (`3`, `2`, `1`, `GO`) runs before play begins.

## Game rules

- The match is first to 10 points.
- Paddle hits change the ball angle based on the point of impact.
- The ball accelerates slightly after each paddle hit, up to a capped speed.
- The AI difficulty is selected at random for each match:
  - Easy: 40%
  - Medium: 30%
  - Hard: 20%
  - Extreme: 10%
- The score is rendered as an overlay so the full OLED height remains playable.

## Build and upload

From the microcontroller project root:

```sh
arduino-cli compile --fqbn esp32:esp32:esp32c3 03_projects/games/pong/pong.ino
arduino-cli upload --fqbn esp32:esp32:esp32c3 -p /dev/ttyACM0 03_projects/games/pong/pong.ino
```

The serial port may differ on another system. Check it with `arduino-cli board list`.
