#include <Arduino.h>
#include <Wire.h>
#include <Adafruit_GFX.h>
#include <Adafruit_SSD1306.h>
#include <Adafruit_MPU6050.h>
#include <Adafruit_Sensor.h>

#define W 128
#define H 64
#define PADDLE_H 14
#define PADDLE_W 3
#define BALL_SIZE 3
#define TOP_MARGIN 0
#define BOTTOM_MARGIN (H - PADDLE_H)
#define MAX_BALL_SPEED 4.2
#define WIN_SCORE 10
#define SHAKE_THRESHOLD 8.0

Adafruit_SSD1306 d(W, H, &Wire, -1);
Adafruit_MPU6050 mpu;

float ballX = W / 2.0;
float ballY = H / 2.0;
float ballVX = 1.8;
float ballVY = 0.7;
float playerY = 25;
float aiY = 25;
int playerScore = 0;
int aiScore = 0;
int aiMode = 1;
float aiError = 0;
unsigned long lastFrame = 0;
unsigned long shakeReadyAt = 0;
bool waitingForShake = false;
float previousAx = 0;
float previousAy = 0;
float previousAz = 0;

void serveBall(int direction) {
  ballX = W / 2.0;
  ballY = H / 2.0;
  ballVX = direction * 1.8;
  ballVY = random(-8, 9) / 10.0;
}

void resetMatch() {
  playerScore = 0;
  aiScore = 0;
  playerY = 25;
  aiY = 25;

  // Randomize the opponent for every new match.
  // Weighted random modes: easy 40%, medium 30%, hard 20%, extreme 10%.
  int roll = random(0, 10);
  if (roll < 4) aiMode = 0;
  else if (roll < 7) aiMode = 1;
  else if (roll < 9) aiMode = 2;
  else aiMode = 3;

  if (aiMode == 0) aiError = random(-12, 13);
  else if (aiMode == 1) aiError = random(-5, 6);
  else aiError = 0;

  serveBall(random(2) ? 1 : -1);
}

void updatePlayer(float verticalAcceleration) {
  // Correct board orientation: positive MPU X means screen-up.
  playerY += verticalAcceleration * -0.85;
  playerY = constrain(playerY, TOP_MARGIN, BOTTOM_MARGIN);
}

void updateAi() {
  float target = ballY - PADDLE_H / 2.0 + aiError;
  float difference = target - aiY;
  float response;
  float maxStep;

  if (aiMode == 0) {
    response = 0.045;
    maxStep = 0.75;
  } else if (aiMode == 1) {
    response = 0.085;
    maxStep = 1.5;
  } else if (aiMode == 2) {
    response = 0.18;
    maxStep = 3.0;
  } else {
    response = 0.32;
    maxStep = 4.5;
  }

  aiY += constrain(difference * response, -maxStep, maxStep);
  aiY = constrain(aiY, TOP_MARGIN, BOTTOM_MARGIN);
}

void bounceFromPaddle(float paddleY, bool movingRight) {
  float paddleCenter = paddleY + PADDLE_H / 2.0;
  float hitOffset = (ballY - paddleCenter) / (PADDLE_H / 2.0);

  ballVX = movingRight ? fabs(ballVX) : -fabs(ballVX);
  ballVX *= 1.04;
  ballVY += hitOffset * 0.65;

  float speed = sqrt(ballVX * ballVX + ballVY * ballVY);
  if (speed > MAX_BALL_SPEED) {
    ballVX = ballVX / speed * MAX_BALL_SPEED;
    ballVY = ballVY / speed * MAX_BALL_SPEED;
  }
}

void showWinner(bool playerWon) {
  const char *winnerText = playerWon ? "YOU WIN" : "AI WINS";

  d.clearDisplay();
  d.setTextColor(SSD1306_WHITE);
  d.setTextSize(2);
  int16_t x1, y1;
  uint16_t textWidth, textHeight;
  d.getTextBounds(winnerText, 0, 0, &x1, &y1, &textWidth, &textHeight);
  d.setCursor((W - textWidth) / 2, 16);
  d.print(winnerText);

  char finalScore[16];
  snprintf(finalScore, sizeof(finalScore), "%d:%d", playerScore, aiScore);
  d.setTextSize(1);
  d.getTextBounds(finalScore, 0, 0, &x1, &y1, &textWidth, &textHeight);
  d.setCursor((W - textWidth) / 2, 44);
  d.print(finalScore);
  d.display();

  delay(1800);
  resetMatch();
  rearmShakeDetection();
}

void showShakeScreen() {
  d.clearDisplay();
  d.setTextColor(SSD1306_WHITE);
  d.setTextSize(2);
  int16_t x1, y1;
  uint16_t textWidth, textHeight;
  d.getTextBounds("PONG", 0, 0, &x1, &y1, &textWidth, &textHeight);
  d.setCursor((W - textWidth) / 2, 12);
  d.print("PONG");

  d.setTextSize(1);
  d.getTextBounds("SHAKE TO START", 0, 0, &x1, &y1, &textWidth, &textHeight);
  d.setCursor((W - textWidth) / 2, 42);
  d.print("SHAKE TO START");
  d.display();
}

