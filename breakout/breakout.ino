#include <Arduino.h>
#include <Wire.h>
#include <Adafruit_GFX.h>
#include <Adafruit_SSD1306.h>
#include <Adafruit_MPU6050.h>
#include <Adafruit_Sensor.h>

#define W 128
#define H 64
#define PADDLE_Y 58
#define PADDLE_H 4
#define BALL_SIZE 3
#define BRICK_ROWS 4
#define BRICK_COLS 8
#define BRICK_W 13
#define BRICK_H 4
#define SHAKE_THRESHOLD 6.0

Adafruit_SSD1306 d(W, H, &Wire, -1);
Adafruit_MPU6050 mpu;

struct Brick {
  uint8_t hp;
  bool hasPowerup;
};

struct Powerup {
  float x, y;
  uint8_t kind; // 0 = Wide, 1 = Slow, 2 = Life
  bool active;
};

Brick bricks[BRICK_ROWS][BRICK_COLS];
Powerup powerups[3];
int basePaddleW = 26;
int paddleW = 26;
float paddleX = (W - 26) / 2.0;
float ballX = W / 2.0;
float ballY = PADDLE_Y - BALL_SIZE - 1;
float ballVX = 1.1;
float ballVY = -1.3;
int score = 0;
int lives = 5;
int level = 1;
uint8_t difficulty = 0; // 0: Easy, 1: Medium, 2: Hard, 3: Extreme
int powerupChance = 35;
float baseSpeed = 1.1;
float maxSpeed = 2.8;
unsigned long widePaddleUntil = 0;
bool ballStuck = true;

const char* getDifficultyName() {
  switch (difficulty) {
    case 0: return "EASY";
    case 1: return "MEDIUM";
    case 2: return "HARD";
    default: return "EXTREME";
  }
}

const char* getDifficultyTag() {
  switch (difficulty) {
    case 0: return "EZ";
    case 1: return "MD";
    case 2: return "HD";
    default: return "XT";
  }
}

unsigned long lastFrame = 0;
unsigned long shakeReadyAt = 0;
bool waitingForShake = true;
float previousAx = 0;
float previousAy = 0;
float previousAz = 0;

void drawCentered(const char *text, int size, int y) {
  d.setTextSize(size);
  int16_t x1, y1;
  uint16_t tw, th;
  d.getTextBounds(text, 0, 0, &x1, &y1, &tw, &th);
  d.setCursor((W - tw) / 2, y);
  d.print(text);
}

void resetBall() {
  ballStuck = true;
  ballX = paddleX + paddleW / 2.0;
  ballY = PADDLE_Y - BALL_SIZE - 1;
  float speedMult = 1.0 + ((level - 1) * 0.10);
  float speed = min(baseSpeed * speedMult, maxSpeed);
  ballVX = speed * 0.75;
  ballVY = -speed;
}

void resetLevel() {
  for (int r = 0; r < BRICK_ROWS; r++) {
    uint8_t hp = 1;
    if (difficulty == 1 && r == 0) hp = 2;
    else if (difficulty == 2 && r <= 1) hp = 2;
    else if (difficulty == 3) {
      if (r == 0) hp = 3;
      else if (r == 1) hp = 2;
    }
    for (int c = 0; c < BRICK_COLS; c++) {
      bricks[r][c].hp = hp;
      bricks[r][c].hasPowerup = (random(100) < powerupChance);
    }
  }
  resetBall();
}

void resetGame() {
  score = 0;
  level = 1;
  widePaddleUntil = 0;
  for (int i = 0; i < 3; i++) powerups[i].active = false;

  // 4-Tier Weighted Difficulty: Easy 40%, Medium 30%, Hard 20%, Extreme 10%
  int roll = random(10);
  if (roll < 4) {
    difficulty = 0; // Easy
    basePaddleW = 26;
    powerupChance = 35;
    lives = 5;
    baseSpeed = 1.1;
    maxSpeed = 2.8;
  } else if (roll < 7) {
    difficulty = 1; // Medium
    basePaddleW = 22;
    powerupChance = 25;
    lives = 4;
    baseSpeed = 1.4;
    maxSpeed = 3.4;
  } else if (roll < 9) {
    difficulty = 2; // Hard
    basePaddleW = 18;
    powerupChance = 18;
    lives = 3;
    baseSpeed = 1.7;
    maxSpeed = 4.0;
  } else {
    difficulty = 3; // Extreme
    basePaddleW = 15;
    powerupChance = 10;
    lives = 2;
    baseSpeed = 2.0;
    maxSpeed = 4.6;
  }

  paddleW = basePaddleW;
  paddleX = (W - paddleW) / 2.0;
  resetLevel();
}

