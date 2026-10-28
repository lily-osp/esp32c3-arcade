#include <Arduino.h>
#include <Wire.h>
#include <Adafruit_GFX.h>
#include <Adafruit_SSD1306.h>
#include <Adafruit_MPU6050.h>
#include <Adafruit_Sensor.h>

#define W 128
#define H 64
#define HUD_H 8
#define CEILING_Y 10.0
#define BALL_RADIUS 2.0
#define FLOOR_H 2
#define NUM_FLOORS 5
#define FLOOR_SPACING 13.0
#define SHAKE_THRESHOLD 6.0

Adafruit_SSD1306 d(W, H, &Wire, -1);
Adafruit_MPU6050 mpu;

struct Floor {
  float y;
  float gapX;
  float gapW;
  float gapVX;
  bool passed;
  bool gemActive;
  float gemX;
  uint8_t gemType; // 0 = Bonus (+50), 1 = Slow (4s), 2 = Life (+1)
};

Floor floors[NUM_FLOORS];
float ballX = 64.0;
float ballY = 20.0;
float ballVX = 0.0;
float ballVY = 0.0;
int score = 0;
int lives = 3;
uint8_t difficulty = 0; // 0: Easy, 1: Medium, 2: Hard, 3: Extreme
float baseGapW = 24.0;
float baseRiseSpeed = 0.55;
float maxRiseSpeed = 1.10;
int gemChance = 30;
unsigned long slowMotionUntil = 0;
unsigned long invulnerableUntil = 0;

unsigned long lastFrame = 0;
unsigned long shakeReadyAt = 0;
bool waitingForShake = true;
float previousAx = 0;
float previousAy = 0;
float previousAz = 0;

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

void drawCentered(const char *text, int size, int y) {
  d.setTextSize(size);
  int16_t x1, y1;
  uint16_t tw, th;
  d.getTextBounds(text, 0, 0, &x1, &y1, &tw, &th);
  d.setCursor((W - tw) / 2, y);
  d.print(text);
}

void generateFloor(Floor &f, float y) {
  f.y = y;
  f.gapW = baseGapW;
  float maxGapX = (float)W - 4.0 - f.gapW;
  f.gapX = random(4, (int)maxGapX);

  f.gapVX = 0.0;
  if (difficulty == 1 && random(100) < 30) {
    f.gapVX = (random(2) == 0) ? 0.35 : -0.35;
  } else if (difficulty >= 2 && random(100) < 60) {
    float sign = (random(2) == 0) ? 1.0 : -1.0;
    f.gapVX = sign * (0.4 + (difficulty * 0.15));
  }

  f.passed = false;
  f.gemActive = (random(100) < gemChance);
  f.gemX = 10.0;
  f.gemType = 0;

  if (f.gemActive) {
    int gemRoll = random(10);
    if (gemRoll < 5) f.gemType = 0;
    else if (gemRoll < 8) f.gemType = 1;
    else f.gemType = 2;

    if (f.gapX > 20.0 && random(2) == 0) {
      f.gemX = random(4, (int)(f.gapX - 4.0));
    } else {
      int rightStart = (int)(f.gapX + f.gapW + 4.0);
      if (rightStart < W - 6) {
        f.gemX = random(rightStart, W - 6);
      } else {
        f.gemX = 8.0;
      }
    }
  }
}

void resetGame() {
  score = 0;
  slowMotionUntil = 0;
  invulnerableUntil = millis() + 1500;

  // 4-tier weighted random difficulty: Easy 40%, Medium 30%, Hard 20%, Extreme 10%
  int roll = random(10);
  if (roll < 4) {
    difficulty = 0; // Easy
    baseGapW = 26.0;
    baseRiseSpeed = 0.52;
    maxRiseSpeed = 0.95;
    lives = 4;
    gemChance = 35;
  } else if (roll < 7) {
    difficulty = 1; // Medium
    baseGapW = 21.0;
    baseRiseSpeed = 0.68;
    maxRiseSpeed = 1.15;
    lives = 3;
    gemChance = 25;
  } else if (roll < 9) {
    difficulty = 2; // Hard
    baseGapW = 16.0;
    baseRiseSpeed = 0.85;
    maxRiseSpeed = 1.35;
    lives = 2;
    gemChance = 18;
  } else {
    difficulty = 3; // Extreme
    baseGapW = 13.0;
    baseRiseSpeed = 1.05;
    maxRiseSpeed = 1.55;
    lives = 1;
    gemChance = 12;
  }

  for (int i = 0; i < NUM_FLOORS; i++) {
    float startY = (float)H - (i * FLOOR_SPACING);
    generateFloor(floors[i], startY);
    floors[i].gemActive = false;
  }

  ballX = 64.0;
  ballY = 18.0;
  ballVX = 0.0;
  ballVY = 0.0;
}

