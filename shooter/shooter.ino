#include <Arduino.h>
#include <Wire.h>
#include <Adafruit_GFX.h>
#include <Adafruit_SSD1306.h>
#include <Adafruit_MPU6050.h>
#include <Adafruit_Sensor.h>

#define W 128
#define H 64
#define PLAYER_X 3
#define PLAYER_W 12
#define PLAYER_H 10
#define ENEMY_W 9
#define ENEMY_H 7
#define BOSS_W 20
#define BOSS_H 14
#define BOSS_MAX_HP 45
#define MIN_BOSS_HP 20
#define MAX_LIVES 12
#define MAX_BULLETS 12
#define MAX_HOSTILE_BULLETS 8
#define MAX_ENEMIES 8
#define MAX_EXPLOSIONS 10
#define SHAKE_THRESHOLD 8.0

Adafruit_SSD1306 d(W, H, &Wire, -1);
Adafruit_MPU6050 mpu;

struct Bullet {
  float x, y;
  bool active;
};

struct HostileBullet {
  float x, y;
  bool active;
};

struct Enemy {
  float x, y, vy;
  bool active;
};

struct Explosion {
  float x, y;
  int frame;
  bool active;
};

Bullet bullets[MAX_BULLETS];
HostileBullet hostileBullets[MAX_HOSTILE_BULLETS];
Enemy enemies[MAX_ENEMIES];
Explosion explosions[MAX_EXPLOSIONS];
float playerY = 27;
int score = 0;
int lives = 7;
int enemySpeed = 1;
int difficulty = 0;
unsigned long shotInterval = 220;
unsigned long spawnInterval = 750;
int maxEnemySpeed = 3;
int doubleShotChance = 5;
int enemiesDefeated = 0;
int nextBossAt = 110;
bool bossActive = false;
float bossX = W + BOSS_W;
float bossY = 25;
float bossVY = 0.6;
int bossHP = 0;
bool doubleShot = false;
bool tripleShot = false;
bool enemyFireUnlocked = false;
unsigned long doubleShotUntil = 0;
float fireRateMultiplier = 1.0;
int fireRateBoosts = 0;
unsigned long lastShot = 0;
unsigned long lastSpawn = 0;
unsigned long lastFrame = 0;
unsigned long shakeReadyAt = 0;
bool waitingForShake = true;
float previousAx = 0;
float previousAy = 0;
float previousAz = 0;

void resetGame() {
  playerY = (H - PLAYER_H) / 2.0;
  score = 0;
  lives = 7;
  enemiesDefeated = 0;
  nextBossAt = random(90, 131);
  bossActive = false;
  bossX = W + BOSS_W;
  bossY = 25;
  bossVY = 0.6;
  bossHP = 0;
  doubleShot = false;
  tripleShot = false;
  enemyFireUnlocked = false;
  doubleShotUntil = 0;
  fireRateMultiplier = 1.0;
  fireRateBoosts = 0;

  // Weighted modes: easy 40%, medium 30%, hard 20%, extreme 10%.
  int roll = random(0, 10);
  if (roll < 4) difficulty = 0;
  else if (roll < 7) difficulty = 1;
  else if (roll < 9) difficulty = 2;
  else difficulty = 3;

  if (difficulty == 0) {
    enemySpeed = 1;
    spawnInterval = 900;
    shotInterval = 190;
    maxEnemySpeed = 2;
    doubleShotChance = 5;
  } else if (difficulty == 1) {
    enemySpeed = 1;
    spawnInterval = 750;
    shotInterval = 220;
    maxEnemySpeed = 3;
    doubleShotChance = 6;
  } else if (difficulty == 2) {
    enemySpeed = 2;
    spawnInterval = 600;
    shotInterval = 245;
    maxEnemySpeed = 4;
    doubleShotChance = 7;
  } else {
    enemySpeed = 3;
    spawnInterval = 450;
    shotInterval = 270;
    maxEnemySpeed = 5;
    doubleShotChance = 8;
  }

  lastShot = millis();
  lastSpawn = millis();

  for (int i = 0; i < MAX_BULLETS; i++) bullets[i].active = false;
  for (int i = 0; i < MAX_HOSTILE_BULLETS; i++) hostileBullets[i].active = false;
  for (int i = 0; i < MAX_ENEMIES; i++) enemies[i].active = false;
  for (int i = 0; i < MAX_EXPLOSIONS; i++) explosions[i].active = false;
}

