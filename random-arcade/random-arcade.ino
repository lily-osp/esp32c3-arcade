#include <Arduino.h>
#include <Wire.h>
#include <Adafruit_GFX.h>
#include <Adafruit_SSD1306.h>
#include <Adafruit_MPU6050.h>
#include <Adafruit_Sensor.h>

// The six production sketches are embedded in separate namespaces.
namespace Pong {
void rearmShakeDetection();
#define setup pongSetup
#define loop pongLoop
#include "../pong/pong.ino"
#undef setup
#undef loop
}

namespace Snake {
#define setup snakeSetup
#define loop snakeLoop
#include "../snake/snake.ino"
#undef setup
#undef loop
}

namespace Shooter {
float enemyFireChancePercent();
void applyScorePenalty(int amount);
#define setup shooterSetup
#define loop shooterLoop
#include "../shooter/shooter.ino"
#undef setup
#undef loop
}

namespace TiltRacer {
#define setup racerSetup
#define loop racerLoop
#include "../tilt-racer/tilt-racer.ino"
#undef setup
#undef loop
}

namespace Breakout {
#define setup breakoutSetup
#define loop breakoutLoop
#include "../breakout/breakout.ino"
#undef setup
#undef loop
}

namespace FallDown {
#define setup falldownSetup
#define loop falldownLoop
#include "../falldown/falldown.ino"
#undef setup
#undef loop
}

enum GameId { PONG_GAME, SNAKE_GAME, SHOOTER_GAME, RACER_GAME, BREAKOUT_GAME, FALLDOWN_GAME };
GameId activeGame;
int previousGame = -1;
bool activeRound = false;

void selectAndStartGame() {
  int selected = random(0, 6);
  while (selected == previousGame) selected = random(0, 6);
  previousGame = selected;
  activeGame = (GameId)selected;

  if (activeGame == PONG_GAME) Pong::pongSetup();
  else if (activeGame == SNAKE_GAME) Snake::snakeSetup();
  else if (activeGame == SHOOTER_GAME) Shooter::shooterSetup();
  else if (activeGame == RACER_GAME) TiltRacer::racerSetup();
  else if (activeGame == BREAKOUT_GAME) Breakout::breakoutSetup();
  else if (activeGame == FALLDOWN_GAME) FallDown::falldownSetup();

  activeRound = false;
}

bool gameIsWaiting() {
  if (activeGame == PONG_GAME) return Pong::waitingForShake;
  if (activeGame == SNAKE_GAME) return !Snake::started;
  if (activeGame == SHOOTER_GAME) return Shooter::waitingForShake;
  if (activeGame == RACER_GAME) return TiltRacer::waitingForShake;
  if (activeGame == BREAKOUT_GAME) return Breakout::waitingForShake;
  return FallDown::waitingForShake;
}

void runActiveGame() {
  if (activeGame == PONG_GAME) Pong::pongLoop();
  else if (activeGame == SNAKE_GAME) Snake::snakeLoop();
  else if (activeGame == SHOOTER_GAME) Shooter::shooterLoop();
  else if (activeGame == RACER_GAME) TiltRacer::racerLoop();
  else if (activeGame == BREAKOUT_GAME) Breakout::breakoutLoop();
  else if (activeGame == FALLDOWN_GAME) FallDown::falldownLoop();
}

void setup() {
  randomSeed((unsigned long)esp_random());
  selectAndStartGame();
}

void loop() {
  runActiveGame();

  if (!activeRound && !gameIsWaiting()) activeRound = true;
  if (activeRound && gameIsWaiting()) {
    delay(100);
    selectAndStartGame();
  }
}
