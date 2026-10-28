# Breakout (Tilt Arkanoid)

Classic brick-breaking arcade game designed for the ESP32-C3, SSD1306 128x64 OLED display, and MPU6050 accelerometer.

## Gameplay
- **Paddle Control**: Tilt the ESP32-C3 horizontally (MPU Y-axis) to slide the paddle left and right.
- **Ball Launch**: Tilt to release the ball from the paddle at the start of each round.
- **Bricks**: 4 rows of 8 bricks (32 total). Top row consists of reinforced 2-hit bricks; lower rows take 1 hit.
- **Power-Ups**: 20% of bricks drop special capsules when destroyed:
  - **Wide Paddle**: Expands paddle width for 8 seconds.
  - **Slow Ball**: Reduces ball velocity by 25% for easier catches.
  - **Life +1**: Restores a lost life (max 5).
- **Levels**: Clearing all 32 bricks triggers the next level with a score bonus and faster ball speed.
- **Shake to Start**: Shake the board to trigger countdown.