void rearmShake() {
  sensors_event_t a, g, t;
  mpu.getEvent(&a, &g, &t);
  previousAx = a.acceleration.x;
  previousAy = a.acceleration.y;
  previousAz = a.acceleration.z;
  shakeReadyAt = millis() + 1000;
  waitingForShake = true;
}

void drawCentered(const char *text, int size, int y) {
  d.setTextSize(size);
  int16_t x1, y1;
  uint16_t textWidth, textHeight;
  d.getTextBounds(text, 0, 0, &x1, &y1, &textWidth, &textHeight);
  d.setCursor((W - textWidth) / 2, y);
  d.print(text);
}

void showStartScreen() {
  d.clearDisplay();
  d.setTextColor(SSD1306_WHITE);
  drawCentered("SHOOTER", 2, 10);
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
  delay(1400);

  resetGame();
  rearmShake();
}

void spawnBullet(float y) {
  for (int i = 0; i < MAX_BULLETS; i++) {
    if (!bullets[i].active) {
      bullets[i].active = true;
      bullets[i].x = PLAYER_X + PLAYER_W;
      bullets[i].y = y;
      return;
    }
  }
}

int availableBulletSlots() {
  int available = 0;
  for (int i = 0; i < MAX_BULLETS; i++) {
    if (!bullets[i].active) available++;
  }
  return available;
}

void fireBullet() {
  unsigned long currentShotInterval =
    (unsigned long)(shotInterval / fireRateMultiplier);
  if (millis() - lastShot < currentShotInterval) return;

  int shotCount = tripleShot ? 3 : ((doubleShot || bossActive) ? 2 : 1);
  if (availableBulletSlots() < shotCount) return;

  lastShot = millis();
  if (shotCount == 3) {
    // Atomic trio: never emit only part of a triple shot.
    spawnBullet(playerY + 1);
    spawnBullet(playerY + PLAYER_H / 2);
    spawnBullet(playerY + PLAYER_H - 1);
  } else if (shotCount == 2) {
    // Atomic pair: never emit only one half of a double shot.
    spawnBullet(playerY + 2);
    spawnBullet(playerY + PLAYER_H - 2);
  } else {
    spawnBullet(playerY + PLAYER_H / 2);
  }
}

void spawnHostileBullet(float x, float y) {
  for (int i = 0; i < MAX_HOSTILE_BULLETS; i++) {
    if (!hostileBullets[i].active) {
      hostileBullets[i].active = true;
      hostileBullets[i].x = x;
      hostileBullets[i].y = y;
      return;
    }
  }
}

void spawnBoss() {
  bossActive = true;
  bossX = W + BOSS_W;
  bossY = random(12, H - BOSS_H - 1);
  bossVY = random(4, 9) / 10.0;
  if (random(2)) bossVY = -bossVY;
  bossHP = random(MIN_BOSS_HP, BOSS_MAX_HP + 1);
  nextBossAt = enemiesDefeated + random(90, 131);
}

void spawnEnemy() {
  if (millis() - lastSpawn < spawnInterval) return;
  lastSpawn = millis();

  for (int i = 0; i < MAX_ENEMIES; i++) {
    if (!enemies[i].active) {
      enemies[i].active = true;
      enemies[i].x = W + 2;
      enemies[i].y = random(10, H - ENEMY_H - 1);
      enemies[i].vy = random(-5, 6) / 10.0;
      return;
    }
  }
}

void spawnExplosion(float x, float y) {
  for (int i = 0; i < MAX_EXPLOSIONS; i++) {
    if (!explosions[i].active) {
      explosions[i].active = true;
      explosions[i].x = x;
      explosions[i].y = y;
      explosions[i].frame = 0;
      return;
    }
  }
}

void updateExplosions() {
  for (int i = 0; i < MAX_EXPLOSIONS; i++) {
    if (!explosions[i].active) continue;
    explosions[i].frame++;
    if (explosions[i].frame > 5) explosions[i].active = false;
  }
}

void updateBullets() {
  for (int i = 0; i < MAX_BULLETS; i++) {
    if (!bullets[i].active) continue;
    bullets[i].x += 3.5;
    if (bullets[i].x > W + 3) bullets[i].active = false;
  }
}

void updateBoss() {
  if (!bossActive) return;

  // The boss slowly travels across the screen instead of stopping at the edge.
  bossX -= 0.18;
  if (bossX < -BOSS_W) {
    bossX = W + BOSS_W;
    lives--;
    applyScorePenalty(10);
  }

  // Boss fire chance is 3% per update.
  if (random(1000000L) < 30000) {
    spawnHostileBullet(bossX - 1, bossY + BOSS_H / 2);
  }

  bossY += bossVY;
  if (bossY < 10 || bossY > H - BOSS_H - 1) {
    bossVY = -bossVY;
    bossY = constrain(bossY, 10, H - BOSS_H - 1);
  }
}

