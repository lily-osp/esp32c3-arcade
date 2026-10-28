#include <Arduino.h>
#include <Wire.h>
#include <Adafruit_GFX.h>
#include <Adafruit_SSD1306.h>
#include <Adafruit_MPU6050.h>
#include <Adafruit_Sensor.h>

#define W 128
#define H 64
#define ROAD_LEFT 9
#define ROAD_RIGHT 118
#define PLAYER_Y 53
#define PLAYER_W 9
#define PLAYER_H 7
#define CAR_W 9
#define CAR_H 7
#define MAX_OBSTACLES 6
#define START_LIVES 3
#define INVINCIBILITY_MS 1000UL
#define SPAWN_MIN_MS 430UL

Adafruit_SSD1306 d(W, H, &Wire, -1);
Adafruit_MPU6050 mpu;

struct Obstacle {
  float x;
  float y;
  bool active;
};

Obstacle obstacles[MAX_OBSTACLES];
float playerX = W / 2.0;
float playerVelocity = 0;
float roadSpeed = 1.35;
float roadOffset = 0;
int score = 0;
int lives = START_LIVES;
unsigned long lastFrame = 0;
unsigned long lastSpawn = 0;
unsigned long lastScore = 0;
unsigned long invincibleUntil = 0;
unsigned long shakeReadyAt = 0;
bool waitingForShake = true;
float previousAx = 0;
float previousAy = 0;
float previousAz = 0;

void drawCentered(const char *text, uint8_t size, int y) {
  d.setTextSize(size);
  int16_t x1, y1;
  uint16_t width, height;
  d.getTextBounds(text, 0, 0, &x1, &y1, &width, &height);
  d.setCursor((W - width) / 2, y);
  d.print(text);
}

void readSensorBaseline() {
  sensors_event_t a, g, t;
  mpu.getEvent(&a, &g, &t);
  previousAx = a.acceleration.x;
  previousAy = a.acceleration.y;
  previousAz = a.acceleration.z;
  shakeReadyAt = millis() + 1000;
}

void showStartScreen() {
  d.clearDisplay();
  d.setTextColor(SSD1306_WHITE);
  drawCentered("TILT RACER", 2, 10);
  drawCentered("SHAKE TO START", 1, 43);
  d.display();
}

void showCountdown() {
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
  d.setTextColor(SSD1306_WHITE);
  drawCentered("GO", 2, 20);
  d.display();
  delay(300);
}

void resetGame() {
  playerX = W / 2.0;
  playerVelocity = 0;
  roadSpeed = 1.35;
  roadOffset = 0;
  score = 0;
  lives = START_LIVES;
  invincibleUntil = 0;
  lastSpawn = millis();
  lastScore = millis();
  for (int i = 0; i < MAX_OBSTACLES; i++) obstacles[i].active = false;
}

int activeObstacleCount() {
  int count = 0;
  for (int i = 0; i < MAX_OBSTACLES; i++) {
    if (obstacles[i].active) count++;
  }
  return count;
}

bool positionHasNearbyObstacle(float x, float y, float distance) {
  for (int i = 0; i < MAX_OBSTACLES; i++) {
    if (!obstacles[i].active) continue;
    if (fabs(obstacles[i].x - x) < CAR_W + 4 &&
        fabs(obstacles[i].y - y) < distance) {
      return true;
    }
  }
  return false;
}

void spawnObstacle() {
  unsigned long elapsed = millis() - lastSpawn;
  unsigned long spawnInterval = 900;
  if (roadSpeed > 2.0) spawnInterval = 720;
  if (roadSpeed > 2.8) spawnInterval = 570;
  if (roadSpeed > 3.5) spawnInterval = SPAWN_MIN_MS;
  if (elapsed < spawnInterval) return;
  if (activeObstacleCount() >= MAX_OBSTACLES - 1) return;

  float spawnY = -CAR_H - random(0, 12);
  float spawnX = random(ROAD_LEFT + CAR_W / 2,
                        ROAD_RIGHT - CAR_W / 2 + 1);
  if (positionHasNearbyObstacle(spawnX, spawnY, 22)) return;

  for (int i = 0; i < MAX_OBSTACLES; i++) {
    if (!obstacles[i].active) {
      obstacles[i].active = true;
      // Traffic can appear anywhere across the road, not only at lane centers.
      obstacles[i].x = spawnX;
      obstacles[i].y = spawnY;
      lastSpawn = millis();
      return;
    }
  }
}

void updateRoad() {
  roadOffset += roadSpeed;
  if (roadOffset >= 12) roadOffset -= 12;
}

void updateObstacles() {
  for (int i = 0; i < MAX_OBSTACLES; i++) {
    if (!obstacles[i].active) continue;
    obstacles[i].y += roadSpeed;
    if (obstacles[i].y > H + CAR_H) obstacles[i].active = false;
  }
}

void updatePlayer(const sensors_event_t &a) {
  // Verified board mapping: MPU Y controls screen left/right.
  playerVelocity += a.acceleration.y * 0.10;
  playerVelocity *= 0.86;
  playerVelocity = constrain(playerVelocity, -2.8, 2.8);
  playerX += playerVelocity;
  playerX = constrain(playerX,
                      ROAD_LEFT + PLAYER_W / 2.0,
                      ROAD_RIGHT - PLAYER_W / 2.0);
}