void showCountdown() {
  for (int number = 3; number >= 1; number--) {
    d.clearDisplay();
    d.setTextColor(SSD1306_WHITE);
    d.setTextSize(4);
    char countdownText[2];
    snprintf(countdownText, sizeof(countdownText), "%d", number);
    int16_t x1, y1;
    uint16_t textWidth, textHeight;
    d.getTextBounds(countdownText, 0, 0, &x1, &y1, &textWidth, &textHeight);
    d.setCursor((W - textWidth) / 2, 14);
    d.print(countdownText);
    d.display();
    delay(1000);
  }

  d.clearDisplay();
  d.setTextColor(SSD1306_WHITE);
  d.setTextSize(2);
  int16_t x1, y1;
  uint16_t textWidth, textHeight;
  d.getTextBounds("GO", 0, 0, &x1, &y1, &textWidth, &textHeight);
  d.setCursor((W - textWidth) / 2, 20);
  d.print("GO");
  d.display();
  delay(300);
}

void rearmShakeDetection() {
  sensors_event_t a, g, t;
  mpu.getEvent(&a, &g, &t);
  previousAx = a.acceleration.x;
  previousAy = a.acceleration.y;
  previousAz = a.acceleration.z;
  shakeReadyAt = millis() + 1000;
  waitingForShake = true;
}

void scorePoint(bool playerWon) {
  if (playerWon) {
    playerScore++;
  } else {
    aiScore++;
  }

  if (playerScore >= WIN_SCORE || aiScore >= WIN_SCORE) {
    showWinner(playerScore >= WIN_SCORE);
    return;
  }

  serveBall(playerWon ? -1 : 1);
  delay(350);
}

void drawGame() {
  d.clearDisplay();

  for (int y = TOP_MARGIN; y < H; y += 5) {
    d.drawPixel(W / 2, y, 1);
  }

  d.fillRect(2, (int)playerY, PADDLE_W, PADDLE_H, 1);
  d.fillRect(W - 5, (int)aiY, PADDLE_W, PADDLE_H, 1);
  d.fillRect((int)ballX, (int)ballY, BALL_SIZE, BALL_SIZE, 1);

  // Score overlay, like Snake: no space is reserved for the HUD.
  char scoreText[16];
  snprintf(scoreText, sizeof(scoreText), "%d:%d", playerScore, aiScore);
  d.setTextSize(1);
  d.setTextColor(SSD1306_WHITE, SSD1306_BLACK);
  int16_t x1, y1;
  uint16_t scoreWidth, scoreHeight;
  d.getTextBounds(scoreText, 0, 0, &x1, &y1, &scoreWidth, &scoreHeight);
  d.setCursor((W - scoreWidth) / 2, 0);
  d.print(scoreText);
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
  d.setCursor(46, 22);
  d.print("PONG");
  d.display();
  delay(500);
  randomSeed(analogRead(0));

  mpu.begin(0x68, &Wire);
  mpu.setAccelerometerRange(MPU6050_RANGE_8_G);
  mpu.setFilterBandwidth(MPU6050_BAND_21_HZ);
  resetMatch();
  rearmShakeDetection();
  showShakeScreen();
}

void loop() {
  if (millis() - lastFrame < 18) {
    delay(2);
    return;
  }
  lastFrame = millis();

  sensors_event_t a, g, t;
  mpu.getEvent(&a, &g, &t);

  if (waitingForShake) {
    showShakeScreen();
    float shakeAmount =
      fabs(a.acceleration.x - previousAx) +
      fabs(a.acceleration.y - previousAy) +
      fabs(a.acceleration.z - previousAz);
    previousAx = a.acceleration.x;
    previousAy = a.acceleration.y;
    previousAz = a.acceleration.z;

    if (millis() >= shakeReadyAt && shakeAmount > SHAKE_THRESHOLD) {
      waitingForShake = false;
      resetMatch();
      showCountdown();
      lastFrame = millis();
    } else {
      delay(25);
      return;
    }
  }

  updatePlayer(a.acceleration.x);
  updateAi();

  ballX += ballVX;
  ballY += ballVY;

  if (ballY <= TOP_MARGIN) {
    ballY = TOP_MARGIN;
    ballVY = fabs(ballVY);
  }
  if (ballY >= H - BALL_SIZE) {
    ballY = H - BALL_SIZE;
    ballVY = -fabs(ballVY);
  }

  if (ballVX < 0 && ballX <= 5 &&
      ballX >= 2 && ballY + BALL_SIZE >= playerY &&
      ballY <= playerY + PADDLE_H) {
    ballX = 5;
    bounceFromPaddle(playerY, true);
  }

  if (ballVX > 0 && ballX + BALL_SIZE >= W - 5 &&
      ballX <= W - 2 && ballY + BALL_SIZE >= aiY &&
      ballY <= aiY + PADDLE_H) {
    ballX = W - 8;
    bounceFromPaddle(aiY, false);
  }

  if (ballX < -BALL_SIZE) {
    scorePoint(false);
    if (waitingForShake) return;
  }
  if (ballX > W) {
    scorePoint(true);
    if (waitingForShake) return;
  }

  drawGame();
}