void updateEnemies() {
  for (int i = 0; i < MAX_ENEMIES; i++) {
    if (!enemies[i].active) continue;

    enemies[i].x -= enemySpeed;
    enemies[i].y += enemies[i].vy;
    if (enemies[i].y < 10 || enemies[i].y > H - ENEMY_H - 1) {
      enemies[i].vy = -enemies[i].vy;
      enemies[i].y = constrain(enemies[i].y, 10, H - ENEMY_H - 1);
    }

    if (enemies[i].x < -ENEMY_W) {
      enemies[i].active = false;
      lives--;
      continue;
    }

    // Enemy fire unlocks once the score first reaches 10 and stays enabled.
    if (!enemyFireUnlocked && score >= 10) enemyFireUnlocked = true;
    if (!enemyFireUnlocked) continue;

    // Base chance is 1.5%/1.5%/2%/2%, plus 0.05% every 15 score.
    float fireChance = enemyFireChancePercent();
    long fireChancePpm = (long)(fireChance * 10000.0);
    if (random(1000000L) < fireChancePpm) {
      spawnHostileBullet(enemies[i].x - 1, enemies[i].y + ENEMY_H / 2);
    }
  }
}

void updateHostileBullets() {
  for (int i = 0; i < MAX_HOSTILE_BULLETS; i++) {
    if (!hostileBullets[i].active) continue;
    hostileBullets[i].x -= 2.2;
    if (hostileBullets[i].x < -3) hostileBullets[i].active = false;
  }
}

float fireRateChancePercent() {
  if (fireRateBoosts >= 3) return 0.0;
  if (fireRateBoosts >= 1) return 1.0;
  return 0.5 + (enemiesDefeated / 25) * 0.5;
}

float enemyFireChancePercent() {
  float baseChance = difficulty <= 1 ? 1.5 : 2.0;
  float scoreBonus = score > 0 ? (score / 15) * 0.05 : 0.0;
  return baseChance + scoreBonus;
}

void applyScorePenalty(int amount) {
  score -= amount;
  if (score < -1) score = -1;
}

bool spriteFullyVisible(float x, float y, int width, int height) {
  return x >= 0 && y >= 0 &&
         x + width <= W && y + height <= H;
}

void checkCollisions() {
  for (int b = 0; b < MAX_BULLETS; b++) {
    if (!bullets[b].active || !bossActive) continue;

    bool bossFullyVisible = spriteFullyVisible(
      bossX, bossY, BOSS_W, BOSS_H);
    bool bossHit = bossFullyVisible &&
                   bullets[b].x >= bossX &&
                   bullets[b].x <= bossX + BOSS_W &&
                   bullets[b].y >= bossY - 2 &&
                   bullets[b].y <= bossY + BOSS_H;
    if (bossHit) {
      bullets[b].active = false;
      bossHP--;
      if (bossHP <= 0) {
        bossActive = false;
        lives = min(lives + 2, MAX_LIVES);
        spawnExplosion(bossX + BOSS_W / 2, bossY + BOSS_H / 2);
        spawnExplosion(bossX + 5, bossY + 4);
        spawnExplosion(bossX + BOSS_W - 5, bossY + 4);
      }
    }
  }

  for (int b = 0; b < MAX_BULLETS; b++) {
    if (!bullets[b].active) continue;

    for (int e = 0; e < MAX_ENEMIES; e++) {
      if (!enemies[e].active) continue;

      bool enemyFullyVisible = spriteFullyVisible(
        enemies[e].x, enemies[e].y, ENEMY_W, ENEMY_H);
      bool hit = enemyFullyVisible &&
                 bullets[b].x >= enemies[e].x &&
                 bullets[b].x <= enemies[e].x + ENEMY_W &&
                 bullets[b].y >= enemies[e].y - 2 &&
                 bullets[b].y <= enemies[e].y + ENEMY_H;
      if (hit) {
        bullets[b].active = false;
        enemies[e].active = false;
        spawnExplosion(enemies[e].x + ENEMY_W / 2,
                       enemies[e].y + ENEMY_H / 2);
        score++;
        enemiesDefeated++;
        if (score % 8 == 0 && enemySpeed < maxEnemySpeed) enemySpeed++;
        if (!bossActive && enemiesDefeated >= nextBossAt) spawnBoss();

        // Triple-shot chance: 0.3% per regular defeat.
        if (!doubleShot && !tripleShot && random(100000L) < 300) {
          tripleShot = true;
          doubleShotUntil = millis() + 7000;
        } else if (!doubleShot && !tripleShot && random(100) < doubleShotChance) {
          // Difficulty-scaled double shot chance: 5%, 6%, 7%, or 8%.
          doubleShot = true;
          doubleShotUntil = millis() + 5000;
        }

        // Up to three permanent fire-rate boosts can be earned.
        if (fireRateBoosts < 3) {
          float fireRateChance = fireRateChancePercent();
          if (random(10000) < (long)(fireRateChance * 100.0)) {
            fireRateMultiplier *= 1.05;
            fireRateBoosts++;
          }
        }
        break;
      }
    }
  }

  for (int i = 0; i < MAX_HOSTILE_BULLETS; i++) {
    if (!hostileBullets[i].active) continue;

    bool hitPlayer = hostileBullets[i].x <= PLAYER_X + PLAYER_W &&
                     hostileBullets[i].x >= PLAYER_X &&
                     hostileBullets[i].y >= playerY &&
                     hostileBullets[i].y <= playerY + PLAYER_H;
    if (hitPlayer) {
      hostileBullets[i].active = false;
      applyScorePenalty(2);
    }
  }

  for (int e = 0; e < MAX_ENEMIES; e++) {
    if (!enemies[e].active) continue;

    bool hitPlayer = enemies[e].x <= PLAYER_X + PLAYER_W &&
                     enemies[e].x + ENEMY_W >= PLAYER_X &&
                     enemies[e].y + ENEMY_H >= playerY &&
                     enemies[e].y <= playerY + PLAYER_H;
    if (hitPlayer) {
      enemies[e].active = false;
      lives--;
    }
  }
}