void showStartScreen() {
  d.clearDisplay();
  d.setTextColor(SSD1306_WHITE);
  drawCentered("BREAKOUT", 2, 10);
  drawCentered("SHAKE TO START", 1, 43);
  d.display();
}

void showCountdown() {
  // Show Difficulty banner
  d.clearDisplay();
  d.setTextColor(SSD1306_WHITE);
  drawCentered("BREAKOUT", 2, 10);
  char modeStr[20];
  snprintf(modeStr, sizeof(modeStr), "MODE: %s", getDifficultyName());
  drawCentered(modeStr, 1, 43);
  d.display();
  delay(1000);

  for (int number = 3; number >= 1; number--) {
    d.clearDisplay();
    d.setTextColor(SSD1306_WHITE);
    char text[2];
    snprintf(text, sizeof(text), "%d", number);
    drawCentered(text, 4, 14);
    d.display();
    delay(1000);
  }

  d.clearDisplay();
  drawCentered("GO", 2, 20);
  d.display();
  delay(300);
}

void showGameOver() {
  d.clearDisplay();
  d.setTextColor(SSD1306_WHITE);
  drawCentered("GAME OVER", 2, 14);
  char result[20];
  snprintf(result, sizeof(result), "SCORE %d", score);
  drawCentered(result, 1, 43);
  d.display();
  delay(1500);

  resetGame();
  sensors_event_t a, g, t;
  mpu.getEvent(&a, &g, &t);
  previousAx = a.acceleration.x;
  previousAy = a.acceleration.y;
  previousAz = a.acceleration.z;
  shakeReadyAt = millis() + 1000;
  waitingForShake = true;
}

void spawnPowerup(float x, float y) {
  for (int i = 0; i < 3; i++) {
    if (!powerups[i].active) {
      powerups[i].active = true;
      powerups[i].x = x;
      powerups[i].y = y;
      powerups[i].kind = random(0, 3);
      return;
    }
  }
}

int countActiveBricks() {
  int count = 0;
  for (int r = 0; r < BRICK_ROWS; r++) {
    for (int c = 0; c < BRICK_COLS; c++) {
      if (bricks[r][c].hp > 0) count++;
    }
  }
  return count;
}

void updatePaddle(float ay) {
  paddleX += ay * 1.15;
  paddleX = constrain(paddleX, 0, W - paddleW);
  if (ballStuck) {
    ballX = paddleX + paddleW / 2.0;
    ballY = PADDLE_Y - BALL_SIZE - 1;
  }
}

void drawGame() {
  d.clearDisplay();

  // Bricks
  for (int r = 0; r < BRICK_ROWS; r++) {
    int by = 10 + r * (BRICK_H + 2);
    for (int c = 0; c < BRICK_COLS; c++) {
      if (bricks[r][c].hp == 0) continue;
      int bx = 4 + c * (BRICK_W + 2);
      if (bricks[r][c].hp >= 2) {
        d.fillRect(bx, by, BRICK_W, BRICK_H, 1);
        d.drawPixel(bx + 1, by + 1, 0);
        d.drawPixel(bx + BRICK_W - 2, by + 1, 0);
      } else {
        d.drawRect(bx, by, BRICK_W, BRICK_H, 1);
      }
    }
  }

  // Paddle & Ball
  d.fillRect((int)paddleX, PADDLE_Y, paddleW, PADDLE_H, 1);
  d.fillRect((int)ballX, (int)ballY, BALL_SIZE, BALL_SIZE, 1);

  // Powerups
  for (int i = 0; i < 3; i++) {
    if (powerups[i].active) {
      d.fillCircle((int)powerups[i].x, (int)powerups[i].y, 2, 1);
      d.drawPixel((int)powerups[i].x, (int)powerups[i].y, 0);
    }
  }

  // HUD
  char hud[28];
  snprintf(hud, sizeof(hud), "S:%d L:%d %s", score, lives, getDifficultyTag());
  d.setTextSize(1);
  d.setTextColor(SSD1306_WHITE, SSD1306_BLACK);
  d.setCursor(1, 0);
  d.print(hud);
  d.setTextColor(SSD1306_WHITE);

  d.display();
}

void setup() {
  Wire.begin(9, 8);
  Wire.setClock(800000);
  d.begin(SSD1306_SWITCHCAPVCC, 0x3C);
  d.clearDisplay();
  d.setTextColor(SSD1306_WHITE);
  drawCentered("BREAKOUT", 2, 22);
  d.display();
  delay(500);

  randomSeed(analogRead(0));
  mpu.begin(0x68, &Wire);
  mpu.setAccelerometerRange(MPU6050_RANGE_8_G);
  mpu.setFilterBandwidth(MPU6050_BAND_21_HZ);

  resetGame();
  sensors_event_t a, g, t;
  mpu.getEvent(&a, &g, &t);
  previousAx = a.acceleration.x;
  previousAy = a.acceleration.y;
  previousAz = a.acceleration.z;
  shakeReadyAt = millis() + 1000;
  showStartScreen();
}

