# Snake

Full-screen, tilt-controlled Snake with edge wrapping for the ESP32-C3 and 128×64 SSD1306 OLED.

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

I2C addresses are OLED `0x3C` and MPU6050 `0x68`.

## Controls

- The OLED shows a boot splash and `SHAKE TO START`.
- Shake the device to begin. A short boot grace period prevents accidental startup motion.
- Tilt left/right with MPU Y acceleration.
- Tilt up/down with MPU X acceleration.
- The direction changes only when the dominant tilt passes the control threshold.

## Game logic

- The playfield uses the complete 128×64 display.
- There are no walls: the snake wraps around every edge.
- Food is placed on an unoccupied grid cell.
- Eating food increases the length and score.
- The movement interval decreases as the score rises.
- Hitting the snake body shows a centered game-over screen.
- The score is displayed as an overlay (`S:<score>`) without reserving playfield space.
- After game over, another shake is required to restart.

## Build and upload

```sh
arduino-cli compile --fqbn esp32:esp32:esp32c3 03_projects/games/snake/snake.ino
arduino-cli upload --fqbn esp32:esp32:esp32c3 -p /dev/ttyACM0 03_projects/games/snake/snake.ino
```