const uint8_t HEART_OUTLINE[] PROGMEM = {
  0x6C, 0x82, 0x82, 0x44, 0x28, 0x10
};

const uint8_t HEART_HALF[] PROGMEM = {
  0x6C, 0xF2, 0xF2, 0x74, 0x28, 0x10
};

const uint8_t HEART_FULL[] PROGMEM = {
  0x6C, 0xFE, 0xFE, 0x7C, 0x38, 0x10
};

void drawHeart(int x, int y, int state) {
  if (state == 0) return;
  const uint8_t *bitmap = state == 3 ? HEART_FULL
                         : state == 2 ? HEART_HALF
                                      : HEART_OUTLINE;
  d.drawBitmap(x, y, bitmap, 7, 6, SSD1306_WHITE);
}

void drawGame() {
  d.clearDisplay();

  // Side-facing spacecraft with fuselage, nose, wings, cockpit, and engines.
  int shipY = (int)playerY;
  d.fillRect(PLAYER_X, shipY + 3, 6, 4, 1);
  d.fillTriangle(PLAYER_X + 3, shipY,
                 PLAYER_X + PLAYER_W, shipY + PLAYER_H / 2,
                 PLAYER_X + 3, shipY + PLAYER_H, 1);
  d.fillRect(PLAYER_X + 1, shipY + 1, 3, 2, 1);
  d.fillRect(PLAYER_X + 1, shipY + PLAYER_H - 3, 3, 2, 1);
  d.drawPixel(PLAYER_X + 7, shipY + 4, 0);
  d.drawPixel(PLAYER_X + 7, shipY + 5, 0);
  d.drawPixel(PLAYER_X - 1, shipY + 3, 1);
  d.drawPixel(PLAYER_X - 1, shipY + 6, 1);

  for (int i = 0; i < MAX_BULLETS; i++) {
    if (bullets[i].active) {
      d.drawFastHLine((int)bullets[i].x, (int)bullets[i].y, 5, 1);
    }
  }

  for (int i = 0; i < MAX_HOSTILE_BULLETS; i++) {
    if (hostileBullets[i].active) {
      d.fillCircle((int)hostileBullets[i].x, (int)hostileBullets[i].y, 1, 1);
    }
  }

  for (int i = 0; i < MAX_ENEMIES; i++) {
    if (!enemies[i].active) continue;
    int x = (int)enemies[i].x;
    int y = (int)enemies[i].y;
    // Larger alien craft: dome, body, wings, lights, and landing legs.
    d.fillRect(x + 2, y, 5, 2, 1);
    d.fillRect(x + 1, y + 2, 7, 3, 1);
    d.drawPixel(x, y + 3, 1);
    d.drawPixel(x + 8, y + 3, 1);
    d.drawPixel(x + 2, y + 5, 1);
    d.drawPixel(x + 6, y + 5, 1);
    d.drawPixel(x + 4, y + 1, 0);
  }

  for (int i = 0; i < MAX_EXPLOSIONS; i++) {
    if (!explosions[i].active) continue;
    int x = (int)explosions[i].x;
    int y = (int)explosions[i].y;
    int radius = explosions[i].frame + 1;
    d.drawPixel(x, y, 1);
    d.drawPixel(x - radius, y, 1);
    d.drawPixel(x + radius, y, 1);
    d.drawPixel(x, y - radius, 1);
    d.drawPixel(x, y + radius, 1);
    if (explosions[i].frame >= 2) {
      d.drawPixel(x - radius, y - radius, 1);
      d.drawPixel(x + radius, y + radius, 1);
      d.drawPixel(x + radius, y - radius, 1);
      d.drawPixel(x - radius, y + radius, 1);
    }
  }

  if (bossActive) {
    int x = (int)bossX;
    int y = (int)bossY;
    // Large boss craft with a reinforced body and twin engines.
    d.fillRect(x + 4, y, 12, 2, 1);
    d.fillRect(x + 2, y + 2, 16, 8, 1);
    d.fillRect(x, y + 5, BOSS_W, 4, 1);
    d.drawPixel(x + 3, y + 10, 1);
    d.drawPixel(x + 6, y + 11, 1);
    d.drawPixel(x + 13, y + 11, 1);
    d.drawPixel(x + 16, y + 10, 1);
    d.drawPixel(x + 8, y + 4, 0);
    d.drawPixel(x + 11, y + 4, 0);

    // Boss health bar below the HUD.
    d.drawRect(82, 9, 44, 3, 1);
    int hpWidth = (bossHP * 42) / BOSS_MAX_HP;
    if (hpWidth > 0) d.fillRect(83, 10, hpWidth, 1, 1);
  }

  // Keep score visible during play with an opaque background.
  char hud[20];
  snprintf(hud, sizeof(hud), "S:%d", score);
  d.setTextSize(1);
  d.setTextColor(SSD1306_WHITE, SSD1306_BLACK);
  d.setCursor(0, 0);
  d.print(hud);
  d.setTextColor(SSD1306_WHITE);

  int remainingLives = lives;
  // Four slots, each representing up to three lives.
  for (int heart = 0; heart < 4; heart++) {
    int state = 0;
    if (remainingLives >= 3) {
      state = 3;
      remainingLives -= 3;
    } else if (remainingLives >= 2) {
      state = 2;
      remainingLives -= 2;
    } else if (remainingLives >= 1) {
      state = 1;
      remainingLives--;
    }
    drawHeart(91 + heart * 9, 1, state);
  }
  d.display();
}