bool overlaps(float ax, float ay, float aw, float ah,
              float bx, float by, float bw, float bh) {
  return ax - aw / 2 < bx + bw / 2 &&
         ax + aw / 2 > bx - bw / 2 &&
         ay - ah / 2 < by + bh / 2 &&
         ay + ah / 2 > by - bh / 2;
}

void checkCollisions() {
  if (millis() < invincibleUntil) return;

  for (int i = 0; i < MAX_OBSTACLES; i++) {
    if (!obstacles[i].active) continue;
    if (overlaps(playerX, PLAYER_Y, PLAYER_W, PLAYER_H,
                 obstacles[i].x, obstacles[i].y, CAR_W, CAR_H)) {
      obstacles[i].active = false;
      lives--;
      invincibleUntil = millis() + INVINCIBILITY_MS;
      playerVelocity = 0;
      if (lives > 0) playerX = W / 2.0;
      return;
    }
  }
}

void drawCar(float centerX, float centerY, bool player) {
  int x = (int)(centerX - CAR_W / 2);
  int y = (int)(centerY - CAR_H / 2);
  d.fillRect(x + 1, y, CAR_W - 2, CAR_H, SSD1306_WHITE);
  d.fillRect(x, y + 2, CAR_W, 3, SSD1306_WHITE);
  d.drawPixel(x + 2, y + 1, SSD1306_BLACK);
  d.drawPixel(x + CAR_W - 3, y + 1, SSD1306_BLACK);
  d.drawPixel(x + 1, y + CAR_H - 1, SSD1306_BLACK);
  d.drawPixel(x + CAR_W - 2, y + CAR_H - 1, SSD1306_BLACK);
  if (player) {
    d.drawFastHLine(x + 3, y + 3, 3, SSD1306_BLACK);
  } else {
    d.drawPixel(x + CAR_W / 2, y + 5, SSD1306_BLACK);
  }
}

void drawGame() {
  d.clearDisplay();

  // Road edges and scrolling lane markers.
  d.drawFastVLine(ROAD_LEFT, 0, H, SSD1306_WHITE);
  d.drawFastVLine(ROAD_RIGHT, 0, H, SSD1306_WHITE);
  for (int lane = 1; lane <= 2; lane++) {
    int x = ROAD_LEFT + lane * 36;
    for (int y = -12; y < H; y += 12) {
      int markerY = y + (int)roadOffset;
      d.drawFastVLine(x, markerY, 6, SSD1306_WHITE);
    }
  }

  for (int i = 0; i < MAX_OBSTACLES; i++) {
    if (obstacles[i].active) drawCar(obstacles[i].x, obstacles[i].y, false);
  }

  bool blinking = millis() < invincibleUntil && ((millis() / 100) % 2 == 0);
  if (!blinking) drawCar(playerX, PLAYER_Y, true);

  char hud[24];
  snprintf(hud, sizeof(hud), "S:%d L:%d", score, lives);
  d.setTextSize(1);
  d.setTextColor(SSD1306_WHITE, SSD1306_BLACK);
  d.setCursor(1, 1);
  d.print(hud);
  d.setTextColor(SSD1306_WHITE);
  d.display();
}

void showGameOver() {
  d.clearDisplay();
  d.setTextColor(SSD1306_WHITE);
  drawCentered("GAME OVER", 2, 12);
  char result[20];
  snprintf(result, sizeof(result), "SCORE %d", score);
  drawCentered(result, 1, 39);
  drawCentered("SHAKE TO REPLAY", 1, 53);
  d.display();
  delay(1200);
  resetGame();
  readSensorBaseline();
  waitingForShake = true;
}

void setup() {
  Wire.begin(9, 8);
  Wire.setClock(800000);
  d.begin(SSD1306_SWITCHCAPVCC, 0x3C);
  d.clearDisplay();
  d.setTextColor(SSD1306_WHITE);
  drawCentered("TILT RACER", 2, 22);
  d.display();
  delay(500);

  randomSeed(analogRead(0));
  mpu.begin(0x68, &Wire);
  mpu.setAccelerometerRange(MPU6050_RANGE_8_G);
  mpu.setFilterBandwidth(MPU6050_BAND_21_HZ);
  resetGame();
  readSensorBaseline();
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
    float shakeAmount = fabs(a.acceleration.x - previousAx) +
                        fabs(a.acceleration.y - previousAy) +
                        fabs(a.acceleration.z - previousAz);
    previousAx = a.acceleration.x;
    previousAy = a.acceleration.y;
    previousAz = a.acceleration.z;

    if (millis() >= shakeReadyAt && shakeAmount > 5.0) {
      waitingForShake = false;
      resetGame();
      showCountdown();
      lastFrame = millis();
    } else {
      delay(25);
      return;
    }
  }

  updatePlayer(a);
  updateRoad();
  updateObstacles();
  spawnObstacle();
  checkCollisions();

  if (millis() - lastScore >= 500) {
    score++;
    lastScore = millis();
    if (roadSpeed < 4.2) roadSpeed += 0.035;
  }

  if (lives <= 0) {
    showGameOver();
    return;
  }

  drawGame();
}
