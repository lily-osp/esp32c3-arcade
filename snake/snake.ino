#include <Arduino.h>
#include <Wire.h>
#include <Adafruit_GFX.h>
#include <Adafruit_SSD1306.h>
#include <Adafruit_MPU6050.h>
#include <Adafruit_Sensor.h>

#define W 128
#define H 64
#define CELL 4
#define COLS (W / CELL)
#define ROWS (H / CELL)
#define MAXLEN 90

Adafruit_SSD1306 d(W, H, &Wire, -1);
Adafruit_MPU6050 mpu;

int snakeX[MAXLEN];
int snakeY[MAXLEN];
int length;
int directionX;
int directionY;
int foodX;
int foodY;
int score;
int moveDelay;
unsigned long lastMove = 0;
unsigned long bootTime = 0;
bool started = false;
float previousAx = 0;
float previousAy = 0;
float previousAz = 0;

void placeFood() {
  bool occupied;
  do {
    foodX = random(COLS);
    foodY = random(ROWS);
    occupied = false;
    for (int i = 0; i < length; i++) {
      if (snakeX[i] == foodX && snakeY[i] == foodY) {
        occupied = true;
        break;
      }
    }
  } while (occupied);
}

void resetGame() {
  length = 7;
  directionX = 1;
  directionY = 0;
  score = 0;
  moveDelay = 135;

  for (int i = 0; i < length; i++) {
    snakeX[i] = 10 - i;
    snakeY[i] = ROWS / 2;
  }
  placeFood();
}

void showGameOver() {
  d.clearDisplay();
  d.setTextColor(SSD1306_WHITE);

  d.setTextSize(2);
  int16_t x1, y1;
  uint16_t textWidth, textHeight;
  d.getTextBounds("GAME OVER", 0, 0, &x1, &y1, &textWidth, &textHeight);
  d.setCursor((W - textWidth) / 2, 18);
  d.print("GAME OVER");

  char scoreText[20];
  snprintf(scoreText, sizeof(scoreText), "SCORE %d", score);
  d.setTextSize(1);
  d.getTextBounds(scoreText, 0, 0, &x1, &y1, &textWidth, &textHeight);
  d.setCursor((W - textWidth) / 2, 42);
  d.print(scoreText);

  d.display();
  delay(900);
  resetGame();
  started = false;

  // Re-arm shake detection from a fresh sensor reading.
  sensors_event_t a, g, t;
  mpu.getEvent(&a, &g, &t);
  previousAx = a.acceleration.x;
  previousAy = a.acceleration.y;
  previousAz = a.acceleration.z;
  bootTime = millis();
}

void showStartScreen() {
  d.clearDisplay();
  d.setTextColor(SSD1306_WHITE);

  int16_t x1, y1;
  uint16_t textWidth, textHeight;

  d.setTextSize(2);
  d.getTextBounds("SNAKE", 0, 0, &x1, &y1, &textWidth, &textHeight);
  d.setCursor((W - textWidth) / 2, 16);
  d.print("SNAKE");

  d.setTextSize(1);
  d.getTextBounds("SHAKE TO START", 0, 0, &x1, &y1, &textWidth, &textHeight);
  d.setCursor((W - textWidth) / 2, 42);
  d.print("SHAKE TO START");
  d.display();
}

void readTilt(const sensors_event_t &a) {

  // Correct MPU orientation for this board:
  // Y acceleration = screen left/right, X acceleration = screen up/down.
  float horizontal = a.acceleration.y;
  float vertical = a.acceleration.x;

  if (fabs(horizontal) > fabs(vertical)) {
    if (horizontal > 3.0 && directionX != -1) {
      directionX = 1;
      directionY = 0;
    } else if (horizontal < -3.0 && directionX != 1) {
      directionX = -1;
      directionY = 0;
    }
  } else {
    if (vertical > 3.0 && directionY != 1) {
      directionX = 0;
      directionY = -1;
    } else if (vertical < -3.0 && directionY != -1) {
      directionX = 0;
      directionY = 1;
    }
  }
}

void drawGame() {
  d.clearDisplay();

  for (int i = 0; i < length; i++) {
    int x = snakeX[i] * CELL;
    int y = snakeY[i] * CELL;
    if (i == 0) {
      d.fillRect(x, y, CELL, CELL, 1);
      d.drawPixel(x + 1, y + 1, 0);
    } else {
      d.fillRect(x, y, CELL - 1, CELL - 1, 1);
    }
  }

  int foodPixelX = foodX * CELL + 1;
  int foodPixelY = foodY * CELL + 1;
  d.fillRect(foodPixelX, foodPixelY, CELL - 1, CELL - 1, 1);

  // Score overlay: gameplay still uses the entire screen.
  d.setTextSize(1);
  d.setTextColor(SSD1306_WHITE, SSD1306_BLACK);
  d.setCursor(0, 0);
  d.print("S:");
  d.print(score);
  d.setTextColor(SSD1306_WHITE);
  d.display();
}

void setup() {
  Wire.begin(9, 8);
  Wire.setClock(800000);
  d.begin(SSD1306_SWITCHCAPVCC, 0x3C);
  d.clearDisplay();
  d.setTextColor(SSD1306_WHITE);
  d.setTextSize(2);
  d.setCursor(34, 22);
  d.print("SNAKE");
  d.display();
  delay(500);
  randomSeed(analogRead(0));

  mpu.begin(0x68, &Wire);
  mpu.setAccelerometerRange(MPU6050_RANGE_8_G);
  mpu.setFilterBandwidth(MPU6050_BAND_21_HZ);
  resetGame();
  bootTime = millis();
  sensors_event_t a, g, t;
  mpu.getEvent(&a, &g, &t);
  previousAx = a.acceleration.x;
  previousAy = a.acceleration.y;
  previousAz = a.acceleration.z;
  showStartScreen();
}

void loop() {
  sensors_event_t a, g, t;
  mpu.getEvent(&a, &g, &t);

  float shakeAmount =
    fabs(a.acceleration.x - previousAx) +
    fabs(a.acceleration.y - previousAy) +
    fabs(a.acceleration.z - previousAz);
  previousAx = a.acceleration.x;
  previousAy = a.acceleration.y;
  previousAz = a.acceleration.z;

  if (!started) {
    showStartScreen();
    if (millis() - bootTime > 1000 && shakeAmount > 5.0) {
      started = true;
      resetGame();
      lastMove = millis();
      delay(180);
    } else {
      delay(25);
    }
    return;
  }

  readTilt(a);

  if (millis() - lastMove < (unsigned long)moveDelay) {
    delay(3);
    return;
  }
  lastMove = millis();

  int nextX = snakeX[0] + directionX;
  int nextY = snakeY[0] + directionY;

  // No walls: wrap around every edge of the playfield.
  if (nextX < 0) nextX = COLS - 1;
  if (nextX >= COLS) nextX = 0;
  if (nextY < 0) nextY = ROWS - 1;
  if (nextY >= ROWS) nextY = 0;

  for (int i = 0; i < length; i++) {
    if (snakeX[i] == nextX && snakeY[i] == nextY) {
      showGameOver();
      return;
    }
  }

  bool ateFood = nextX == foodX && nextY == foodY;
  if (ateFood && length < MAXLEN) length++;

  for (int i = length - 1; i > 0; i--) {
    snakeX[i] = snakeX[i - 1];
    snakeY[i] = snakeY[i - 1];
  }
  snakeX[0] = nextX;
  snakeY[0] = nextY;

  if (ateFood) {
    score++;
    if (moveDelay > 55) moveDelay -= 3;
    placeFood();
  }

  drawGame();
}