void setup() {
  Wire.begin(9, 8);
  Wire.setClock(800000);
  d.begin(SSD1306_SWITCHCAPVCC, 0x3C);
  d.clearDisplay();
  d.setTextColor(SSD1306_WHITE);
  drawCentered("SHOOTER", 2, 22);
  d.display();
  delay(500);

  randomSeed(analogRead(0));
  mpu.begin(0x68, &Wire);
  mpu.setAccelerometerRange(MPU6050_RANGE_8_G);
  mpu.setFilterBandwidth(MPU6050_BAND_21_HZ);
  resetGame();
  rearmShake();
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
    float shakeAmount =
      fabs(a.acceleration.x - previousAx) +
      fabs(a.acceleration.y - previousAy) +
      fabs(a.acceleration.z - previousAz);
    previousAx = a.acceleration.x;
    previousAy = a.acceleration.y;
    previousAz = a.acceleration.z;

    if (millis() >= shakeReadyAt && shakeAmount > SHAKE_THRESHOLD) {
      waitingForShake = false;
      resetGame();
      showCountdown();
      lastFrame = millis();
    } else {
      delay(25);
      return;
    }
  }

  // Correct MPU orientation: X controls screen up/down.
  playerY += a.acceleration.x * -0.75;
  playerY = constrain(playerY, 10, H - PLAYER_H);

  if ((doubleShot || tripleShot) && millis() >= doubleShotUntil) {
    doubleShot = false;
    tripleShot = false;
  }

  fireBullet();
  spawnEnemy();
  updateBullets();
  updateHostileBullets();
  updateEnemies();
  updateBoss();
  updateExplosions();
  checkCollisions();

  if (lives <= 0 || score <= -1) {
    showGameOver();
    return;
  }

  drawGame();
}
