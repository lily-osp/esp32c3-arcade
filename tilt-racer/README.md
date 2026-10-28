# Tilt Racer

A tilt-controlled survival racer for the ESP32-C3, SSD1306 OLED, and MPU6050. Steer through a scrolling three-lane road, avoid traffic, and survive as the track accelerates.

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

The OLED uses I2C address `0x3C`; the MPU6050 uses `0x68`. Both devices share the GPIO9/GPIO8 I2C bus.

## Controls and start sequence

- The boot screen waits for a shake.
- Shaking starts a visible three-second countdown: `3`, `2`, `1`, `GO`.
- Tilt left/right to steer. MPU Y acceleration controls screen horizontal movement.
- The car has momentum and damping, so returning the board toward level slows the car instead of stopping it instantly.
- Shake again after game over to replay.

## Game logic

- The player starts with 3 lives.
- Traffic enters from the top and moves downward with the scrolling road.
- The road has three visual lanes, but the player can steer continuously within the road rather than snapping between lanes.
- Obstacles spawn at varied positions across the full road width, including between lane markers and near the edges.
- Spacing checks prevent overlapping traffic from creating an unavoidable wall.
- Colliding with traffic removes one life, centers the player, and grants a brief blinking invincibility period.
- Score increases with survival time.
- The road speed gradually increases up to a capped maximum.
- The compact score/lives overlay is drawn with an opaque background so it remains readable without hiding the road.

## Build and upload

```sh
arduino-cli compile --fqbn esp32:esp32:esp32c3 03_projects/games/tilt-racer
arduino-cli upload --fqbn esp32:esp32:esp32c3 -p /dev/ttyACM0 03_projects/games/tilt-racer
```