void loop() {
  if (millis() - lastFrame < 20) {
    delay(2);
    return;
  }
  lastFrame = millis();

  sensors_event_t a, g, t;
  mpu.getEvent(&a, &g, &t);

  if (waitingForShake) {
    showStartScreen();
    float shake = fabs(a.acceleration.x - previousAx) +
                  fabs(a.acceleration.y - previousAy) +
                  fabs(a.acceleration.z - previousAz);
    previousAx = a.acceleration.x;
    previousAy = a.acceleration.y;
    previousAz = a.acceleration.z;

    if (millis() >= shakeReadyAt && shake > SHAKE_THRESHOLD) {
      waitingForShake = false;
      resetGame();
      showCountdown();
      lastFrame = millis();
    } else {
      delay(25);
      return;
    }
  }

  updatePaddle(a.acceleration.y);

  if (millis() >= widePaddleUntil) {
    paddleW = basePaddleW;
  }

  if (ballStuck) {
    if (fabs(a.acceleration.y) > 2.0) {
      ballStuck = false;
      ballVX = (a.acceleration.y > 0) ? 1.6 : -1.6;
    }
    drawGame();
    return;
  }

  ballX += ballVX;
  ballY += ballVY;

  if (ballX <= 0) { ballX = 0; ballVX = fabs(ballVX); }
  if (ballX >= W - BALL_SIZE) { ballX = W - BALL_SIZE; ballVX = -fabs(ballVX); }
  if (ballY <= 8) { ballY = 8; ballVY = fabs(ballVY); }

  // Bounce paddle
  if (ballVY > 0 &&
      ballY + BALL_SIZE >= PADDLE_Y &&
      ballY + BALL_SIZE <= PADDLE_Y + PADDLE_H + 2 &&
      ballX + BALL_SIZE >= paddleX &&
      ballX <= paddleX + paddleW) {
    ballY = PADDLE_Y - BALL_SIZE;
    float center = paddleX + paddleW / 2.0;
    float offset = (ballX + BALL_SIZE / 2.0 - center) / (paddleW / 2.0);
    float speed = sqrt(ballVX * ballVX + ballVY * ballVY);
    speed = constrain(speed * 1.02, baseSpeed, maxSpeed);
    ballVX = offset * (speed * 0.85);
    ballVY = -sqrt(max(1.0f, speed * speed - ballVX * ballVX));
  }

  // Bricks collision
  for (int r = 0; r < BRICK_ROWS; r++) {
    int by = 10 + r * (BRICK_H + 2);
    for (int c = 0; c < BRICK_COLS; c++) {
      if (bricks[r][c].hp == 0) continue;
      int bx = 4 + c * (BRICK_W + 2);

      if (ballX + BALL_SIZE >= bx && ballX <= bx + BRICK_W &&
          ballY + BALL_SIZE >= by && ballY <= by + BRICK_H) {
        bricks[r][c].hp--;
        score += 10;
        ballVY = -ballVY;
        if (bricks[r][c].hp == 0 && bricks[r][c].hasPowerup) {
          spawnPowerup(bx + BRICK_W / 2.0, by + BRICK_H);
        }
        break;
      }
    }
  }

  // Powerups update
  for (int i = 0; i < 3; i++) {
    if (!powerups[i].active) continue;
    powerups[i].y += 1.0;
    if (powerups[i].y >= PADDLE_Y && powerups[i].y <= PADDLE_Y + PADDLE_H + 4 &&
        powerups[i].x >= paddleX - 2 && powerups[i].x <= paddleX + paddleW + 2) {
      powerups[i].active = false;
      if (powerups[i].kind == 0) {
        paddleW = basePaddleW + 8;
        widePaddleUntil = millis() + 8000;
      } else if (powerups[i].kind == 1) {
        ballVX *= 0.75;
        ballVY *= 0.75;
      } else if (lives < 5) {
        lives++;
      }
    } else if (powerups[i].y > H) {
      powerups[i].active = false;
    }
  }

  // Ball fell
  if (ballY > H) {
    lives--;
    if (lives <= 0) {
      showGameOver();
      return;
    }
    resetBall();
  }

  // Next level
  if (countActiveBricks() == 0) {
    level++;
    score += 100;
    resetLevel();
  }

  drawGame();
}