void showStartScreen() {
  d.clearDisplay();
  d.setTextColor(SSD1306_WHITE);
  drawCentered("FALL DOWN", 2, 10);
  drawCentered("SHAKE TO START", 1, 43);
  d.display();
}

void showCountdown() {
  d.clearDisplay();
  d.setTextColor(SSD1306_WHITE);
  drawCentered("FALL DOWN", 2, 10);
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

void drawGame(unsigned long now) {
  d.clearDisplay();

  // Ceiling spikes
  d.drawLine(0, HUD_H, W - 1, HUD_H, 1);
  for (int sx = 3; sx < W - 3; sx += 6) {
    d.drawPixel(sx, HUD_H + 1, 1);
  }

  // Floors
  for (int i = 0; i < NUM_FLOORS; i++) {
    int fy = (int)floors[i].y;
    if (fy < HUD_H || fy >= H) continue;

    int gx = (int)floors[i].gapX;
    int gw = (int)floors[i].gapW;

    if (gx > 0) {
      d.fillRect(0, fy, gx, FLOOR_H, 1);
    }
    int rightX = gx + gw;
    if (rightX < W) {
      d.fillRect(rightX, fy, W - rightX, FLOOR_H, 1);
    }

    // Gem item
    if (floors[i].gemActive) {
      int gemX = (int)floors[i].gemX;
      int gemY = (int)(floors[i].y - 3.0);
      if (gemY >= HUD_H && gemY < H - 2) {
        if (floors[i].gemType == 0) {
          // Bonus star
          d.drawPixel(gemX, gemY - 2, 1);
          d.drawPixel(gemX - 1, gemY - 1, 1);
          d.drawPixel(gemX + 1, gemY - 1, 1);
          d.drawPixel(gemX, gemY, 1);
        } else if (floors[i].gemType == 1) {
          // Slow motion
          d.fillRect(gemX - 2, gemY - 2, 5, 3, 1);
          d.drawPixel(gemX, gemY - 1, 0);
        } else {
          // Life heart
          d.drawPixel(gemX - 1, gemY - 2, 1);
          d.drawPixel(gemX + 1, gemY - 2, 1);
          d.drawPixel(gemX, gemY - 1, 1);
          d.drawPixel(gemX, gemY, 1);
        }
      }
    }
  }

  // Ball
  bool isVisible = (now >= invulnerableUntil) || ((now / 100) % 2 == 0);
  if (isVisible) {
    int bx = (int)ballX;
    int by = (int)ballY;
    d.fillCircle(bx, by, 2, 1);
    d.drawPixel(bx, by, 0); // Glint
  }

  // HUD
  char hud[28];
  if (now < slowMotionUntil) {
    snprintf(hud, sizeof(hud), "S:%d L:%d SLOW", score, lives);
  } else {
    snprintf(hud, sizeof(hud), "S:%d L:%d %s", score, lives, getDifficultyTag());
  }
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
  drawCentered("FALL DOWN", 2, 22);
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
  unsigned long now = millis();
  if (now - lastFrame < 20) {
    delay(2);
    return;
  }
  lastFrame = now;

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

    if (now >= shakeReadyAt && shake > SHAKE_THRESHOLD) {
      waitingForShake = false;
      resetGame();
      showCountdown();
      lastFrame = millis();
    } else {
      delay(25);
      return;
    }
  }

  // Calculate current floor rise speed
  float speedProgression = (score / 100.0) * 0.03;
  float riseSpeed = min(baseRiseSpeed + speedProgression, maxRiseSpeed);
  if (now < slowMotionUntil) {
    riseSpeed *= 0.50;
  }

  // Tilt steering
  float steerAccel = a.acceleration.y * 0.58;
  ballVX = (ballVX + steerAccel) * 0.86;
  ballX += ballVX;

  // Wall collisions
  float minX = BALL_RADIUS + 1.0;
  float maxX = (float)W - BALL_RADIUS - 1.0;
  if (ballX <= minX) {
    ballX = minX;
    ballVX = -ballVX * 0.25;
  } else if (ballX >= maxX) {
    ballX = maxX;
    ballVX = -ballVX * 0.25;
  }

  // Gravity
  ballVY = min(ballVY + 0.30f, 3.6f);
  float prevBallY = ballY;
  ballY += ballVY;

  // Floor collision
  bool onFloor = false;
  float floorLiftY = 0.0;

  for (int i = 0; i < NUM_FLOORS; i++) {
    float floorTop = floors[i].y;
    float ballBottom = ballY + BALL_RADIUS;
    float prevBallBottom = prevBallY + BALL_RADIUS;

    if (prevBallBottom <= floorTop + 1.5 && ballBottom >= floorTop) {
      bool inGap = (ballX >= floors[i].gapX + 1.0) &&
                   (ballX <= floors[i].gapX + floors[i].gapW - 1.0);

      if (inGap) {
        if (!floors[i].passed) {
          floors[i].passed = true;
          score += 10;
        }
      } else {
        ballY = floorTop - BALL_RADIUS;
        ballVY = 0.0;
        onFloor = true;
        floorLiftY = riseSpeed;
      }
    }

    // Gem collection
    if (floors[i].gemActive) {
      float distX = fabs(ballX - floors[i].gemX);
      float distY = fabs(ballY - (floors[i].y - 2.0));
      if (distX < 5.0 && distY < 5.0) {
        floors[i].gemActive = false;
        if (floors[i].gemType == 0) score += 50;
        else if (floors[i].gemType == 1) slowMotionUntil = now + 4000;
        else {
          if (lives < 5) lives++;
          else score += 100;
        }
      }
    }
  }

  if (onFloor) {
    ballY -= floorLiftY;
  }

  // Move floors
  float highestFloorY = 0.0;
  for (int i = 0; i < NUM_FLOORS; i++) {
    if (floors[i].y > highestFloorY) highestFloorY = floors[i].y;
  }

  for (int i = 0; i < NUM_FLOORS; i++) {
    floors[i].y -= riseSpeed;

    if (fabs(floors[i].gapVX) > 0.01) {
      floors[i].gapX += floors[i].gapVX;
      float maxGx = (float)W - 4.0 - floors[i].gapW;
      if (floors[i].gapX <= 4.0) {
        floors[i].gapX = 4.0;
        floors[i].gapVX = -floors[i].gapVX;
      } else if (floors[i].gapX >= maxGx) {
        floors[i].gapX = maxGx;
        floors[i].gapVX = -floors[i].gapVX;
      }
    }

    if (floors[i].y < (HUD_H - 1.0)) {
      float newY = max(highestFloorY, (float)H) + FLOOR_SPACING;
      generateFloor(floors[i], newY);
      highestFloorY = newY;
    }
  }

  // Ceiling hazard
  if (ballY - BALL_RADIUS <= CEILING_Y) {
    if (now >= invulnerableUntil) {
      lives--;
      if (lives <= 0) {
        showGameOver();
        return;
      }
      ballY = 36.0;
      ballX = 64.0;
      ballVY = 0.0;
      invulnerableUntil = now + 1800;
    } else {
      ballY = CEILING_Y + BALL_RADIUS + 1.0;
      ballVY = 1.0;
    }
  }

  // Bottom boundary
  if (ballY + BALL_RADIUS > H) {
    ballY = H - BALL_RADIUS;
    ballVY = 0.0;
  }

  drawGame(now);
}
